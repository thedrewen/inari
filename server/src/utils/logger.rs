use chrono_tz::Europe::Brussels;

use crate::utils::config::Config;

pub fn log_info(message: String) {
    let date = chrono::offset::Utc::now().with_timezone(&Brussels).format("%d/%m/%Y %H:%M:%S");
    println!("[{}] {}", date, message);
}

pub fn log_error(message: String) {
    let date = chrono::offset::Utc::now().with_timezone(&Brussels).format("%d/%m/%Y %H:%M:%S");
    eprintln!("[ERROR][{}] {}", date, message);
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
    Preferred protocol : {}
    Max Distance : {}
    Default channel : {}
    
",
        config.app_version(),
        config.protocol_mode(),
        config.max_distance(),
        config.default_channel()
    );

    if !*config.disable_telemetry() {
        log_info(format!("================================================================================================"));
        log_info(format!("Telemetry is enabled. To disable it, add the environment variable `DISABLE_TELEMETRY=true`."));
        log_info(format!("We use telemetry to find out which version of Inari is being used and how many people are using it."));
        log_info(format!("For more information about telemetry, visit https://inari.thedrewen.com/."));
        log_info(format!("================================================================================================"));
    }
    
    if *config.is_generated_api_key() {
        log_info(format!("Auto Generated Api key : {}", config.api_key()));
    }
}