use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub deploy: Option<DeployConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployStrategy {
    #[serde(rename = "s3_and_cloudfront")]
    S3AndCloudfront,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub strategy: DeployStrategy,
    pub bucket: String,
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    TomlError(toml::de::Error),
}

impl Config {
    pub fn load(input_dir: &Path) -> Result<Option<Self>, ConfigError> {
        let config_path = input_dir.join("config.toml");
        if !config_path.exists() {
            return Ok(None);
        }
        let config_str = fs::read_to_string(config_path).map_err(ConfigError::IoError)?;
        let config: Config = toml::from_str(&config_str).map_err(ConfigError::TomlError)?;
        Ok(Some(config))
    }
}
