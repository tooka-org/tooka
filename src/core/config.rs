use std::{
    fs,
    io,
    path::PathBuf,
};

use directories_next::{ProjectDirs, UserDirs};
use serde::{Deserialize, Serialize};



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

        let project_dir = ProjectDirs::from("io", "github.benji377", "tooka")
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
        let config_path = ProjectDirs::from("io", "github.benji377", "tooka")
            .expect("Failed to get project directories")
            .config_dir()
            .join(crate::globals::CONFIG_FILE_NAME);
        if config_path.exists() {
            let file = fs::File::open(config_path)?;
            let reader = io::BufReader::new(file);
            let config: Config = serde_yaml::from_reader(reader)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save the configuration to the config file
    pub fn save(&self) -> io::Result<()> {
        let config_path = ProjectDirs::from("io", "github.benji377", "tooka")
            .expect("Failed to get project directories")
            .config_dir()
            .join(crate::globals::CONFIG_FILE_NAME);
        fs::create_dir_all(config_path.parent().unwrap())?;
        let file = fs::File::create(config_path)?;
        serde_yaml::to_writer(file, self).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(())
    }
}
