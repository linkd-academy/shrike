#[derive(Debug)]
pub struct AppConfig {
    pub node_version: String,
    pub log_level: String,
    pub batch_size: u64,
    pub keep_alive: bool,
    pub keep_alive_interval: u64,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            node_version: String::from("v0.106.3"),
            log_level: String::from("info"),
            batch_size: 25,
            keep_alive: false,
            keep_alive_interval: 5,
        }
    }
}
