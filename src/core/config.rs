use std::{env, fs, path::PathBuf};
use serde::{Deserialize, Serialize};
use crate::error::TookaError;
use crate::context::{CONFIG_FILE_NAME, CONFIG_VERSION, DEFAULT_LOGS_FOLDER, RULES_FILE_NAME};
use crate::core::environment::{get_dir_with_env, get_source_folder};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
/// Configuration structure for Tooka
pub struct Config {
    pub version: usize,
    pub source_folder: PathBuf,
    pub rules_file: PathBuf,
    pub logs_folder: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        log::debug!("Creating default configuration for Tooka");

        Self::new_with_fallbacks().unwrap_or_else(|e| {
            log::error!("Failed to construct default config: {}", e);
            // Safe fallback to minimal config
            Config {
                version: CONFIG_VERSION,
                source_folder: PathBuf::from("."),
                rules_file: PathBuf::from(RULES_FILE_NAME),
                logs_folder: PathBuf::from(DEFAULT_LOGS_FOLDER),
            }
        })
    }
}

impl Config {
    fn new_with_fallbacks() -> Result<Self, TookaError> {
        let home_dir = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                log::warn!("$HOME not set; using current directory as fallback.");
                PathBuf::from(".")
            });

        let source_folder = get_source_folder(&home_dir)?;
        let data_dir = get_dir_with_env("TOOKA_DATA_DIR", |d| d.data_dir(), &home_dir, ".local/share")?;

        Ok(Self {
            version: CONFIG_VERSION,
            source_folder,
            rules_file: data_dir.join(RULES_FILE_NAME),
            logs_folder: data_dir.join(DEFAULT_LOGS_FOLDER),
        })
    }

    pub fn load() -> Result<Self, TookaError> {
        log::debug!("Loading configuration for Tooka");
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let file = fs::File::open(&config_path)?;
            let reader = std::io::BufReader::new(file);
            let config: Config = serde_yaml::from_reader(reader)?;
            Ok(config)
        } else {
            let config = Config::new_with_fallbacks()?;
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), TookaError> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(&config_path)?;
        serde_yaml::to_writer(file, self)?;
        Ok(())
    }

    pub fn locate_config_file(&self) -> Result<PathBuf, TookaError> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            Ok(config_path)
        } else {
            Err(TookaError::ConfigError("Config file not found".into()))
        }
    }

    pub fn reset_config(&mut self) -> Result<(), TookaError> {
        *self = Config::new_with_fallbacks()?;
        self.save()
    }

    pub fn show_config(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_else(|_| "Failed to serialize config".into())
    }

    fn config_path() -> Result<PathBuf, TookaError> {
        let home_dir = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let config_dir = get_dir_with_env(
            "TOOKA_CONFIG_DIR",
            |d| d.config_dir(),
            &home_dir,
            ".config",
        )?;

        Ok(config_dir.join(CONFIG_FILE_NAME))
    }
}