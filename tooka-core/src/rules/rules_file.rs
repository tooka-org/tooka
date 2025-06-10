//! Provides the `RulesFile` struct representing the `rules.yaml` configuration file
//! and methods to load, save, add, remove, find, export, list, and toggle rules.
//! Handles reading from and writing to disk, rule validation, and rule management
//! within Tooka's file operation rules system.

use crate::{core::context, core::error::TookaError, rules::rule::Rule};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Top-level struct for the `rules.yaml` file containing all rules.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RulesFile {
    pub rules: Vec<Rule>,
}

/// Represents the rules file, providing methods to load, save, and manipulate rules
impl RulesFile {
    /// Loads all rules from the default `rules.yaml` file path.
    /// Creates an empty file if none exists.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
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

    /// Loads rules filtered by a list of rule IDs.
    ///
    /// # Errors
    /// Returns an error if any requested rule ID is not found.
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

    /// Saves the current set of rules to the rules file on disk.
    ///
    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn save(&self) -> Result<(), TookaError> {
        log::debug!("Saving rules to file");
        let path = Self::rules_file_path()?;
        Self::write_to_file(&path, self)?;
        log::debug!("Saved {} rules to {}", self.rules.len(), path.display());
        Ok(())
    }

    /// Adds rule(s) from a YAML file path.
    /// Supports single or multiple rules depending on YAML content.
    /// Optionally overwrites existing rules with the same ID.
    ///
    /// # Errors
    /// Returns an error if the file can't be read, parsed, or validation fails.
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

    /// Removes a rule identified by its ID.
    ///
    /// # Errors
    /// Returns an error if the rule ID is not found.
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

    /// Finds a rule by its ID.
    ///
    /// Returns `Some(Rule)` if found, otherwise `None`.
    pub fn find_rule(&self, rule_id: &str) -> Option<Rule> {
        log::debug!("Finding rule with id: {rule_id}");
        self.rules.iter().find(|r| r.id == rule_id).cloned()
    }

    /// Exports a rule by ID either to a file or prints it to stdout.
    ///
    /// # Errors
    /// Returns an error if the rule ID is not found or if writing to file fails.
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

    /// Returns a clone of all rules.
    pub fn list_rules(&self) -> Vec<Rule> {
        log::debug!("Listing all rules");
        self.rules.clone()
    }

    /// Toggles the `enabled` flag of a rule identified by its ID.
    ///
    /// # Errors
    /// Returns an error if the rule ID is not found.
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
