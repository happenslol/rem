use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use tokio::fs;
use toml::toml;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub require_bash_extension: bool,
    pub require_lib_extension: bool,

    #[serde(default)]
    pub repo: HashMap<String, crate::repo::GenericRepo>,
}

fn get_default_config() -> Config {
    let config = toml! {
        require_bash_extension = false
        require_lib_extension = false
    };

    config.try_into().expect("Failed to get default config")
}

fn get_config_path() -> Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or(anyhow!("Failed to get home directory"))?;
    path.push(".remconf.toml");
    Ok(path)
}

pub async fn load_config() -> Result<Config> {
    let path = get_config_path()?;

    if !path.is_file() {
        return Ok(get_default_config());
    }

    let config_str = fs::read_to_string(&path).await?;
    Ok(toml::from_str(&config_str)?)
}

pub async fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path()?;
    let config_str = toml::to_string(config)?;
    fs::write(path, &config_str).await?;

    Ok(())
}
