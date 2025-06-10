//! Error definitions for Tooka.
//!
//! This module defines the `TookaError` enum that encompasses all possible
//! errors encountered in the application, including I/O, parsing, configuration,
//! logging, rule validation, and matching errors.
//!
//! It also defines `RuleValidationError` for detailed validation error reporting
//! related to rules processing.

use glob::PatternError;
use std::{io, path};
use thiserror::Error;

/// Represents all errors that can occur within Tooka.
///
/// Wraps errors from standard IO, parsing libraries, configuration,
/// logging, rules processing, and other application-specific failures.
#[derive(Debug, Error)]
pub enum TookaError {
    // === General ===
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV parse error: {0}")]
    Csv(#[from] csv::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("File operation error: {0}")]
    FileOperationError(String),

    // === Config ===
    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Rules file error: {0}")]
    RulesFileError(String),

    #[error("Logger error: {0}")]
    LoggerError(#[from] flexi_logger::FlexiLoggerError),

    #[error("Config already initialized")]
    ConfigAlreadyInitialized,

    #[error("Rules file already initialized")]
    RulesFileAlreadyInitialized,

    // === Matching ===
    #[error("Invalid glob pattern: {0}")]
    InvalidGlobPattern(#[from] PatternError),

    #[error("Invalid regex pattern: {0}")]
    InvalidRegexPattern(#[from] regex::Error),

    #[error("Failed prefix: {0}")]
    FailedPrefix(#[from] path::StripPrefixError),

    // === Rules ===
    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Rule validation error: {0}")]
    RuleValidationError(#[from] RuleValidationError),

    #[error("Invalid rule: {0}")]
    InvalidRule(String),

    // === Others ===
    #[error("Failed to generate PDF: {0}")]
    PdfGenerationError(String),

    #[error("Other: {0}")]
    Other(String),
}

/// Errors specific to rule validation failures.
#[derive(Debug, Error)]
pub enum RuleValidationError {
    #[error("rule id is required")]
    MissingId,

    #[error("rule {0}: name is required")]
    MissingName(String),

    #[error("rule {0}: at least one action is required")]
    NoActions(String),

    #[error("rule {0}: action {1} invalid: {2}")]
    InvalidAction(String, usize, String),

    #[error("rule {0}: invalid conditions: {1}")]
    InvalidCondition(String, String),
}
