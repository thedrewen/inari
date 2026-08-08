use std::{env};

use dotenvy::dotenv;

mod handle_client;
mod logger;
mod routes;
mod core;

#[tokio::main]
async fn main() {
    
    dotenv().ok();

    let max_distance: f64 = env::var("MAX_DISTANCE")
            .unwrap_or(String::from("20"))
            .parse()
            .unwrap_or(20.0);
    
    println!("
 _____                 _ 
|_   _|               (_)
  | | _ __   __ _ _ __ _ 
  | || '_ \\ / _` | '__| |
 _| || | | | (_| | |  | |
 \\___/_| |_|\\__,_|_|  |_|
                         
    Version : {}
    Git : https://git.foxdroidlabs.com/Foxdroid-Prod/inari
    Docker Hub : https://hub.docker.com/r/thedrewen/inari
    Discord : https://discord.gg/4vCG6vMNDJ
    Max Distance : {}
    
", env!("CARGO_PKG_VERSION"), max_distance);

    let port: u16 = env::var("API_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3030);
    
    logger::log_info(String::from("Starting web server..."));
    
    logger::log_info(format!("Started on http://0.0.0.0:{port}"));
    warp::serve(routes::all().await).run(([0, 0, 0, 0], port)).await;
}
