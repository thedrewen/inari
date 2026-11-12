use crate::app::App;

mod app;
mod classes;
mod routes;
mod utils;

#[tokio::main]
async fn main() {
    let app = App::new().await;
    app.run().await;
}
