use warp::Filter;

use crate::utils::config::Config;

pub fn index(config: &Config) -> impl Filter<> {
    warp::get()
        .and(warp::fs::dir("./static"))
        .with(warp::reply::with::header("X-Hash-Check", "5f671f128f0be6dd3d0d8e5713fa03b17957c805a077c24f1a72c5871d0919b9"))
        .with(warp::reply::with::header("X-App-Version", config.app_version()))
}