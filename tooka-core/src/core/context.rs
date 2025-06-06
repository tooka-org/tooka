use crate::{common::config::Config, core::error::TookaError, rules::rules_file::RulesFile};
use anyhow::{Context, Result};
use std::sync::{Arc, Mutex, OnceLock};

/// Constants for Tooka application
/// These constants define the configuration version, file names, and application metadata.
pub static CONFIG_VERSION: usize = 0;
pub static CONFIG_FILE_NAME: &str = "tooka.yaml";
pub static RULES_FILE_NAME: &str = "rules.yaml";
pub static DEFAULT_LOGS_FOLDER: &str = "logs";
pub const APP_QUALIFIER: &str = "io";
pub const APP_ORG: &str = "github.benji377";
pub const APP_NAME: &str = "tooka";
pub const LOGO_VECTOR_STR: &str = include_str!("../../assets/logo.svg");

/// Global static variables to hold the configuration and rules file.
static CONFIG: OnceLock<Arc<Mutex<Config>>> = OnceLock::new();
static RULES_FILE: OnceLock<Arc<Mutex<RulesFile>>> = OnceLock::new();

/// Initializes the global configuration.
pub fn init_config() -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;
    CONFIG
        .set(Arc::new(Mutex::new(config)))
        .map_err(|_| TookaError::ConfigAlreadyInitialized.into())
}

/// Initializes the global rules file.
pub fn init_rules_file() -> Result<()> {
    let rules_file = RulesFile::load().context("Failed to load rules file")?;
    RULES_FILE
        .set(Arc::new(Mutex::new(rules_file)))
        .map_err(|_| TookaError::RulesFileAlreadyInitialized.into())
}

/// Sets the global configuration with a new instance.
pub fn set_filtered_rules_file(rule_ids: &[String]) -> Result<()> {
    let rules_file =
        RulesFile::load_from_ids(rule_ids).context("Failed to load rules from provided IDs")?;
    RULES_FILE
        .set(Arc::new(Mutex::new(rules_file)))
        .map_err(|_| TookaError::RulesFileAlreadyInitialized.into())
}

/// Helper to lock the global rules file with context-aware error handling
pub fn get_locked_rules_file() -> Result<std::sync::MutexGuard<'static, RulesFile>> {
    let rules_file = RULES_FILE
        .get()
        .ok_or_else(|| anyhow::anyhow!("Rules file not initialized"))?;
    rules_file
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to acquire lock on rules file: {}", e))
}

/// Helper to lock the global config with context-aware error handling
pub fn get_locked_config() -> Result<std::sync::MutexGuard<'static, Config>> {
    let config = CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized"))?;
    config
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to acquire lock on config: {}", e))
}
