use crate::core::rule::{Rule, RuleValidationError};
use crate::globals;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    io::{self, Read},
    path::Path,
};

/// Top-level struct for the rules.yaml file
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RulesFile {
    pub rules: Vec<Rule>,
}

impl RulesFile {
    pub fn load() -> Result<Self, io::Error> {
        log::debug!("Loading rules from file");
        let config = globals::get_config();
        let config = config
            .lock()
            .map_err(|_| io::Error::other("Failed to lock config"))?;
        let path = Path::new(&config.rules_file);

        if !path.exists() {
            log::warn!(
                "Rules file does not exist: {}, creating new one",
                path.display()
            );
            let empty = Self::default();
            let content = serde_yaml::to_string(&empty)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            fs::write(path, content)?;
            return Ok(empty);
        }
        if !path.is_file() {
            log::error!("Rules file is not a regular file: {}", path.display());
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Rules file is not a regular file",
            ));
        }
        let content = fs::read_to_string(path)?;
        let rules: Self = serde_yaml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        log::debug!("Successfully loaded {} rules", rules.rules.len());
        Ok(rules)
    }

    pub fn load_from_ids(rule_ids: &[String]) -> Result<Self, io::Error> {
        log::debug!("Loading rules for specified IDs: {rule_ids:?}");
        let all_rules = Self::load()?;
        let mut filtered_rules = Vec::new();
        for rule_id in rule_ids {
            match all_rules.rules.iter().find(|rule| &rule.id == rule_id) {
                Some(rule) => filtered_rules.push(rule.clone()),
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Rule with id '{rule_id}' not found"),
                    ));
                }
            }
        }
        log::debug!("Filtered rules: {filtered_rules:?}");
        Ok(Self {
            rules: filtered_rules,
        })
    }

    pub fn save(&self) -> Result<(), io::Error> {
        log::debug!("Saving rules to file");
        let config = globals::get_config();
        let config = config
            .lock()
            .map_err(|_| io::Error::other("Failed to lock config"))?;
        let path = Path::new(&config.rules_file);
        let content = serde_yaml::to_string(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)?;
        log::debug!(
            "Successfully saved {} rules to {}",
            self.rules.len(),
            path.display()
        );
        Ok(())
    }

    pub fn add_rule_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        log::debug!("Adding rule(s) from file: {file_path}");

        let mut file = fs::File::open(file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        if content.trim_start().starts_with("rules:") {
            log::debug!("Detected multiple rules in file");
            self.add_multiple_rules(&content)
        } else {
            log::debug!("Detected single rule in file");
            self.add_single_rule(&content)
        }
    }

    fn add_single_rule(&mut self, yaml: &str) -> Result<(), Box<dyn Error>> {
        let new_rule: Rule = serde_yaml::from_str(yaml)?;
        log::debug!("Parsed new rule: {new_rule:?}");
        new_rule.validate()?;

        if self.rules.iter().any(|r| r.id == new_rule.id) {
            return Err(Box::new(RuleValidationError::InvalidAction(
                new_rule.id.clone(),
                0,
                "Rule ID already exists".into(),
            )));
        }

        self.rules.push(new_rule);
        self.save()?;
        Ok(())
    }

    fn add_multiple_rules(&mut self, yaml: &str) -> Result<(), Box<dyn Error>> {
        let new_rules: RulesFile = serde_yaml::from_str(yaml)?;

        for rule in new_rules.rules {
            log::debug!("Parsed rule: {rule:?}");
            rule.validate()?;

            if self.rules.iter().any(|r| r.id == rule.id) {
                return Err(Box::new(RuleValidationError::InvalidAction(
                    rule.id.clone(),
                    0,
                    "Rule ID already exists".into(),
                )));
            }

            self.rules.push(rule);
        }

        self.save()?;
        Ok(())
    }

    pub fn remove_rule(&mut self, rule_id: &str) -> Result<(), Box<dyn Error>> {
        log::debug!("Removing rule with id: {rule_id}");
        if let Some(pos) = self.rules.iter().position(|r| r.id == rule_id) {
            self.rules.remove(pos);
            self.save()?;
            log::debug!("Successfully removed rule with id: {rule_id}");
            Ok(())
        } else {
            log::error!("Rule with id '{rule_id}' not found");
            Err(Box::new(RuleValidationError::InvalidAction(
                rule_id.to_string(),
                0,
                "rule not found".into(),
            )))
        }
    }

    pub fn find_rule(&self, rule_id: &str) -> Option<Rule> {
        log::debug!("Finding rule with id: {rule_id}");
        self.rules.iter().find(|r| r.id == rule_id).cloned()
    }

    pub fn export_rule(&self, rule_id: &str, out_path: Option<&str>) -> Result<(), Box<dyn Error>> {
        log::debug!(
            "Exporting rule with id: {} to {}",
            rule_id,
            out_path.unwrap_or("stdout")
        );
        if let Some(rule) = self.rules.iter().find(|r| r.id == rule_id) {
            let content = serde_yaml::to_string(rule)?;
            log::debug!("Serialized rule: {rule:?}");
            if let Some(path) = out_path {
                fs::write(path, content)?;
                log::debug!("Successfully exported rule {rule_id} to {path}");
            } else {
                println!("{content}");
                log::debug!("Successfully exported rule {rule_id} to stdout");
            }
            Ok(())
        } else {
            log::error!("Rule with id '{rule_id}' not found");
            Err(Box::new(RuleValidationError::InvalidAction(
                rule_id.to_string(),
                0,
                "rule not found".into(),
            )))
        }
    }

    pub fn list_rules(&self) -> Vec<Rule> {
        log::debug!("Listing all rules");
        self.rules.clone()
    }

    pub fn toggle_rule(&mut self, rule_id: &str) -> Result<(), Box<dyn Error>> {
        log::debug!("Toggling rule with id: {rule_id}");
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = !rule.enabled;
            self.save()?;
            log::debug!("Successfully toggled rule with id: {rule_id}");
            Ok(())
        } else {
            log::error!("Rule with id '{rule_id}' not found");
            Err(Box::new(RuleValidationError::InvalidAction(
                rule_id.to_string(),
                0,
                "rule not found".into(),
            )))
        }
    }
}
