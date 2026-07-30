use futures_util::StreamExt;
use warp::filters::ws::WebSocket;

pub async fn handle_client(ws: WebSocket, id: usize) {
    let (mut tx, mut rx) = ws.split();
}