use std::{env, fs, io, path::PathBuf};
use serde::{Deserialize, Serialize};
use crate::{context::{CONFIG_VERSION, DEFAULT_LOGS_FOLDER, RULES_FILE_NAME}, core::environment::{get_dir_with_env, get_source_folder}};

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

        let home_dir = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| {
            log::warn!("$HOME not set; using current directory as fallback.");
            PathBuf::from(".")
        });

        let source_folder = get_source_folder(&home_dir);
        let data_dir = get_dir_with_env("TOOKA_DATA_DIR", |d| d.data_dir(), &home_dir, ".local/share");

        let config = Self {
            version: CONFIG_VERSION,
            source_folder,
            rules_file: data_dir.join(RULES_FILE_NAME),
            logs_folder: data_dir.join(DEFAULT_LOGS_FOLDER),
        };

        log::info!("Default Tooka config: {config:?}");
        config
    }
}

impl Config {
    pub fn load() -> io::Result<Self> {
        log::debug!("Loading configuration for Tooka");
        let config_path = Self::config_path();
        log::debug!("Config file path: {}", config_path.display());

        if config_path.exists() {
            let file = fs::File::open(&config_path)?;
            let reader = io::BufReader::new(file);
            let config: Config = serde_yaml::from_reader(reader).map_err(io::Error::other)?;
            log::info!("Configuration loaded successfully: {config:?}");
            Ok(config)
        } else {
            let config = Self::default();
            config.save();
            log::info!("Configuration file not found, created default configuration: {config:?}");
            Ok(config)
        }
    }

    pub fn save(&self) {
        let config_path = Self::config_path();
        log::debug!("Saving configuration to: {}", config_path.display());
        fs::create_dir_all(config_path.parent().unwrap()).expect("Failed to create config directory");
        let file = fs::File::create(config_path).expect("Failed to create configuration file");
        serde_yaml::to_writer(file, self).map_err(io::Error::other).expect("Failed to write configuration file");
        log::info!("Configuration saved successfully");
    }

    pub fn locate_config_file(&self) -> io::Result<PathBuf> {
        let config_path = Self::config_path();
        log::debug!("Config file path: {}", config_path.display());
        if config_path.exists() {
            log::info!("Configuration file found at: {}", config_path.display());
            Ok(config_path)
        } else {
            log::warn!("Configuration file not found at: {}", config_path.display());
            Err(io::Error::new(io::ErrorKind::NotFound, "Configuration file not found"))
        }
    }

    pub fn reset_config(&mut self) {
        log::debug!("Resetting configuration to default values");
        *self = Config::default();
        self.save();
    }

    pub fn show_config(&self) -> String {
        log::debug!("Showing current configuration");
        serde_yaml::to_string(self).expect("Failed to serialize configuration to YAML")
    }

    fn config_path() -> PathBuf {
        let home_dir = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."));
        let config_dir = get_dir_with_env("TOOKA_CONFIG_DIR", |d| d.config_dir(), &home_dir, ".config");
        config_dir.join(crate::context::CONFIG_FILE_NAME)
    }
}
