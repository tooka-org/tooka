use serde::{Deserialize, Serialize};
use crate::error::RuleValidationError;

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
    /// List of matches that this rule applies to
    pub matches: Vec<Match>,
    /// If true, all matches must match for the rule to apply
    pub match_all: bool,
    /// One or more actions to perform if the rule matches
    pub actions: Vec<Action>,
}

/// Represents a match condition for files
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    /// File extensions to match (e.g., ["jpg", "png"])
    pub extensions: Option<Vec<String>>,
    /// MIME type to match (e.g., "image/jpeg")
    pub mime_type: Option<String>,
    /// Glob pattern to match file paths (e.g., "*.jpg")
    pub pattern: Option<String>,
    /// Metadata match criteria
    pub metadata: Option<MetadataMatch>,
    /// Matches files older than this many days
    pub older_than_days: Option<u32>,
    /// Matches files larger than this size in KB
    pub size_greater_than_kb: Option<u64>,
    /// Matches files created between the specified date range
    pub created_between: Option<DateRange>,
    /// Matches files named with a specific regex pattern
    pub filename_regex: Option<String>,
    /// Matches files which are symlinks
    pub is_symlink: Option<bool>,
    /// Matches files owned by a specific user
    pub owner: Option<String>,
}

/// Represents metadata match criteria for files
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataMatch {
    /// If true, match files with EXIF date metadata
    pub exif_date: bool,
    /// List of metadata fields to match
    pub fields: Vec<MetadataField>,
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
pub struct DateRange {
    pub from: String,
    pub to: String,
}

/// Represents an action to perform when a rule matches
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Action {
    /// Move the file to a new location using a path template to create the destination path
    /// Optionally create directories if they do not exist
    Move {
        destination: String,
        #[serde(default)]
        path_template: Option<PathTemplate>,
        #[serde(default)]
        create_dirs: bool,
    },
    /// Copy the file to a new location using a path template to create the destination path
    /// Optionally create directories if they do not exist
    Copy {
        destination: String,
        #[serde(default)]
        path_template: Option<PathTemplate>,
        #[serde(default)]
        create_dirs: bool,
    },
    /// Rename the file using a template for the new name
    Rename {
        rename_template: String,
    },
    /// Compress the file to a new location using a path template to create the destination path
    /// Optionally specify the compression format and create directories if they do not exist
    Compress {
        destination: String,
        #[serde(default)]
        format: Option<String>,
        #[serde(default)]
        create_dirs: bool,
    },
    /// Delete the file
    Delete,
    /// Skip the file without any action
    Skip,
}

/// Represents a path template for creating destination paths
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathTemplate {
    pub source: String,
    pub format: String,
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
        if self.actions.is_empty() {
            log::error!(
                "Rule validation failed: no actions defined for rule {}",
                self.id
            );
            return Err(RuleValidationError::NoActions(self.id.clone()));
        }
        for (i, action) in self.actions.iter().enumerate() {
            log::debug!("Validating action {} of rule {}", i, self.id);
            match action {
                Action::Move { destination, .. }
                | Action::Copy { destination, .. }
                | Action::Compress { destination, .. } => {
                    if destination.trim().is_empty() {
                        log::error!("Rule {}: action {} is missing destination", self.id, i);
                        return Err(RuleValidationError::InvalidAction(
                            self.id.clone(),
                            i,
                            "missing destination".into(),
                        ));
                    }
                }
                Action::Rename { rename_template } => {
                    if rename_template.trim().is_empty() {
                        log::error!("Rule {}: action {} has empty rename_template", self.id, i);
                        return Err(RuleValidationError::InvalidAction(
                            self.id.clone(),
                            i,
                            "empty rename_template".into(),
                        ));
                    }
                }
                Action::Delete | Action::Skip => {
                    // No additional validation needed for Delete and Skip
                }
            }
        }
        Ok(())
    }
}
