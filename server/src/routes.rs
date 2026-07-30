use std::sync::atomic::AtomicUsize;

use warp::{Filter, reply::Reply};

use crate::handle_client::handle_client;

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub fn all() -> impl Filter<Extract = impl Reply> + Clone {
    let root = warp::path!("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            ws.on_upgrade(move |websocket| handle_client(websocket, id))
        });
    
    root
}