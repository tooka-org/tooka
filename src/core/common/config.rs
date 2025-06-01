use super::environment::{get_dir_with_env, get_source_folder};
use crate::context::{CONFIG_FILE_NAME, CONFIG_VERSION, DEFAULT_LOGS_FOLDER, RULES_FILE_NAME};
use crate::error::TookaError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

/// Configuration structure for Tooka
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Version of the configuration file
    pub version: usize,
    /// Folder that Tooka will sort files in
    pub source_folder: PathBuf,
    /// Path to the file containing all rules
    pub rules_file: PathBuf,
    /// Folder where Tooka will store logs
    pub logs_folder: PathBuf,
}

/// Default values for the configuration
impl Default for Config {
    /// Creates a default configuration for Tooka using fallback folder paths
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

/// Implementation of the Config struct
impl Config {
    /// Creates a new configuration with fallback paths
    fn new_with_fallbacks() -> Result<Self, TookaError> {
        let home_dir = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| {
            log::warn!("$HOME not set; using current directory as fallback.");
            PathBuf::from(".")
        });

        let source_folder = get_source_folder(&home_dir)?;
        let data_dir = get_dir_with_env(
            "TOOKA_DATA_DIR",
            |d| d.data_dir(),
            &home_dir,
            ".local/share",
        )?;

        Ok(Self {
            version: CONFIG_VERSION,
            source_folder,
            rules_file: data_dir.join(RULES_FILE_NAME),
            logs_folder: data_dir.join(DEFAULT_LOGS_FOLDER),
        })
    }

    /// Attempts to load the configuration from the default path, creating it if it does not exist.
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

    /// Saves the current configuration to the default path
    pub fn save(&self) -> Result<(), TookaError> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(&config_path)?;
        serde_yaml::to_writer(file, self)?;
        Ok(())
    }

    /// Locates the configuration file in the expected directory
    pub fn locate_config_file(&self) -> Result<PathBuf, TookaError> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            Ok(config_path)
        } else {
            Err(TookaError::ConfigError("Config file not found".into()))
        }
    }

    /// Resets the configuration to default values and saves it
    pub fn reset_config(&mut self) -> Result<(), TookaError> {
        *self = Config::new_with_fallbacks()?;
        self.save()
    }
    /// Displays the current configuration in a human-readable format
    pub fn show_config(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_else(|_| "Failed to serialize config".into())
    }

    /// Returns the path to the configuration file, creating it if necessary
    fn config_path() -> Result<PathBuf, TookaError> {
        let home_dir = env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let config_dir =
            get_dir_with_env("TOOKA_CONFIG_DIR", |d| d.config_dir(), &home_dir, ".config")?;

        Ok(config_dir.join(CONFIG_FILE_NAME))
    }
}
