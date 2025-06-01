use crate::context;
use crate::core::rules::rule::Rule;
use crate::error::TookaError;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Top-level struct for the `rules.yaml` file
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RulesFile {
    pub rules: Vec<Rule>,
}

/// Represents the rules file, providing methods to load, save, and manipulate rules
impl RulesFile {
    /// Load rules from the default file path
    pub fn load() -> Result<Self, TookaError> {
        log::debug!("Loading rules from file");

        let path = Self::rules_file_path()?;

        if !path.exists() {
            log::warn!(
                "Rules file does not exist: {}, creating new one",
                path.display()
            );
            let empty = Self::default();
            Self::write_to_file(&path, &empty)?;
            return Ok(empty);
        }

        if !path.is_file() {
            return Err(TookaError::ConfigError(format!(
                "Rules file is not a regular file: {}",
                path.display()
            )));
        }

        let content = fs::read_to_string(&path)?;
        let rules: Self = serde_yaml::from_str(&content)?;

        log::debug!("Successfully loaded {} rules", rules.rules.len());
        Ok(rules)
    }

    /// Load rules by a list of rule IDs
    pub fn load_from_ids(rule_ids: &[String]) -> Result<Self, TookaError> {
        log::debug!("Loading rules for specified IDs: {rule_ids:?}");
        let all_rules = Self::load()?;
        let mut filtered_rules = Vec::new();

        for rule_id in rule_ids {
            match all_rules.rules.iter().find(|rule| &rule.id == rule_id) {
                Some(rule) => filtered_rules.push(rule.clone()),
                None => {
                    return Err(TookaError::RuleNotFound(format!(
                        "Rule with id '{rule_id}' not found"
                    )));
                }
            }
        }

        log::debug!("Filtered rules: {filtered_rules:?}");
        Ok(Self {
            rules: filtered_rules,
        })
    }

    /// Save the current rules to the file
    pub fn save(&self) -> Result<(), TookaError> {
        log::debug!("Saving rules to file");
        let path = Self::rules_file_path()?;
        Self::write_to_file(&path, self)?;
        log::debug!("Saved {} rules to {}", self.rules.len(), path.display());
        Ok(())
    }

    /// Add rule(s) from a YAML file (single or multiple)
    pub fn add_rule_from_file(
        &mut self,
        file_path: &str,
        overwrite: bool,
    ) -> Result<(), TookaError> {
        log::debug!("Adding rule(s) from file: {file_path}");

        let mut content = String::new();
        fs::File::open(file_path)?.read_to_string(&mut content)?;

        if content.trim_start().starts_with("rules:") {
            self.add_multiple_rules(&content, overwrite)
        } else {
            self.add_single_rule(&content, overwrite)
        }
    }

    /// Add a single rule from a YAML string, optionally overwriting existing rules
    fn add_single_rule(&mut self, yaml: &str, overwrite: bool) -> Result<(), TookaError> {
        let rule: Rule = serde_yaml::from_str(yaml)?;
        log::debug!("Parsed new rule: {rule:?}");
        rule.validate()?;

        if let Some(pos) = self.rules.iter().position(|r| r.id == rule.id) {
            if overwrite {
                self.rules[pos] = rule;
                self.save()?;
                return Ok(());
            } else {
                return Err(TookaError::InvalidRule(format!(
                    "Rule ID '{}' already exists",
                    rule.id
                )));
            }
        }

        self.rules.push(rule);
        self.save()?;
        Ok(())
    }

    /// Add multiple rules from a YAML string, optionally overwriting existing rules
    fn add_multiple_rules(&mut self, yaml: &str, overwrite: bool) -> Result<(), TookaError> {
        let parsed: RulesFile = serde_yaml::from_str(yaml)?;

        for rule in parsed.rules {
            log::debug!("Parsed rule: {rule:?}");
            rule.validate()?;

            if let Some(pos) = self.rules.iter().position(|r| r.id == rule.id) {
                if overwrite {
                    self.rules[pos] = rule;
                } else {
                    return Err(TookaError::InvalidRule(format!(
                        "Rule ID '{}' already exists",
                        rule.id
                    )));
                }
            } else {
                self.rules.push(rule);
            }
        }

        self.save()?;
        Ok(())
    }

    /// Remove a rule by ID
    pub fn remove_rule(&mut self, rule_id: &str) -> Result<(), TookaError> {
        log::debug!("Removing rule with id: {rule_id}");

        if let Some(pos) = self.rules.iter().position(|r| r.id == rule_id) {
            self.rules.remove(pos);
            self.save()?;
            log::debug!("Successfully removed rule with id: {rule_id}");
            Ok(())
        } else {
            Err(TookaError::RuleNotFound(format!(
                "Rule with id '{}' not found",
                rule_id
            )))
        }
    }

    /// Find a rule by ID
    pub fn find_rule(&self, rule_id: &str) -> Option<Rule> {
        log::debug!("Finding rule with id: {rule_id}");
        self.rules.iter().find(|r| r.id == rule_id).cloned()
    }

    /// Export a rule by ID to a file or stdout
    pub fn export_rule(&self, rule_id: &str, out_path: Option<&str>) -> Result<(), TookaError> {
        log::debug!(
            "Exporting rule with id: {} to {}",
            rule_id,
            out_path.unwrap_or("stdout")
        );

        if let Some(rule) = self.rules.iter().find(|r| r.id == rule_id) {
            let content = serde_yaml::to_string(rule)?;
            if let Some(path) = out_path {
                fs::write(path, content)?;
                log::debug!("Exported rule {rule_id} to {path}");
            } else {
                println!("{content}");
                log::debug!("Exported rule {rule_id} to stdout");
            }
            Ok(())
        } else {
            Err(TookaError::RuleNotFound(format!(
                "Rule with id '{}' not found",
                rule_id
            )))
        }
    }

    /// Return all rules
    pub fn list_rules(&self) -> Vec<Rule> {
        log::debug!("Listing all rules");
        self.rules.clone()
    }

    /// Toggle the `enabled` flag on a rule
    pub fn toggle_rule(&mut self, rule_id: &str) -> Result<(), TookaError> {
        log::debug!("Toggling rule with id: {rule_id}");

        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = !rule.enabled;
            self.save()?;
            log::debug!("Successfully toggled rule with id: {rule_id}");
            Ok(())
        } else {
            Err(TookaError::RuleNotFound(format!(
                "Rule with id '{}' not found",
                rule_id
            )))
        }
    }

    /// Helper function to get the path to the rules file
    fn rules_file_path() -> Result<PathBuf, TookaError> {
        let config = context::get_locked_config()
            .map_err(|e| TookaError::ConfigError(format!("Failed to get config: {}", e)))?;

        Ok(Path::new(&config.rules_file).to_path_buf())
    }

    /// Helper function to write rules to a file
    fn write_to_file(path: &Path, rules: &Self) -> Result<(), TookaError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::File::create(path)?;
        serde_yaml::to_writer(file, rules)?;
        Ok(())
    }
}
