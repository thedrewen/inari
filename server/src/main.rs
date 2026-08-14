use crate::app::App;

mod app;
mod utils;
mod routes;
mod classes;

#[tokio::main]
async fn main() {
    let app = App::new().await;
    app.run().await;
}