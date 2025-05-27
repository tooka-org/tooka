use std::{fs, io, path::PathBuf};

use directories_next::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};

use crate::context::{APP_NAME, APP_ORG, APP_QUALIFIER};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
/// Configuration structure for Tooka
pub struct Config {
    /// Version of the configuration file
    pub version: usize,
    /// Source folder to sort files from
    pub source_folder: PathBuf,
    /// Path to the rules file
    pub rules_file: PathBuf,
    /// Path to the logs folder
    pub logs_folder: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        log::debug!("Creating default configuration for Tooka");

        let downloads_dir = UserDirs::new()
            .and_then(|dirs| dirs.download_dir().map(std::path::Path::to_path_buf))
            .expect("Failed to get downloads directory");

        let project_dir = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .expect("Failed to get project directories");

        let config = Self {
            version: crate::context::CONFIG_VERSION,
            source_folder: downloads_dir,
            rules_file: project_dir.data_dir().join(crate::context::RULES_FILE_NAME),
            logs_folder: project_dir
                .data_dir()
                .join(crate::context::DEFAULT_LOGS_FOLDER),
        };

        log::info!("Default configuration created: {config:?}");

        config
    }
}

impl Config {
    /// Load the configuration from the config file
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

    /// Save the configuration to the config file
    pub fn save(&self) {
        log::debug!("Saving configuration file");
        let config_path = Self::config_path();
        log::debug!("Config file path: {}", config_path.display());
        fs::create_dir_all(config_path.parent().unwrap())
            .expect("Failed to create config directory");
        let file = fs::File::create(config_path).expect("Failed to create configuration file");
        serde_yaml::to_writer(file, self)
            .map_err(io::Error::other)
            .expect("Failed to write configuration file");
        log::info!("Configuration saved successfully");
    }
    /// Get the path to the configuration file
    fn config_path() -> PathBuf {
        ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .expect("Failed to get project directories")
            .config_dir()
            .join(crate::context::CONFIG_FILE_NAME)
    }

    /// Locate the configuration file
    pub fn locate_config_file(&self) -> io::Result<PathBuf> {
        let config_path = Self::config_path();
        log::debug!("Config file path: {}", config_path.display());
        if config_path.exists() {
            log::info!("Configuration file found at: {}", config_path.display());
            Ok(config_path)
        } else {
            log::warn!("Configuration file not found at: {}", config_path.display());
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Configuration file not found",
            ))
        }
    }

    /// Resets the configuration to default values
    pub fn reset_config(&mut self) {
        log::debug!("Resetting configuration to default values");
        *self = Config::default();
        self.save();
    }

    /// Show the current configuration
    pub fn show_config(&self) -> std::string::String {
        log::debug!("Showing current configuration");
        serde_yaml::to_string(self).expect("Failed to serialize configuration to YAML")
    }
}
