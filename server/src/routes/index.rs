use warp::{Filter, Rejection, Reply};

pub fn index(
    app_version: String,
    default_channel: String,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let html = std::fs::read_to_string("./static/index.html")
        .expect("index.html not found !")
        .replace(
            "__DEFAULT_CHANNEL__",
            &html_escape::encode_double_quoted_attribute(&default_channel),
        );

    let index_html = warp::path::end().map(move || warp::reply::html(html.clone()));
    let assets = warp::fs::dir("./static");

    warp::get()
        .and(index_html.or(assets))
        .with(warp::reply::with::header(
            "X-Hash-Check",
            "5f671f128f0be6dd3d0d8e5713fa03b17957c805a077c24f1a72c5871d0919b9",
        ))
        .with(warp::reply::with::header("X-App-Version", app_version))
}
