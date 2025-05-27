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
    /// List of matches that this rule applies to
    pub matches: Vec<Match>,
    /// If true, all matches must match for the rule to apply
    pub match_all: bool,
    /// One or more actions to perform if the rule matches
    pub actions: Vec<Action>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    pub extensions: Option<Vec<String>>,
    pub mime_type: Option<String>,
    pub pattern: Option<String>,
    pub metadata: Option<MetadataMatch>,
    pub conditions: Option<Conditions>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataMatch {
    pub exif_date: bool,
    pub fields: Vec<MetadataField>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetadataField {
    pub key: String,
    pub value: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conditions {
    pub older_than_days: Option<u32>,
    pub size_greater_than_kb: Option<u64>,
    pub created_between: Option<DateRange>,
    pub filename_regex: Option<String>,
    pub is_symlink: Option<bool>,
    pub owner: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateRange {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    /// Action type: move, copy, rename, compress, delete, skip
    #[serde(rename = "type")]
    pub r#type: String,
    /// Destination path for move/copy/compress actions
    pub destination: Option<String>,
    /// Optional path template for move/copy actions
    pub path_template: Option<PathTemplate>,
    /// Optional rename template for rename actions
    pub rename_template: Option<String>,
    /// If true, create directories if they do not exist
    pub create_dirs: Option<bool>,
    /// Compression format (e.g., zip, tar.gz) for compress actions
    pub format: Option<String>,
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
            match action.r#type.as_str() {
                "move" | "copy" | "rename" | "compress" => {
                    if action.destination.is_none() {
                        log::error!("Rule {}: action {} is missing destination", self.id, i);
                        return Err(RuleValidationError::InvalidAction(
                            self.id.clone(),
                            i,
                            "missing destination".into(),
                        ));
                    }
                }
                "delete" | "skip" => {}
                other => {
                    log::error!(
                        "Rule {}: action {} has unknown type '{}'",
                        self.id,
                        i,
                        other
                    );
                    return Err(RuleValidationError::InvalidAction(
                        self.id.clone(),
                        i,
                        format!("unknown action type '{other}'"),
                    ));
                }
            }
        }
        Ok(())
    }
}
