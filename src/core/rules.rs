use serde::{Deserialize, Serialize};
use std::{
    fs,
    io,
    path::Path,
    error::Error,
    fmt,
};

/// Top-level struct for the rules.yaml file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RulesFile {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub matcher: Match,
    pub actions: Vec<Action>,
    pub flags: Flags,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    pub extensions: Option<Vec<String>>,
    pub mime_type: Option<String>,
    pub pattern: Option<String>,
    pub metadata: Option<MetadataMatch>,
    pub conditions: Option<Conditions>,
    pub any: Option<Vec<Match>>,
    pub all: Option<Vec<Match>>,
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
    #[serde(rename = "type")]
    pub r#type: String,
    pub destination: Option<String>,
    pub path_template: Option<PathTemplate>,
    pub rename_template: Option<String>,
    pub create_dirs: Option<bool>,
    pub format: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PathTemplate {
    pub source: String,
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Flags {
    pub dry_run: bool,
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
            RuleValidationError::MissingName(id) => write!(f, "rule {}: name is required", id),
            RuleValidationError::NoActions(id) => write!(f, "rule {}: at least one action is required", id),
            RuleValidationError::InvalidAction(id, idx, msg) => {
                write!(f, "rule {}: action {} invalid: {}", id, idx, msg)
            }
        }
    }
}

impl Error for RuleValidationError {}

impl Rule {
    pub fn validate(&self) -> Result<(), RuleValidationError> {
        if self.id.is_empty() {
            return Err(RuleValidationError::MissingId);
        }
        if self.name.is_empty() {
            return Err(RuleValidationError::MissingName(self.id.clone()));
        }
        if self.actions.is_empty() {
            return Err(RuleValidationError::NoActions(self.id.clone()));
        }
        for (i, action) in self.actions.iter().enumerate() {
            match action.r#type.as_str() {
                "move" | "copy" | "rename" => {
                    if action.destination.is_none() {
                        return Err(RuleValidationError::InvalidAction(
                            self.id.clone(),
                            i,
                            "missing destination".into(),
                        ));
                    }
                }
                "delete" | "skip" => {}
                other => {
                    return Err(RuleValidationError::InvalidAction(
                        self.id.clone(),
                        i,
                        format!("unknown action type '{}'", other),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Loads rules from a YAML file
pub fn load_rules<P: AsRef<Path>>(path: P) -> io::Result<RulesFile> {
    let data = fs::read(&path)?;
    let rf: RulesFile = serde_yaml::from_slice(&data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    for rule in &rf.rules {
        rule.validate().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    }
    Ok(rf)
}

/// Saves rules to a YAML file
pub fn save_rules<P: AsRef<Path>>(path: P, rf: &RulesFile) -> io::Result<()> {
    let data = serde_yaml::to_string(rf)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, data.as_bytes())?;
    Ok(())
}

/// Adds a rule ensuring unique IDs and validates it
pub fn add_rule<P: AsRef<Path>>(path: P, new_rule: Rule) -> io::Result<()> {
    let mut rf = load_rules(&path)?;
    if rf.rules.iter().any(|r| r.id == new_rule.id) {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!("rule with ID {} already exists", new_rule.id)));
    }
    new_rule.validate().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    rf.rules.push(new_rule);
    save_rules(path, &rf)
}

/// Removes a rule by ID
pub fn remove_rule<P: AsRef<Path>>(path: P, rule_id: &str) -> io::Result<()> {
    let mut rf = load_rules(&path)?;
    let original_len = rf.rules.len();
    rf.rules.retain(|r| r.id != rule_id);
    if rf.rules.len() == original_len {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("no rule found with ID {}", rule_id)));
    }
    save_rules(path, &rf)
}

/// Finds a rule by ID
pub fn find_rule<'a>(rf: &'a RulesFile, rule_id: &str) -> Option<&'a Rule> {
    rf.rules.iter().find(|r| r.id == rule_id)
}

/// Exports a single rule by writing it to a standalone YAML file
pub fn export_rule<P: AsRef<Path>>(rules_path: P, rule_id: &str, out_path: P) -> io::Result<()> {
    let rf = load_rules(rules_path)?;
    let rule = find_rule(&rf, rule_id)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "rule not found"))?;
    let export = RulesFile { rules: vec![rule.clone()] };
    let data = serde_yaml::to_string(&export)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(out_path, data.as_bytes())?;
    Ok(())
}
