use warp::{Filter, reject::Rejection, reply::Reply};

use crate::{
    app::Bundle,
    routes::api::{ErrorResponse, channels::Channel},
};

pub fn channel(bundle: Bundle) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let bundle_filter = warp::any().map(move || bundle.clone());

    warp::path!("channels" / String).and(bundle_filter).then(
        |channel_name: String, bundle: Bundle| async move {
            let channels = bundle.mediasoup_manager.routers().read().await;
            let channel = channels.get(&channel_name).clone();

            if let Some(channel) = channel {
                let reply_channel = Channel::new(channel);
                let reply_channel = Channel::load_clients(reply_channel, channel).await;
                let reply = warp::reply::json(&reply_channel);
                return warp::reply::with_status(reply, warp::http::StatusCode::OK);
            }

            let reply = warp::reply::json(&ErrorResponse {
                code: 404,
                status: "Channel not found.".to_string(),
            });
            warp::reply::with_status(reply, warp::http::StatusCode::NOT_FOUND)
        },
    )
}
