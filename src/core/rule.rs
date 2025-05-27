use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

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
    /// Conditions that must be met for the match to apply
    pub older_than_days: Option<u32>,
    pub size_greater_than_kb: Option<u64>,
    pub created_between: Option<DateRange>,
    pub filename_regex: Option<String>,
    pub is_symlink: Option<bool>,
    pub owner: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataMatch {
    /// If true, match files with EXIF date metadata
    pub exif_date: bool,
    /// List of metadata fields to match
    pub fields: Vec<MetadataField>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataField {
    /// Metadata field key (e.g., "EXIF:DateTime")
    pub key: String,
    /// Optional value to match against the field
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateRange {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Action {
    Move {
        destination: String,
        #[serde(default)]
        path_template: Option<PathTemplate>,
        #[serde(default)]
        create_dirs: bool,
    },
    Copy {
        destination: String,
        #[serde(default)]
        path_template: Option<PathTemplate>,
        #[serde(default)]
        create_dirs: bool,
    },
    Rename {
        rename_template: String,
    },
    Compress {
        destination: String,
        #[serde(default)]
        format: Option<String>,
        #[serde(default)]
        create_dirs: bool,
    },
    Delete,
    Skip,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathTemplate {
    pub source: String,
    pub format: String,
}

#[derive(Debug)]
pub enum RuleValidationError {
    MissingId,
    MissingName(String),
    NoActions(String),
    InvalidAction(String, usize, String),
}

impl fmt::Display for RuleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleValidationError::MissingId => write!(f, "rule id is required"),
            RuleValidationError::MissingName(id) => write!(f, "rule {id}: name is required"),
            RuleValidationError::NoActions(id) => {
                write!(f, "rule {id}: at least one action is required")
            }
            RuleValidationError::InvalidAction(id, idx, msg) => {
                write!(f, "rule {id}: action {idx} invalid: {msg}")
            }
        }
    }
}

impl Error for RuleValidationError {}

impl Rule {
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
