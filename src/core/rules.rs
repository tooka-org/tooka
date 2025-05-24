use serde::{Deserialize, Serialize};
use std::{
    fs,
    io,
    path::Path,
    error::Error,
    fmt,
};
use crate::core::config;

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
    pub actions: Vec<Action>
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


pub fn load_rules() -> Result<RulesFile, io::Error> {
    let config = config::Config::load().expect("Failed to load configuration");
    let path = Path::new(&config.rules_file);
    if !path.exists() {
        return Ok(RulesFile { rules: Vec::new() });
    }
    if !path.is_file() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Rules file is not a regular file"));
    }
    if let Err(e) = fs::File::open(path) {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, format!("Rules file is not readable: {}", e)));
    }
    let content = fs::read_to_string(path)?;
    let rules: RulesFile = serde_yaml::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(rules)
}

pub fn load_rules_from_ids(rule_ids: Vec<String>) -> Result<RulesFile, io::Error> {
    let all_rules = load_rules()?;
    let mut filtered_rules = Vec::new();
    for rule_id in &rule_ids {
        match all_rules.rules.iter().find(|rule| &rule.id == rule_id) {
            Some(rule) => filtered_rules.push(rule.clone()),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Rule with id '{}' not found", rule_id),
                ));
            }
        }
    }
    Ok(RulesFile { rules: filtered_rules })
}

pub fn save_rules(rules: &RulesFile) -> Result<(), io::Error> {
    let config = config::Config::load().expect("Failed to load configuration");
    let path = Path::new(&config.rules_file);
    let content = serde_yaml::to_string(rules)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, content)?;
    Ok(())
}

pub fn add_rule_from_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let rules = load_rules()?;
    let new_rule: Rule = serde_yaml::from_reader(fs::File::open(file_path)?)?;
    
    new_rule.validate()?;

    if rules.rules.iter().any(|r| r.id == new_rule.id) {
        return Err(Box::new(RuleValidationError::InvalidAction(
            new_rule.id.clone(),
            0,
            "rule ID already exists".into(),
        )));
    }

    let mut updated_rules = rules.rules;
    updated_rules.push(new_rule);
    
    save_rules(&RulesFile { rules: updated_rules })?;
    Ok(())
}

pub fn remove_rule(rule_id: &str) -> Result<(), Box<dyn Error>> {
    let mut rules = load_rules()?;
    if let Some(pos) = rules.rules.iter().position(|r| r.id == rule_id) {
        rules.rules.remove(pos);
        save_rules(&rules)?;
        Ok(())
    } else {
        Err(Box::new(RuleValidationError::InvalidAction(
            rule_id.to_string(),
            0,
            "rule not found".into(),
        )))
    }
}

pub fn find_rule(rule_id: &str) -> Result<Option<Rule>, Box<dyn Error>> {
    let rules = load_rules()?;
    Ok(rules.rules.into_iter().find(|r| r.id == rule_id))
}

pub fn export_rule(rule_id: &str, out_path: &str) -> Result<(), Box<dyn Error>> {
    let rules = load_rules()?;
    if let Some(rule) = rules.rules.into_iter().find(|r| r.id == rule_id) {
        let content = serde_yaml::to_string(&rule)
            .map_err(|e| Box::new(io::Error::new(io::ErrorKind::InvalidData, e)))?;
        fs::write(out_path, content)?;
        Ok(())
    } else {
        Err(Box::new(RuleValidationError::InvalidAction(
            rule_id.to_string(),
            0,
            "rule not found".into(),
        )))
    }
}

pub fn list_rules() -> Result<Vec<Rule>, Box<dyn Error>> {
    let rules = load_rules()?;
    Ok(rules.rules)
}