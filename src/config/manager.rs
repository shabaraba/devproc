use crate::models::Config;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub struct ConfigManager;

impl ConfigManager {
    pub fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("devproc")
            .join("config.toml")
    }

    pub fn load() -> Result<Config> {
        let config_path = Self::get_config_path();

        if !config_path.exists() {
            let config = Config::default();
            Self::save(&config)?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(config: &Config) -> Result<()> {
        let config_path = Self::get_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(config)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}
