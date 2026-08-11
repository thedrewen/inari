use warp::Filter;

use crate::{routes::{index::index, ws::ws}, utils::{config::Config, logger}};

pub struct App {
    config: Config
}

impl App {
    pub fn new() -> Self {
        let config = Config::new();
        
        logger::log_start(&config);
        
        Self {
            config
        }
    }

    pub fn get_bundle(&self) -> Bundle {
        let config = self.config.clone();
        Bundle {
            config
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
    pub config: Config
}