use serde::Serialize;
use warp::{Filter, Rejection, Reply};

use crate::app::Bundle;

#[derive(Serialize)]
pub struct Room {
    name: String,
    has_connector: bool,
    clients_count: usize,
}

pub fn channels(bundle: Bundle) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let bundle_filter = warp::any().map(move || bundle.clone());

    warp::path("channels")
        .and(bundle_filter)
        .then(|bundle: Bundle| async move {
            let mut rooms: Vec<Room> = vec![];

            let routers = bundle.mediasoup_manager.routers().read().await;
            for (name, router) in routers.iter() {
                rooms.push(Room {
                    name: name.clone(),
                    has_connector: router
                        .has_connector()
                        .load(std::sync::atomic::Ordering::Relaxed),
                    clients_count: router
                        .client_count()
                        .load(std::sync::atomic::Ordering::Relaxed),
                });
            }
            warp::reply::json(&rooms)
        })
}
