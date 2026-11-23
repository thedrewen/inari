use warp::{Filter, reject::Rejection, reply::Reply};

use crate::{app::Bundle, routes::api::rooms::rooms};

pub mod rooms;

pub fn api(bundle: Bundle) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("api").and(rooms(bundle.clone()))
}
