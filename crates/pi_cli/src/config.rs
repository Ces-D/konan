use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize)]
pub struct KonanIotConfig {
    pub endpoint: String,
    pub port: u16,
    pub client_id: String,
    pub cert_path: PathBuf,
    pub private_key_path: PathBuf,
    pub root_trust_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub connect: KonanIotConfig,
}

pub static CONFIG: LazyLock<Result<Config>> = LazyLock::new(Config::load);

impl Config {
    pub fn get() -> Result<&'static Config> {
        CONFIG.as_ref().map_err(|e| anyhow::anyhow!("{e}"))
    }

    fn load() -> Result<Self> {
        let config_path = std::env::home_dir()
            .context("Could not determine home directory")?
            .join(".config/konan/config.toml");
        let raw = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config at '{}'", config_path.display()))?;
        toml::from_str(&raw).map_err(|e| {
            anyhow::anyhow!("Failed to parse config '{}': {}", config_path.display(), e)
        })
    }
}
