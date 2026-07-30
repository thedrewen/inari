use tokio::sync::mpsc::UnboundedSender;
use warp::filters::ws::Message;

#[derive(Clone)]
pub struct Client {
    id: usize,
    mpsc_tx: UnboundedSender<Message>
}

impl Client {
    pub fn new(id: usize, mpsc_tx: UnboundedSender<Message>) -> Self {
        Self {
            id,
            mpsc_tx
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
}