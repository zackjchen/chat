use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    // pub host: String,
    pub port: u16,
    pub db_url: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthConfig {
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let ret: Result<AppConfig, _> = match (
            fs::File::open("notify.yml"),
            fs::File::open("/etc/config/notify.yml"),
            std::env::var("NOTIFY_CONFIG"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (Err(_), Ok(reader), _) => serde_yaml::from_reader(reader),
            (Err(_), Err(_), Ok(reader)) => serde_yaml::from_reader(fs::File::open(reader)?),
            _ => bail!("Config file not found"),
        };

        Ok(ret?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_config() -> anyhow::Result<()> {
        let config = AppConfig::load()?;
        println!("{:?}", config);
        Ok(())
    }
}
