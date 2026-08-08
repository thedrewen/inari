use crate::app::App;

mod app;
mod utils;

#[tokio::main]
async fn main() {
    let app = App::new();
}