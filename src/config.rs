use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use tokio::fs;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Config {
    pub require_bash_extension: Option<String>,
    pub require_lib_extension: Option<String>,

    #[serde(default)]
    pub repo: HashMap<String, crate::repo::GenericRepo>,
}

fn get_config_path() -> Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or(anyhow!("Failed to get home directory"))?;
    path.push(".remconf.toml");
    Ok(path)
}

pub async fn load_config() -> Result<Config> {
    let path = get_config_path()?;

    if !path.is_file() {
        return Ok(Config::default());
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
