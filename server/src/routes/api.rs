use warp::{Filter, reject::Rejection, reply::Reply};

use crate::{app::Bundle, routes::api::channels::channels};

pub mod channels;

pub fn api(bundle: Bundle) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("api").and(channels(bundle.clone()))
}
