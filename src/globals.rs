use crate::core::config::Config;
use crate::core::rules_file::RulesFile;
use std::sync::{Arc, Mutex, OnceLock};

pub static CONFIG_VERSION: usize = 0;
pub static CONFIG_FILE_NAME: &str = "tooka.yaml";
pub static RULES_FILE_NAME: &str = "rules.yaml";
pub static DEFAULT_LOGS_FOLDER: &str = "logs";
pub const APP_QUALIFIER: &str = "io";
pub const APP_ORG: &str = "github.benji377";
pub const APP_NAME: &str = "tooka";

static CONFIG: OnceLock<Arc<Mutex<Config>>> = OnceLock::new();
static RULES_FILE: OnceLock<Arc<Mutex<RulesFile>>> = OnceLock::new();

pub fn get_config() -> Arc<Mutex<Config>> {
    CONFIG.get().expect("Config not initialized").clone()
}

pub fn get_rules_file() -> Arc<Mutex<RulesFile>> {
    RULES_FILE
        .get()
        .expect("Rules file not initialized")
        .clone()
}

pub fn init_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    CONFIG
        .set(Arc::new(Mutex::new(config)))
        .map_err(|_| "Config already initialized".into())
}

pub fn init_rules_file() -> Result<(), Box<dyn std::error::Error>> {
    let rules_file = RulesFile::load()?;
    RULES_FILE
        .set(Arc::new(Mutex::new(rules_file)))
        .map_err(|_| "Rules file already initialized".into())
}

pub fn set_filtered_rules_file(rule_ids: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let rules_file = RulesFile::load_from_ids(rule_ids)?;
    RULES_FILE
        .set(Arc::new(Mutex::new(rules_file)))
        .map_err(|_| "Rules file already initialized".into())
}
