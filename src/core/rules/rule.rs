use crate::error::RuleValidationError;
use serde::{Deserialize, Serialize};

/// Represents a rule for file operations in Tooka
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    /// Unique identifier for the rule
    pub id: String,
    /// Human-readable name for the rule
    pub name: String,
    /// If the rule is enabled or not
    pub enabled: bool,
    /// Optional description of the rule
    pub description: Option<String>,
    /// Priority of the rule, higher numbers indicate higher priority
    pub priority: u32,
    /// Conditions this rule applies to
    pub when: Conditions,
    /// Actions to perform if the rule matches
    pub then: Vec<Action>,
}

/// Represents the conditions under which a rule applies
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conditions {
    /// If to match all conditions (AND) or any condition (OR), default is false (AND)
    #[serde(default)]
    pub any: Option<bool>,
    /// Filename with regex pattern to match against
    pub filename: Option<String>,
    /// List of file extensions to match against
    pub extensions: Option<Vec<String>>,
    /// Pattern to match against the file path (glob pattern)
    pub path: Option<String>,
    /// String that defines file size range to match against, eg <100, >1, 10-100, etc.
    pub size_kb: Option<Range>,
    /// MIme type to match against, e.g., "image/jpeg"
    pub mime_type: Option<String>,
    /// Date range to match against, e.g., files created within this range
    pub created_date: Option<DateRange>,
    /// Date range to match against, e.g., files modified within this range
    pub modified_date: Option<DateRange>,
    /// If the file is a symlink
    pub is_symlink: Option<bool>,
    /// Additional metadata fields to match against
    pub metadata: Option<Vec<MetadataField>>,
}

/// Represents a single metadata field to match against
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataField {
    /// Metadata field key (e.g., "EXIF:DateTime")
    pub key: String,
    /// Optional value to match against the field
    pub value: Option<String>,
}

/// Represents a date range for matching files
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Range {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

/// Represents a date range for matching files
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateRange {
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Represents an action to perform when a rule matches
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Action {
    /// Move the file to a new location and optionally preserve the directory structure
    Move {
        to: String,
        #[serde(default)]
        preserve_structure: bool,
    },
    /// Copy the file to a new location and optionally preserve the directory structure
    Copy {
        to: String,
        #[serde(default)]
        preserve_structure: bool,
    },
    /// Rename the file using a template for the new name
    Rename { to: String },
    /// Delete the file
    Delete {
        #[serde(default)]
        trash: bool, // If true, move to trash instead of permanent deletion
    },
    /// Skip the file without any action
    Skip,
}

/// Implementation of Rule validation logic
impl Rule {
    /// Validates the rule to ensure it has all required fields and actions
    pub fn validate(&self) -> Result<(), RuleValidationError> {
        if self.id.is_empty() {
            log::error!("Rule validation failed: missing id");
            return Err(RuleValidationError::MissingId);
        }
        if self.name.is_empty() {
            log::error!("Rule validation failed: missing name for rule {}", self.id);
            return Err(RuleValidationError::MissingName(self.id.clone()));
        }
        if self.then.is_empty() {
            log::error!(
                "Rule validation failed: no actions defined for rule {}",
                self.id
            );
            return Err(RuleValidationError::NoActions(self.id.clone()));
        }

        for (i, action) in self.then.iter().enumerate() {
            log::debug!("Validating action {} of rule {}", i, self.id);
            match action {
                Action::Move { to, .. } | Action::Copy { to, .. } | Action::Rename { to } => {
                    if to.trim().is_empty() {
                        log::error!("Rule {}: action {} is missing destination", self.id, i);
                        return Err(RuleValidationError::InvalidAction(
                            self.id.clone(),
                            i,
                            "missing destination".into(),
                        ));
                    }
                }
                Action::Delete { trash } => {
                    // No additional validation needed for Delete
                    if *trash && !self.when.is_symlink.unwrap_or(false) {
                        log::warn!(
                            "Rule {}: Delete action with trash enabled but not a symlink",
                            self.id
                        );
                    }
                }
                Action::Skip => {
                    // No additional validation needed for Skip
                }
            }
        }
        Ok(())
    }
}
