use config::{Config as ConfigFile, ConfigError, File};
use std::path::Path;
use url::Url;

pub struct Config {
    pub api_port: u16,
    pub rpc_base_url: String,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = Path::new("config/default.toml");

        let settings = ConfigFile::builder()
            .add_source(File::from(config_path))
            // Opcional: permite sobrescrever com arquivo local
            .add_source(File::with_name("config/local").required(false))
            .build()?;

        Ok(Config {
            api_port: settings.get_int("server.port")? as u16,
            rpc_base_url: settings.get_string("rpc.base_url")?,
        })
    }

    pub fn get_rpc_folder_name(&self) -> String {
        let url = Url::parse(&self.rpc_base_url)
            .unwrap_or_else(|_| Url::parse("http://localhost").unwrap());
        format!(
            "{}{}",
            url.host_str().unwrap_or("localhost"),
            url.port().map(|p| format!("_{}", p)).unwrap_or_default()
        )
    }
}
