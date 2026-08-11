use warp::filters::ws::WebSocket;

use crate::app::{App, Bundle};

pub struct Client {
    id: usize
}

impl Client {
    pub fn new(id: usize) -> Self {
        Self {
            id
        }
    }

    pub async fn handle(self, ws: WebSocket, bundle: Bundle) {
        
    } 
}