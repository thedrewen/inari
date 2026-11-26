use std::sync::atomic::Ordering;

use serde::Serialize;
use warp::{Filter, Rejection, Reply};

use crate::{app::Bundle, classes::mediasoup::router::InariRouter};

#[derive(Serialize)]
pub struct Client {
    id: usize,
    display_name: String,
    is_force_muted: bool,
    is_force_deafened: bool,
    is_muted: bool,
    is_deafened: bool,
    head_orientation: [f64; 3],
}

#[derive(Serialize)]
pub struct Channel {
    name: String,
    has_connector: bool,
    clients_count: usize,
    permanent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    clients: Option<Vec<Client>>,
}

impl Channel {
    pub fn new(router: &InariRouter) -> Self {
        Self {
            name: (*router.name()).clone(),
            clients_count: router.client_count().load(Ordering::Relaxed),
            has_connector: router.has_connector().load(Ordering::Relaxed),
            permanent: *router.permanent(),
            clients: None,
        }
    }

    pub async fn load_clients(mut v: Self, router: &InariRouter) -> Self {
        let mut clients = vec![];

        let rclients = router.clients().read().await.clone();
        for c in rclients.values() {
            clients.push(Client {
                id: *c.id(),
                display_name: c.display_name().read().await.clone(),
                head_orientation: c.head_orientation().read().await.clone(),

                is_force_muted: c.is_force_muted().load(Ordering::Relaxed),
                is_force_deafened: c.is_force_deafened().load(Ordering::Relaxed),
                is_muted: c.is_muted().load(Ordering::Relaxed),
                is_deafened: c.is_deafened().load(Ordering::Relaxed)
            });
        }

        v.clients = Some(clients);

        v
    }
}

pub fn channels(bundle: Bundle) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let bundle_filter = warp::any().map(move || bundle.clone());

    warp::path("channels")
        .and(bundle_filter)
        .then(|bundle: Bundle| async move {
            let mut rooms: Vec<Channel> = vec![];

            let routers = bundle.mediasoup_manager.routers().read().await;
            for router in routers.values() {
                rooms.push(Channel::new(router));
            }
            warp::reply::json(&rooms)
        })
}
