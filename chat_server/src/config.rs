use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    // pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // read from ./app.yml or /etc/config/app.yml  or env

        let ret: Result<AppConfig, _> = match (
            fs::File::open("app.yml"),
            fs::File::open("/etc/config/app.yml"),
            std::env::var("CHAT_CONFIG"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (Err(_), Ok(reader), _) => serde_yaml::from_reader(reader),
            (Err(_), Err(_), Ok(reader)) => serde_yaml::from_reader(fs::File::open(reader)?),
            _ => bail!("Config file not found"),
        };

        Ok(ret?)
    }
}
