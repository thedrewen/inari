use chrono_tz::Europe::Brussels;

pub fn log_info(message: String) {
    let date = chrono::offset::Utc::now().with_timezone(&Brussels).format("%d/%m/%Y %H:%M:%S");
    println!("[{}] {}", date, message);
}