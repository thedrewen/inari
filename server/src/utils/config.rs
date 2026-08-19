use std::env;

use dotenvy::dotenv;
use getset::Getters;
use rand::{RngExt, distr::Alphanumeric};

use crate::utils::logger;

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct Config {
    api_port: u16,
    webrtc_server_port: u16,
    server_public_ip: String,
    api_key: String,
    is_generated_api_key: bool,
    max_distance: f64,
    app_version: String
}

impl Config {
    pub fn new() -> Self {
        logger::log_info(format!("Loading environment variables..."));
        dotenv().ok();

        let api_port: u16 = env::var("API_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3030);

        let mut is_generated_api_key = false;
        let api_key: String = env::var("API_KEY").ok().unwrap_or_else(|| {
            let generated_api_key = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(14)
                .map(char::from)
                .collect();
            is_generated_api_key = true;

            generated_api_key
        });

        let server_public_ip = env::var("SERVER_PUBLIC_IP")
            .ok()
            .unwrap_or(String::from("127.0.0.1"));

        let webrtc_server_port: u16 = env::var("WEBRTC_SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(44444);

        let max_distance: f64 = env::var("MAX_DISTANCE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20.0);

        let app_version = env!("CARGO_PKG_VERSION").to_string();

        Self {
            api_port,
            api_key,
            server_public_ip,
            webrtc_server_port,
            is_generated_api_key,
            max_distance,
            app_version
        }
    }


}
