use crate::error::RuleValidationError;
use serde::{Deserialize, Serialize, ser::SerializeMap};

/// Represents a rule for file operations in Tooka
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Conditions {
    /// If to match all conditions (AND) or any condition (OR), default is false (AND)
    #[serde(default)]
    pub any: Option<bool>,
    /// Filename with regex pattern to match against
    pub filename: Option<String>,
    /// List of file extensions to match against
    #[serde(default)]
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

/// Represents a date range for matching files
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Range {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

/// Represents a date range for matching files
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DateRange {
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Represents an action to perform when a rule matches
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Move(MoveAction),
    Copy(CopyAction),
    Rename(RenameAction),
    Delete(DeleteAction),
    Skip, // No payload
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MoveAction {
    pub to: String,
    #[serde(default)]
    pub preserve_structure: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CopyAction {
    pub to: String,
    #[serde(default)]
    pub preserve_structure: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RenameAction {
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeleteAction {
    #[serde(default)]
    pub trash: bool,
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use Action::*;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Move(data) => map.serialize_entry("move", data)?,
            Copy(data) => map.serialize_entry("copy", data)?,
            Rename(data) => map.serialize_entry("rename", data)?,
            Delete(data) => map.serialize_entry("delete", data)?,
            Skip => map.serialize_entry("skip", &())?, // serialize as `skip: null`
        }
        map.end()
    }
}

/// Implementation of Rule validation logic
impl Rule {
    /// Validates the rule to ensure it has all required fields and valid structure.
    pub fn validate(&self) -> Result<(), RuleValidationError> {
        if self.id.trim().is_empty() {
            log::error!("Rule validation failed: missing id");
            return Err(RuleValidationError::MissingId);
        }

        if self.name.trim().is_empty() {
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

        // Check for duplicate metadata keys
        if let Some(metadata) = &self.when.metadata {
            let mut keys = std::collections::HashSet::new();
            for field in metadata {
                if !keys.insert(&field.key) {
                    log::error!("Rule {}: duplicate metadata key '{}'", self.id, field.key);
                    return Err(RuleValidationError::InvalidCondition(
                        self.id.clone(),
                        format!("Duplicate metadata key '{}'", field.key),
                    ));
                }
            }
        }

        // Check size range consistency
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

        // Check created and modified date formats (optional)
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
                Action::Skip => {}
            }
        }

        Ok(())
    }
}
