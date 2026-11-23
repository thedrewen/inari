use serde::Serialize;
use warp::{Filter, reject::Rejection, reply::Reply};

use crate::{
    app::Bundle,
    routes::api::{channel::channel, channels::channels},
};

pub mod channel;
pub mod channels;

#[derive(Serialize)]
pub struct ErrorResponse {
    status: String,
    code: usize,
}

pub fn api(bundle: Bundle) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("api").and(channel(bundle.clone()).or(channels(bundle.clone())))
}
