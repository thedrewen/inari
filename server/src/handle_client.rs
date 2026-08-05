use futures_util::{SinkExt, StreamExt};
use warp::filters::ws::WebSocket;

use crate::core::{client::Client, manager::Manager, requests::{WsRequests, WsRequestsKind}};

pub async fn handle_client(ws: WebSocket, manager: Manager, id: usize) {
    let (mut tx, mut rx) = ws.split();

    let (mpsc_tx, mut mpsc_rx) = tokio::sync::mpsc::unbounded_channel();
    
    let client = Client::new(id, mpsc_tx);
    manager.connect_client(client).await;

    tokio::spawn(async move {
        while let Some(result) = mpsc_rx.recv().await {
            let _ = tx.send(result).await;
        }
    });

    while let Some(result) = rx.next().await {
        println!("{:?}", result);

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
                    WsRequestsKind::Link { code } => {
                        manager.set_code(id, code).await;
                    },
                    WsRequestsKind::CreateProducerTransport {  } => {
                        let result = manager.create_producer_transport(id, value.transaction_uuid.clone()).await;
                        if result.is_none() {
                            break;
                        }
                    },
                    WsRequestsKind::ConnectProducerTransport { dtls_parameters } => {
                        manager.connect_producer_transport(id, dtls_parameters).await;
                    }
                }
            },
            Err(_) => break
        }
    }
    
    manager.disconnect_client(id).await;
}