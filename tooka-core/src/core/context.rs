//! Core application context for Tooka.
//!
//! This module defines global constants and manages the global state for
//! configuration and rules file, providing thread-safe access via `Mutex`
//! wrapped in `Arc` and initialized once with `OnceLock`.
//!
//! It includes functions to initialize, set, and safely access these globals.

use crate::{common::config::Config, core::error::TookaError, rules::rules_file::RulesFile};
use anyhow::{Context, Result};
use std::sync::{Arc, Mutex, OnceLock};

/// Configuration version number.
pub static CONFIG_VERSION: usize = 0;
/// Default config file name.
pub static CONFIG_FILE_NAME: &str = "tooka.yaml";
/// Default rules file name.
pub static RULES_FILE_NAME: &str = "rules.yaml";
/// Default folder for logs.
pub static DEFAULT_LOGS_FOLDER: &str = "logs";

/// Application qualifier (used for config directory identification).
pub const APP_QUALIFIER: &str = "io";
/// Application organization identifier.
pub const APP_ORG: &str = "github.benji377";
/// Application name.
pub const APP_NAME: &str = "tooka";

/// Global, thread-safe storage of the configuration.
static CONFIG: OnceLock<Arc<Mutex<Config>>> = OnceLock::new();
/// Global, thread-safe storage of the rules file.
static RULES_FILE: OnceLock<Arc<Mutex<RulesFile>>> = OnceLock::new();

/// Loads and initializes the global configuration.
///
/// # Errors
/// Returns an error if loading the configuration or initialization fails.
pub fn init_config() -> Result<()> {
    let config = Config::load().context("Failed to load configuration")?;
    CONFIG
        .set(Arc::new(Mutex::new(config)))
        .map_err(|_| TookaError::ConfigAlreadyInitialized.into())
}

/// Loads and initializes the global rules file.
///
/// # Errors
/// Returns an error if loading the rules file or initialization fails.
pub fn init_rules_file() -> Result<()> {
    let rules_file = RulesFile::load().context("Failed to load rules file")?;
    RULES_FILE
        .set(Arc::new(Mutex::new(rules_file)))
        .map_err(|_| TookaError::RulesFileAlreadyInitialized.into())
}

/// Sets a filtered rules file using a list of rule IDs.
///
/// # Errors
/// Returns an error if loading the filtered rules file or initialization fails.
pub fn set_filtered_rules_file(rule_ids: &[String]) -> Result<()> {
    let rules_file =
        RulesFile::load_from_ids(rule_ids).context("Failed to load rules from provided IDs")?;
    RULES_FILE
        .set(Arc::new(Mutex::new(rules_file)))
        .map_err(|_| TookaError::RulesFileAlreadyInitialized.into())
}

/// Locks and returns a reference to the global rules file.
///
/// # Errors
/// Returns an error if the rules file is not initialized or lock acquisition fails.
pub fn get_locked_rules_file() -> Result<std::sync::MutexGuard<'static, RulesFile>> {
    let rules_file = RULES_FILE
        .get()
        .ok_or_else(|| anyhow::anyhow!("Rules file not initialized"))?;
    rules_file
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to acquire lock on rules file: {}", e))
}

/// Locks and returns a reference to the global configuration.
///
/// # Errors
/// Returns an error if the config is not initialized or lock acquisition fails.
pub fn get_locked_config() -> Result<std::sync::MutexGuard<'static, Config>> {
    let config = CONFIG
        .get()
        .ok_or_else(|| anyhow::anyhow!("Config not initialized"))?;
    config
        .lock()
        .map_err(|e| anyhow::anyhow!("Failed to acquire lock on config: {}", e))
}
