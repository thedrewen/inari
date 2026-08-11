use warp::{Filter, reject::Rejection, reply::Reply};

pub fn index(app_version: String) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::get()
        .and(warp::fs::dir("./static"))
        .with(warp::reply::with::header("X-Hash-Check", "5f671f128f0be6dd3d0d8e5713fa03b17957c805a077c24f1a72c5871d0919b9"))
        .with(warp::reply::with::header("X-App-Version", app_version))
}