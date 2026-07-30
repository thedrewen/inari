use futures_util::{SinkExt, StreamExt};
use warp::filters::ws::WebSocket;

use crate::core::{client::Client, manager::Manager};

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
        match result {
            Ok(msg) => {
                
            },
            Err(_) => break
        }
    }

    manager.disconnect_client(id).await;
}