use std::sync::atomic::AtomicUsize;

use warp::{Filter, reject::Rejection, reply::Reply};

use crate::{app::{Bundle}, classes::client::Client};

static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub fn ws(bundle: Bundle) -> impl Filter<Error = Rejection, Extract = impl Reply> + Clone {
    let bundle_filter = warp::any().map(move || bundle.clone());
    warp::path!("ws")
        .and(warp::ws())
        .and(bundle_filter)
        .map(|ws: warp::ws::Ws, bundle| {
            let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let client = Client::new(id);
            ws.on_upgrade(move |websocket| Client::handle(client, websocket, bundle) )
        })
}