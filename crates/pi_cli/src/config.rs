use anyhow::{Context, Result};
use serde::Deserialize;
use std::{path::PathBuf, sync::LazyLock};

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

pub fn application_storage_path() -> Result<PathBuf> {
    let path = std::env::home_dir()
        .context("Could not determine home directory")?
        .join(cli_shared::APPLICATION_STORAGE_DIR);
    if !path.exists() {
        std::fs::create_dir_all(&path)
            .with_context(|| format!("Failed to storage directory '{}'", path.display()))?;
    }
    Ok(path)
}

pub fn pulse_database_path() -> Result<PathBuf> {
    let db_path = application_storage_path()?.join("pulse.db");
    Ok(db_path)
}

pub fn pulse_files_dir() -> Result<PathBuf> {
    let pulse_path = application_storage_path()?.join(cli_shared::PI_CLI_PULSE_DIR);
    if !pulse_path.exists() {
        std::fs::create_dir_all(&pulse_path)
            .with_context(|| format!("Failed to create directory '{}'", pulse_path.display()))?;
    }
    Ok(pulse_path)
}
