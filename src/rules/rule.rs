//! Defines the core data structures representing file operation rules in Tooka.
//! Includes rule conditions, actions, and validation logic ensuring rule correctness.
//! Supports complex matching criteria such as filename patterns, metadata, size, dates, etc.

use std::{fs, path::Path};

use crate::core::error::RuleValidationError;
use serde::{Deserialize, Serialize};

/// Represents a rule for file operations, specifying when it applies and what actions to take.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Unique identifier for the rule.
    pub id: String,
    /// Human-readable name of the rule.
    pub name: String,
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Optional detailed description.
    pub description: Option<String>,
    /// Rule priority (higher is more important).
    pub priority: u32,
    /// Conditions to match files for this rule.
    pub when: Conditions,
    /// Actions to perform when conditions match.
    pub then: Vec<Action>,
}

/// Contains matching criteria to determine when a rule applies.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Conditions {
    /// If true, matches if any condition is true (logical OR); otherwise all must match (AND).
    #[serde(default)]
    pub any: Option<bool>,
    /// Regex pattern to match against the filename.
    pub filename: Option<String>,
    /// List of file extensions to match.
    #[serde(default)]
    pub extensions: Option<Vec<String>>,
    /// Glob pattern for file path matching.
    pub path: Option<String>,
    /// File size range in KB.
    pub size_kb: Option<Range>,
    /// MIME type filter.
    pub mime_type: Option<String>,
    /// Date range when the file was created.
    pub created_date: Option<DateRange>,
    /// Date range when the file was modified.
    pub modified_date: Option<DateRange>,
    /// Whether the file is a symbolic link.
    pub is_symlink: Option<bool>,
    /// Additional metadata fields for matching.
    #[serde(default)]
    pub metadata: Option<Vec<MetadataField>>,
}

/// Represents a single metadata field to match against
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MetadataField {
    /// Metadata field key (e.g., "EXIF:DateTime")
    pub key: String,
    /// Optional value to match against the field
    pub value: Option<String>,
}

/// Represents a data range for matching files
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Range {
    /// Minimum size in KB (inclusive)
    pub min: Option<u64>,
    /// Maximum size in KB (inclusive)
    pub max: Option<u64>,
}

/// Represents a date range for matching files
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DateRange {
    /// Optional start date in RFC3339 format (inclusive)
    pub from: Option<String>,
    /// Optional end date in RFC3339 format (inclusive)
    pub to: Option<String>,
}

/// Represents an action to perform when a rule matches
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    /// Move the file to a new location
    Move(MoveAction),
    /// Copy the file to a new location
    Copy(CopyAction),
    /// Rename the file to a new name
    Rename(RenameAction),
    /// Delete the file, optionally moving it to trash
    Delete(DeleteAction),
    /// Executes a CLI command or script
    Execute(ExecuteAction),
    /// Skip the file without any action
    Skip,
}

/// Represents a move action, specifying the destination path and whether to preserve structure
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MoveAction {
    /// Destination path where the file should be moved
    pub to: String,
    /// If true, preserves the directory structure relative to the source path
    #[serde(default)]
    pub preserve_structure: bool,
}

/// Represents a copy action, specifying the destination path and whether to preserve structure
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CopyAction {
    /// Destination path where the file should be copied
    pub to: String,
    /// If true, preserves the directory structure relative to the source path
    #[serde(default)]
    pub preserve_structure: bool,
}

/// Represents a rename action, specifying the new name for the file
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RenameAction {
    /// New name for the file, can include metadata placeholders
    pub to: String,
}

/// Represents a delete action, specifying whether to move the file to trash
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeleteAction {
    /// If true, moves the file to the trash instead of permanently deleting it
    #[serde(default)]
    pub trash: bool,
}

/// Represents an execute action, specifying the command to run and its arguments
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecuteAction {
    /// Command to execute, can be a shell command or script
    pub command: String,
    /// Arguments to pass to the command
    pub args: Vec<String>,
}

/// Validates the rule's fields and consistency.
///
/// Checks for required fields, duplicate metadata keys, valid size ranges,
/// valid date formats, and valid action configurations.
///
/// Returns an error if validation fails.
impl Rule {
    /// Constructs rules from a YAML file.
    /// Supports both single-rule files and multi-rule files (under `rules:` key).
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, RuleValidationError> {
        let content = fs::read_to_string(path).map_err(|e| {
            RuleValidationError::InvalidFormat(format!("Failed to read file: {}", e))
        })?;

        if content.trim_start().starts_with("rules:") {
            // Multiple rules
            let parsed: Result<RulesWrapper, _> = serde_yaml::from_str(&content);
            match parsed {
                Ok(wrapper) => Ok(wrapper.rules),
                Err(e) => Err(RuleValidationError::InvalidFormat(format!(
                    "YAML parsing failed: {e}"
                ))),
            }
        } else {
            // Single rule
            let rule: Result<Rule, _> = serde_yaml::from_str(&content);
            match rule {
                Ok(r) => Ok(vec![r]),
                Err(e) => Err(RuleValidationError::InvalidFormat(format!(
                    "YAML parsing failed: {e}"
                ))),
            }
        }
    }
    /// Validates the rule, with an optional `deep` check for logic and content consistency.
    ///
    /// If `deep` is `false`, only structural deserialization is considered valid.
    /// If `deep` is `true`, content checks like required fields, logic checks, and date format are applied.
    pub fn validate(&self, deep: bool) -> Result<(), RuleValidationError> {
        if !deep {
            return Ok(()); // Structural deserialization has already passed
        }

        if self.id.trim().is_empty() {
            return Err(RuleValidationError::MissingId);
        }

        if self.name.trim().is_empty() {
            return Err(RuleValidationError::MissingName(self.id.clone()));
        }

        if self.then.is_empty() {
            return Err(RuleValidationError::NoActions(self.id.clone()));
        }

        if let Some(metadata) = &self.when.metadata {
            let mut keys = std::collections::HashSet::new();
            for field in metadata {
                if !keys.insert(&field.key) {
                    return Err(RuleValidationError::InvalidCondition(
                        self.id.clone(),
                        format!("Duplicate metadata key '{}'", field.key),
                    ));
                }
            }
        }

        if let Some(size) = &self.when.size_kb {
            if let (Some(min), Some(max)) = (size.min, size.max) {
                if min > max {
                    return Err(RuleValidationError::InvalidCondition(
                        self.id.clone(),
                        "Invalid size_kb range: min > max".into(),
                    ));
                }
            }
        }

        for (label, date_range) in [
            ("created_date", &self.when.created_date),
            ("modified_date", &self.when.modified_date),
        ] {
            if let Some(range) = date_range {
                if let Some(from) = &range.from {
                    if let Err(e) = chrono::DateTime::parse_from_rfc3339(from) {
                        return Err(RuleValidationError::InvalidCondition(
                            self.id.clone(),
                            format!("Invalid {} 'from' date: {}", label, e),
                        ));
                    }
                }
                if let Some(to) = &range.to {
                    if let Err(e) = chrono::DateTime::parse_from_rfc3339(to) {
                        return Err(RuleValidationError::InvalidCondition(
                            self.id.clone(),
                            format!("Invalid {} 'to' date: {}", label, e),
                        ));
                    }
                }
            }
        }

        // Action validation
        for (i, action) in self.then.iter().enumerate() {
            match action {
                Action::Move(inner) => {
                    if inner.to.trim().is_empty() {
                        return Err(RuleValidationError::InvalidAction(
                            self.id.clone(),
                            i,
                            "Missing destination path".into(),
                        ));
                    }
                }
                Action::Copy(inner) => {
                    if inner.to.trim().is_empty() {
                        return Err(RuleValidationError::InvalidAction(
                            self.id.clone(),
                            i,
                            "Missing destination path".into(),
                        ));
                    }
                }
                Action::Rename(inner) => {
                    if inner.to.trim().is_empty() {
                        return Err(RuleValidationError::InvalidAction(
                            self.id.clone(),
                            i,
                            "Missing rename target path".into(),
                        ));
                    }
                }
                Action::Delete(inner) => {
                    if inner.trash && !self.when.is_symlink.unwrap_or(false) {
                        log::warn!(
                            "Rule {}: Delete action with trash enabled but file is not marked as symlink",
                            self.id
                        );
                    }
                }
                Action::Execute(inner) => {
                    if inner.command.trim().is_empty() {
                        return Err(RuleValidationError::InvalidAction(
                            self.id.clone(),
                            i,
                            "Missing command to execute".into(),
                        ));
                    }
                }
                Action::Skip => {}
            }
        }

        Ok(())
    }
}

/// Wrapper for multi-rule YAML files
#[derive(Debug, Serialize, Deserialize)]
struct RulesWrapper {
    pub rules: Vec<Rule>,
}
