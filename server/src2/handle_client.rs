use futures_util::{SinkExt, StreamExt};
use warp::filters::ws::{WebSocket};

use crate::core::{client::Client, manager::Manager, requests::{WsRequests, WsRequestsKind}, responses::WsResponses};

pub async fn handle_client(ws: WebSocket, manager: Manager, id: usize) {
    let (mut tx, mut rx) = ws.split();

    let (mpsc_tx, mut mpsc_rx) = tokio::sync::mpsc::unbounded_channel();

    let client = Client::new(id, mpsc_tx);
    let client_copy = client.clone();
    let mut is_client = true;
    manager.connect_client(client).await;

    tokio::spawn(async move {
        while let Some(result) = mpsc_rx.recv().await {
            let _ = tx.send(result).await;
        }
    });

    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                let value = msg.to_str();
                if value.is_err() {
                    continue;
                }
                let value = value.unwrap();
                let value = serde_json::from_str(value);
                if value.is_err() {
                    continue;
                }
                let value: WsRequests = value.unwrap();

                match value.kind {
                    // Clients :
                    WsRequestsKind::Link { code } => {
                        if is_client {
                            manager.set_code(id, code).await;
                        }
                    },
                    WsRequestsKind::RtpCapabilities {  } => {
                        if is_client {
                            let response = WsResponses {
                                kind: crate::core::responses::WsResponsesKind::RtpCapabilities { rtp_capabilities: manager.get_rtp_capabilities().await },
                                transaction_uuid: value.transaction_uuid.clone()
                            };
                            client_copy.send(response);
                        }
                    },
                    WsRequestsKind::CreateTransport { is_producer } => {
                        if is_client {
                            let result = manager.create_transport(id, is_producer, value.transaction_uuid.clone()).await;
                            if result.is_none() {
                                break;
                            }
                        }
                    },
                    WsRequestsKind::ConnectTransport { is_producer, dtls_parameters } => {
                        if is_client {
                             manager.connect_transport(id, is_producer, dtls_parameters, value.transaction_uuid).await;
                        }
                    },
                    WsRequestsKind::Produce { kind, rtp_parameters } => {
                        if is_client {
                            let result = manager.produce(id, kind, rtp_parameters, value.transaction_uuid.clone()).await;
                            if result.is_none() {
                                break;
                            }
                        }
                    },
                    WsRequestsKind::FetchClients {  } => {
                        if is_client {
                            manager.send_producers_ids(id, value.transaction_uuid.clone()).await;
                        }
                    },
                    WsRequestsKind::Consume { producer_id, rtp_capabilities } => {
                        if is_client {
                            let result = manager.consume(id, producer_id, rtp_capabilities, value.transaction_uuid.clone()).await;
                            if result.is_none() {
                                break;
                            }
                        }
                    }
                    // Server :
                    WsRequestsKind::Auth { api_key } => {
                        if manager.get_api_key() == api_key {
                            is_client = false;
                            manager.set_connected_as_server(id).await;
                            client_copy.send(WsResponses { kind: crate::core::responses::WsResponsesKind::Auth {  }, transaction_uuid: value.transaction_uuid.clone() });
                        } else {
                            break;
                        }
                    },
                    WsRequestsKind::Clients { clients } => {
                        if !is_client {
                            manager.handle_client_as_server(clients).await;
                        }
                    }
                }
            },
            Err(_) => break
        }
    }
    
    manager.disconnect_client(id).await;
}