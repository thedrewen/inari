use std::sync::atomic::AtomicUsize;

use warp::{Filter, reply::Reply};

use crate::{core::manager::Manager, handle_client::handle_client};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub async fn all() -> impl Filter<Extract = impl Reply> + Clone {

    let static_route = warp::get()
        .and(warp::fs::dir("./static"))
        .with(warp::reply::with::header("X-Hash-Check", "5f671f128f0be6dd3d0d8e5713fa03b17957c805a077c24f1a72c5871d0919b9"));

    let manager = Manager::new().await;

    let manager_filter = warp::any().map(move || manager.clone());

    let root = warp::path!("ws")
        .and(warp::ws())
        .and(manager_filter)
        .map(|ws: warp::ws::Ws, manager| {
            let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            ws.on_upgrade(move |websocket| handle_client(websocket, manager, id))
        }).or(static_route);
    
    root
}