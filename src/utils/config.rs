use anyhow::{Ok, Result, anyhow};
use dotenvy::dotenv;
use envy::from_env;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_connection_timeout: u64,
    pub server_host: String,
    pub server_port: u32,
    pub rest_countries_api: String,
    pub exchange_rates_api: String,
}

pub fn load_config() -> Result<Config> {
    dotenv().ok();

    let config = from_env::<Config>().map_err(|e| anyhow!("Configuration error: {}", e))?;

    Ok(config)
}
