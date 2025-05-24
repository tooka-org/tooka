use std::{
    fs,
    io,
    path::PathBuf,
};

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
    /// Indicates if the first run setup is complete
    pub first_run_complete: bool,
}

impl Default for Config {
    fn default() -> Self {
        let downloads_dir = UserDirs::new()
            .and_then(|dirs| dirs.download_dir().map(|p| p.to_path_buf()))
            .expect("Failed to get downloads directory");

        let project_dir = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .expect("Failed to get project directories");

        Self {
            config_version: crate::globals::CONFIG_VERSION,
            source_folder: downloads_dir,
            rules_file: project_dir.data_dir().join(crate::globals::RULES_FILE_NAME),
            logs_folder: project_dir.data_dir().join(crate::globals::DEFAULT_LOGS_FOLDER),
            first_run_complete: false,
        }
    }
}

impl Config {

    /// Load the configuration from the config file
    pub fn load() -> io::Result<Self> {
        let config_path = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .expect("Failed to get project directories")
            .config_dir()
            .join(crate::globals::CONFIG_FILE_NAME);
        if config_path.exists() {
            let file = fs::File::open(config_path)?;
            let reader = io::BufReader::new(file);
            let config: Config = serde_yaml::from_reader(reader)
                .map_err(io::Error::other)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save the configuration to the config file
    pub fn save(&self) -> io::Result<()> {
        let config_path = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
            .expect("Failed to get project directories")
            .config_dir()
            .join(crate::globals::CONFIG_FILE_NAME);
        fs::create_dir_all(config_path.parent().unwrap())?;
        let file = fs::File::create(config_path)?;
        serde_yaml::to_writer(file, self).map_err(io::Error::other)?;
        Ok(())
    }
}

/// Locate the configuration file
pub fn locate_config_file() -> io::Result<PathBuf> {
    let config_path = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .expect("Failed to get project directories")
        .config_dir()
        .join(crate::globals::CONFIG_FILE_NAME);
    if config_path.exists() {
        Ok(config_path)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Configuration file not found"))
    }
}

/// Resets the configuration to default values
pub fn reset_config() -> io::Result<()> {
    let default_config = Config::default();
    default_config.save()
}

/// Show the current configuration
pub fn show_config() -> io::Result<String> {
    let config = Config::load()?;
    let yaml = serde_yaml::to_string(&config)
        .map_err(io::Error::other)?;
    Ok(yaml)
}