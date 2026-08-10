use crate::{routes::index::index, utils::{config::Config, logger}};

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

    pub async fn run(&self) {
        let port = *self.config.api_port();
        let routes = index(self.config.app_version().clone());

        warp::serve(routes)
            .run(([0, 0, 0, 0], port)).await;
    }
}