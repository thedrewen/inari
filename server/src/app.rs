use warp::Filter;

use crate::{classes::mediasoup::mediasoup_manager::{MediaSoupManager}, routes::{index::index, ws::ws}, utils::{config::Config, logger}};

pub struct App {
    config: Config,
    mediasoup_manager: MediaSoupManager
}

impl App {
    pub async fn new() -> Self {
        let config = Config::new();
        logger::log_start(&config);
        let mediasoup_manager = MediaSoupManager::new(&config).await;
        
        
        Self {
            config,
            mediasoup_manager
        }
    }

    pub fn get_bundle(&self) -> Bundle {
        let config = self.config.clone();
        let mediasoup_manager = self.mediasoup_manager.clone();
        Bundle {
            config,
            mediasoup_manager
        }
    }

    pub async fn run(&self) {
        let port = *self.config.api_port();
        let routes = index(self.config.app_version().clone())
            .or(ws(self.get_bundle()));
 

        warp::serve(routes)
            .run(([0, 0, 0, 0], port)).await;
    }
}

#[derive(Clone)]
pub struct Bundle {
    pub config: Config,
    pub mediasoup_manager: MediaSoupManager
}