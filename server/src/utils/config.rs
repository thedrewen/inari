use std::{env, fs::{self, File}};

use dotenvy::dotenv;
use getset::Getters;
use rand::{RngExt, distr::Alphanumeric};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::utils::logger;

#[derive(Getters, Clone)]
#[getset(get = "pub")]
pub struct Config {
    api_port: u16,
    webrtc_server_port: u16,
    protocol_mode: String,
    server_public_ip: String,
    api_key: String,
    default_channel: String,
    is_generated_api_key: bool,
    max_distance: f64,
    app_version: String,
    disable_telemetry: bool
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

        let default_channel = env::var("DEFAULT_CHANNEL")
            .ok()
            .unwrap_or(String::from("default"));

        let webrtc_server_port: u16 = env::var("WEBRTC_SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(44444);

        let protocol_mode: String = env::var("PROTOCOL_MODE")
            .ok()
            .unwrap_or(String::from("udp"));

        let max_distance: f64 = env::var("MAX_DISTANCE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20.0);

        let disable_telemetry: String = env::var("DISABLE_TELEMETRY")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or("false".to_string());

        let app_version = env!("CARGO_PKG_VERSION").to_string();

        Self {
            api_port,
            api_key,
            default_channel,
            server_public_ip,
            webrtc_server_port,
            protocol_mode,
            is_generated_api_key,
            max_distance,
            app_version,
            disable_telemetry: disable_telemetry == String::from("true")
        }
    }

    pub async fn send_telemetry_data(&self) {
        if !*self.disable_telemetry() {
            
            match fs::exists("./inari.id") {
                Ok(true) => {},
                Ok(false) => {
                    let unique_instance_id = Uuid::new_v4().to_string();

                    let _ = fs::write("./inari.id", unique_instance_id.clone());

                    let _resp = reqwest::Client::new().post("https://inari.thedrewen.com/api/telemetry")
                        .json(&json!({"inari_version": *self.app_version(), "unique_instance_id": unique_instance_id}))
                        .send()
                        .await;
                },
                Err(_) => {}
            }
        }
    }
}
