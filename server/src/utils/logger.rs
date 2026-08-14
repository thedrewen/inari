use chrono_tz::Europe::Brussels;

use crate::utils::config::Config;

pub fn log_info(message: String) {
    let date = chrono::offset::Utc::now().with_timezone(&Brussels).format("%d/%m/%Y %H:%M:%S");
    println!("[{}] {}", date, message);
}

pub fn log_error(message: String) {
    let date = chrono::offset::Utc::now().with_timezone(&Brussels).format("%d/%m/%Y %H:%M:%S");
    println!("[ERROR][{}] {}", date, message);
}

pub fn log_start(config: &Config) {
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
    
",
        config.app_version(),
        config.max_distance()    
    );
}