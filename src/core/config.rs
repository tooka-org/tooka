use std::{fs, io, path::PathBuf};

use directories_next::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};

use crate::globals::{APP_NAME, APP_ORG, APP_QUALIFIER};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
/// Configuration structure for Tooka
pub struct Config {
    /// Version of the configuration file
    pub config_version: usize,
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
            .and_then(|dirs| dirs.download_dir().map(|p| p.to_path_buf()))
            .expect("Failed to get downloads directory");

        let project_dir = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .expect("Failed to get project directories");

        let config = Self {
            config_version: crate::globals::CONFIG_VERSION,
            source_folder: downloads_dir,
            rules_file: project_dir.data_dir().join(crate::globals::RULES_FILE_NAME),
            logs_folder: project_dir
                .data_dir()
                .join(crate::globals::DEFAULT_LOGS_FOLDER)
        };

        log::info!("Default configuration created: {:?}", config);

        config
    }
}

/// Implementation of configuration loading and saving for the `Config` struct.
///
/// Provides methods to load the configuration from a YAML file located in the user's
/// configuration directory, and to save the current configuration back to that file.
/// If the configuration file does not exist, a default configuration is created and saved.
///
/// # Methods
///
/// - `load()`: Loads the configuration from the config file. If the file does not exist,
///   creates a default configuration, saves it, and returns it.
/// - `save(&self)`: Saves the current configuration to the config file, creating any
///   necessary directories.
impl Config {
    /// Load the configuration from the config file
    pub fn load() -> io::Result<Self> {
        log::debug!("Loading configuration for Tooka");
        let config_path = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .expect("Failed to get project directories")
            .config_dir()
            .join(crate::globals::CONFIG_FILE_NAME);
        log::debug!("Config file path: {:?}", config_path);
        if config_path.exists() {
            let file = fs::File::open(config_path).expect("Failed to open configuration file");
            let reader = io::BufReader::new(file);
            let config: Config = serde_yaml::from_reader(reader)
                .map_err(io::Error::other)
                .expect("Failed to parse configuration file");
            log::info!("Configuration loaded successfully: {:?}", config);
            Ok(config)
        } else {
            let config = Self::default();
            config.save().expect("Failed to save default configuration");
            log::info!(
                "Configuration file not found, created default configuration: {:?}",
                config
            );
            Ok(config)
        }
    }

    /// Save the configuration to the config file
    pub fn save(&self) -> io::Result<()> {
        log::debug!("Saving configuration file");
        let config_path = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .expect("Failed to get project directories")
            .config_dir()
            .join(crate::globals::CONFIG_FILE_NAME);
        log::debug!("Config file path: {:?}", config_path);
        fs::create_dir_all(config_path.parent().unwrap())
            .expect("Failed to create config directory");
        let file = fs::File::create(config_path).expect("Failed to create configuration file");
        serde_yaml::to_writer(file, self)
            .map_err(io::Error::other)
            .expect("Failed to write configuration file");
        log::info!("Configuration saved successfully");
        Ok(())
    }
}

/// Locate the configuration file
pub fn locate_config_file() -> io::Result<PathBuf> {
    log::debug!("Locating configuration file");
    let config_path = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .expect("Failed to get project directories")
        .config_dir()
        .join(crate::globals::CONFIG_FILE_NAME);
    log::debug!("Config file path: {:?}", config_path);
    if config_path.exists() {
        log::info!("Configuration file found at: {:?}", config_path);
        Ok(config_path)
    } else {
        log::warn!("Configuration file not found at: {:?}", config_path);
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Configuration file not found",
        ))
    }
}

/// Resets the configuration to default values
pub fn reset_config() -> io::Result<()> {
    log::debug!("Resetting configuration to default values");
    let default_config = Config::default();
    default_config.save()
}

/// Show the current configuration
pub fn show_config() -> io::Result<String> {
    log::debug!("Showing current configuration");
    let config = Config::load().expect("Failed to load configuration");
    let yaml = serde_yaml::to_string(&config)
        .map_err(io::Error::other)
        .expect("Failed to serialize configuration to YAML");
    Ok(yaml)
}
