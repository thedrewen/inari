use tokio::sync::mpsc::UnboundedSender;
use warp::filters::ws::Message;

use crate::core::responses::WsResponses;

#[derive(Clone)]
pub struct Client {
    id: usize,
    mpsc_tx: UnboundedSender<Message>,
    connected: bool,
    pub code: String
}

impl Client {
    pub fn new(id: usize, mpsc_tx: UnboundedSender<Message>) -> Self {
        Self {
            id,
            mpsc_tx,
            connected: false,
            code: String::new()
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn send(&self, response: WsResponses) {
        let _ = self.mpsc_tx.send(
            Message::text(
                serde_json::to_string(&response).unwrap()
            )
        );
    }
}