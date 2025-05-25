use crate::core::config;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt, fs,
    io::{self, Read},
    path::Path,
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
    #[serde(rename = "match")]
    pub r#match: Match,
    pub actions: Vec<Action>,
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
            RuleValidationError::NoActions(id) => {
                write!(f, "rule {}: at least one action is required", id)
            }
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
                        format!("unknown action type '{}'", other),
                    ));
                }
            }
        }
        Ok(())
    }
}

pub fn load_rules() -> Result<RulesFile, io::Error> {
    log::debug!("Loading rules from file");
    let config = config::Config::load().expect("Failed to load configuration");
    let path = Path::new(&config.rules_file);
    if !path.exists() {
        log::warn!(
            "Rules file does not exist: {}, creating new one",
            path.display()
        );
        let empty = RulesFile { rules: Vec::new() };
        let content = serde_yaml::to_string(&empty)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .expect("Failed to serialize empty rules");
        fs::write(path, content).expect("Failed to write empty rules file");
        return Ok(empty);
    }
    if !path.is_file() {
        log::error!("Rules file is not a regular file: {}", path.display());
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Rules file is not a regular file",
        ));
    }
    if let Err(e) = fs::File::open(path) {
        log::error!("Rules file is not readable: {}", e);
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Rules file is not readable: {}", e),
        ));
    }
    let content = fs::read_to_string(path).expect("Failed to read rules file");
    let rules: RulesFile = serde_yaml::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        .expect("Failed to parse rules file");
    log::debug!("Successfully loaded {} rules", rules.rules.len());
    Ok(rules)
}

pub fn load_rules_from_ids(rule_ids: Vec<String>) -> Result<RulesFile, io::Error> {
    log::debug!("Loading rules for specified IDs: {:?}", rule_ids);
    let all_rules = load_rules().expect("Failed to load all rules");
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
    log::debug!("Filtered rules: {:?}", filtered_rules);
    Ok(RulesFile {
        rules: filtered_rules,
    })
}

pub fn save_rules(rules: &RulesFile) -> Result<(), io::Error> {
    log::debug!("Saving rules to file");
    let config = config::Config::load().expect("Failed to load configuration");
    let path = Path::new(&config.rules_file);
    let content = serde_yaml::to_string(rules)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        .expect("Failed to serialize rules");
    fs::write(path, content).expect("Failed to write rules file");
    log::debug!(
        "Successfully saved {} rules to {}",
        rules.rules.len(),
        path.display()
    );
    Ok(())
}

pub fn add_rule_from_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    log::debug!("Adding rule(s) from file: {}", file_path);

    let mut file = fs::File::open(file_path)
        .map_err(|e| Box::new(io::Error::new(io::ErrorKind::NotFound, e)))
        .expect("Failed to open rule file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| Box::new(io::Error::new(io::ErrorKind::InvalidData, e)))
        .expect("Failed to read rule file");

    if content.trim_start().starts_with("rules:") {
        log::debug!("Detected multiple rules in file");
        add_multiple_rules(&content)
    } else {
        log::debug!("Detected single rule in file");
        add_single_rule(&content)
    }
}

fn add_single_rule(yaml: &str) -> Result<(), Box<dyn Error>> {
    let mut rules = load_rules().expect("Failed to load existing rules");
    let new_rule: Rule = serde_yaml::from_str(yaml)
        .map_err(|e| Box::new(io::Error::new(io::ErrorKind::InvalidData, e)))
        .expect("Failed to parse rule from YAML");
    log::debug!("Parsed new rule: {:?}", new_rule);
    new_rule
        .validate()
        .map_err(|e| Box::new(io::Error::new(io::ErrorKind::InvalidData, e)))
        .expect("Rule validation failed");

    if rules.rules.iter().any(|r| r.id == new_rule.id) {
        return Err(Box::new(RuleValidationError::InvalidAction(
            new_rule.id.clone(),
            0,
            "Rule ID already exists".into(),
        )));
    }

    rules.rules.push(new_rule);
    save_rules(&rules)
        .map_err(|e| Box::new(io::Error::other(e)))
        .expect("Failed to save updated rules");
    Ok(())
}

fn add_multiple_rules(yaml: &str) -> Result<(), Box<dyn Error>> {
    let mut rules = load_rules().expect("Failed to load existing rules");
    let new_rules: RulesFile = serde_yaml::from_str(yaml)
        .map_err(|e| Box::new(io::Error::new(io::ErrorKind::InvalidData, e)))
        .expect("Failed to parse rules from YAML");

    for rule in new_rules.rules {
        log::debug!("Parsed rule: {:?}", rule);
        rule.validate()
            .map_err(|e| Box::new(io::Error::new(io::ErrorKind::InvalidData, e)))
            .expect("Rule validation failed");

        if rules.rules.iter().any(|r| r.id == rule.id) {
            return Err(Box::new(RuleValidationError::InvalidAction(
                rule.id.clone(),
                0,
                "Rule ID already exists".into(),
            )));
        }

        rules.rules.push(rule);
    }

    save_rules(&rules)
        .map_err(|e| Box::new(io::Error::other(e)))
        .expect("Failed to save updated rules");
    Ok(())
}

pub fn remove_rule(rule_id: &str) -> Result<(), Box<dyn Error>> {
    log::debug!("Removing rule with id: {}", rule_id);
    let mut rules = load_rules().expect("Failed to load existing rules");
    if let Some(pos) = rules.rules.iter().position(|r| r.id == rule_id) {
        rules.rules.remove(pos);
        save_rules(&rules).expect("Failed to save updated rules after removal");
        log::debug!("Successfully removed rule with id: {}", rule_id);
        Ok(())
    } else {
        log::error!("Rule with id '{}' not found", rule_id);
        Err(Box::new(RuleValidationError::InvalidAction(
            rule_id.to_string(),
            0,
            "rule not found".into(),
        )))
    }
}

pub fn find_rule(rule_id: &str) -> Result<Option<Rule>, Box<dyn Error>> {
    log::debug!("Finding rule with id: {}", rule_id);
    let rules = load_rules().expect("Failed to load existing rules");
    Ok(rules.rules.into_iter().find(|r| r.id == rule_id))
}

pub fn export_rule(rule_id: &str, out_path: &str) -> Result<(), Box<dyn Error>> {
    log::debug!("Exporting rule with id: {} to path: {}", rule_id, out_path);
    let rules = load_rules().expect("Failed to load existing rules");
    if let Some(rule) = rules.rules.into_iter().find(|r| r.id == rule_id) {
        let content = serde_yaml::to_string(&rule)
            .map_err(|e| Box::new(io::Error::new(io::ErrorKind::InvalidData, e)))
            .expect("Failed to serialize rule");
        log::debug!("Serialized rule: {:?}", rule);
        fs::write(out_path, content).expect("Failed to write rule to file");
        log::debug!("Successfully exported rule {} to {}", rule_id, out_path);
        Ok(())
    } else {
        log::error!("Rule with id '{}' not found", rule_id);
        Err(Box::new(RuleValidationError::InvalidAction(
            rule_id.to_string(),
            0,
            "rule not found".into(),
        )))
    }
}

pub fn list_rules() -> Result<Vec<Rule>, Box<dyn Error>> {
    log::debug!("Listing all rules");
    let rules = load_rules().expect("Failed to load existing rules");
    Ok(rules.rules)
}

pub fn toggle_rule(rule_id: &str) -> Result<(), Box<dyn Error>> {
    log::debug!("Toggling rule with id: {}", rule_id);
    let mut rules = load_rules().expect("Failed to load existing rules");
    if let Some(rule) = rules.rules.iter_mut().find(|r| r.id == rule_id) {
        rule.enabled = !rule.enabled;
        save_rules(&rules).expect("Failed to save updated rules after toggling");
        log::debug!("Successfully toggled rule with id: {}", rule_id);
        Ok(())
    } else {
        log::error!("Rule with id '{}' not found", rule_id);
        Err(Box::new(RuleValidationError::InvalidAction(
            rule_id.to_string(),
            0,
            "rule not found".into(),
        )))
    }
}
