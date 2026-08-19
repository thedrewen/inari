use futures_util::{SinkExt, StreamExt};
use getset::Getters;
use tokio::sync::mpsc::UnboundedSender;
use warp::filters::ws::{Message, WebSocket};

use crate::{app::Bundle, classes::{mediasoup::router::InariRouter, ws::{requests::{WsRequests, WsRequestsKind}, responses::{WsResponses, WsResponsesKind}}}, utils::logger};

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct Client {
    id: usize,
    is_server_client: bool,
    channel: Option<InariRouter>,
    mpsc_tx: Option<UnboundedSender<Message>>,
    code: String,
    bundle: Bundle
}

impl Client {
    pub fn new(id: usize, bundle: Bundle) -> Self {
        Self {
            id,
            is_server_client: false,
            channel: None,
            mpsc_tx: None,
            code: String::new(),
            bundle
        }
    }

    pub fn send(&self, response: WsResponses) {
        if let Some(mpsc_tx) = self.mpsc_tx.clone() {
            let _ = mpsc_tx.send(
                Message::text(serde_json::to_string(&response).unwrap())
            );
        }
    }

    pub async fn handle(mut self, ws: WebSocket) {
        let (mut tx, mut rx) = ws.split();
        let (mpsc_tx, mut mpsc_rx) = tokio::sync::mpsc::unbounded_channel();
        self.mpsc_tx = Some(mpsc_tx);

        logger::log_info(format!("Client #{} has logged in.", self.id));

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
                        WsRequestsKind::Auth { is_server, auth_key, channel } => {
                            self.is_server_client = is_server;

                            if is_server {
                                logger::log_info(format!("Client #{} authentificated as server !", self.id));
                                if auth_key != self.bundle.config.api_key().clone() {
                                    break;
                                }
                            } else {
                                self.code = auth_key;
                            }

                            let router = self.bundle.mediasoup_manager.get_router(channel).await;
                            router.add_client(&self);
                            self.channel = Some(router);

                            self.send(WsResponses { kind: WsResponsesKind::Auth {  }, transaction_uuid: value.transaction_uuid.clone() });
                        },
                        
                        // Clients
                        WsRequestsKind::RtpCapabilities {  } => {
                            if !self.is_server_client {

                            }
                        },
                        WsRequestsKind::CreateTransport { is_producer } => {
                            // TODO
                        },
                        WsRequestsKind::ConnectTransport { is_producer, dtls_parameters } => {
                            // TODO
                        },
                        WsRequestsKind::Produce { kind, rtp_parameters } => {
                            // TODO
                        },
                        WsRequestsKind::Consume { producer_id, rtp_capabilities } => {

                        },
                        WsRequestsKind::FetchClients {  } => {
                            
                        },
                        
                        // Servers
                        WsRequestsKind::Clients { clients } => {
                            // TODO
                        }
                    }
                },
                Err(_) => break
            }
        }

        if let Some(channel) = &self.channel {
            channel.remove_client(&self).await;
        }

        logger::log_info(format!("Client #{} has logged out.", self.id));
    }
}