use crate::utils::{config::Config, logger};

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
}