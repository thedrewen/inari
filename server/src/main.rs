use crate::app::App;

mod app;
mod utils;
mod routes;

#[tokio::main]
async fn main() {
    let app = App::new();
    app.run().await;
}