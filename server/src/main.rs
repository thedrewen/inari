use std::{env};

use dotenvy::dotenv;

mod handle_client;
mod logger;
mod routes;
mod core;

#[tokio::main]
async fn main() {
    
    dotenv().ok();

    println!("
 _____                 _ 
|_   _|               (_)
  | | _ __   __ _ _ __ _ 
  | || '_ \\ / _` | '__| |
 _| || | | | (_| | |  | |
 \\___/_| |_|\\__,_|_|  |_|
                         
                         ");

    let port: u16 = env::var("API_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3030);
    
    logger::log_info(String::from("Starting web server..."));
    
    logger::log_info(format!("Started on http://0.0.0.0:{port}"));
    warp::serve(routes::all().await).run(([0, 0, 0, 0], port)).await;
}
