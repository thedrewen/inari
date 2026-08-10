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

    pub fn get_routes(&self) -> impl warp::Filter<> {
        let root = index(&self.config);

        root
    }
}