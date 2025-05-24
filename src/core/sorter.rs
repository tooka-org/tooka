use std::path::PathBuf;
use rayon::prelude::*;

use crate::core::rules::{self, RulesFile};
use crate::core::config;
use crate::core::file_match;
use crate::core::file_ops;
use crate::core::logger::log_file_operation;

pub struct MatchResult {
    pub file_name: String,
    pub matched_rule_id: String,
    pub current_path: PathBuf,
    pub new_path: PathBuf,
}

pub fn sort_files(source: String, rules: String, dry_run: bool) -> Result<Vec<MatchResult>, String> {
    let source_path = if source == "<default>" {
        config::Config::load().unwrap().source_folder
    } else {
        PathBuf::from(source)
    };

    if !source_path.exists() || !source_path.is_dir() {
        return Err(format!("Source path '{}' does not exist or is not a directory.", source_path.display()));
    }

    let rules_file = if rules == "<all>" {
        let rf = rules::load_rules().map_err(|e| e.to_string())?;
        if rf.rules.is_empty() {
            return Err("No rules found to apply.".to_string());
        }
        rf
    } else {
        let rule_ids: Vec<String> = rules.split(',').map(|s| s.trim().to_string()).collect();
        if rule_ids.is_empty() {
            return Err("No valid rule IDs provided.".to_string());
        }
        rules::load_rules_from_ids(rule_ids).map_err(|e| e.to_string())?
    };

    let entries: Vec<PathBuf> = source_path.read_dir()
        .map_err(|e| e.to_string())?
        .filter_map(|res| match res {
            Ok(entry) => {
                match entry.file_type() {
                    Ok(ft) if ft.is_file() => Some(entry.path()),
                    _ => None,
                }
            }
            Err(_) => None,
        })
        .collect();

    // Use Rayon to process files in parallel
    let results: Result<Vec<_>, _> = entries.par_iter()
        .map(|file_path| sort_file(file_path, &rules_file, dry_run))
        .collect();

    // Flatten Vec<Vec<MatchResult>> into Vec<MatchResult>
    results.map(|vec_of_vec| vec_of_vec.into_iter().flatten().collect())
}

fn sort_file(file_path: &PathBuf, rules_file: &RulesFile, dry_run: bool) -> Result<Vec<MatchResult>, String> {
    let file_name = file_path.file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("Failed to get file name from path '{}'", file_path.display()))?
        .to_string();

    let mut results = Vec::new();

    for rule in &rules_file.rules {
        if file_match::match_rule_matcher(file_path, &rule.matcher) {
            // For each action in the rule, execute it and collect results
            for action in &rule.actions {
                let op_result = file_ops::execute_action(file_path, action, dry_run)?;
                if dry_run {
                    log_file_operation(&format!("DRY-{}] '{}' to '{}'", action.r#type, file_path.display(), op_result.new_path.display()));
                } else {
                    log_file_operation(&format!("[{}] '{}' to '{}'", action.r#type, file_path.display(), op_result.new_path.display()));
                }
                results.push(MatchResult {
                    file_name: file_name.clone(),
                    matched_rule_id: rule.id.clone(),
                    current_path: file_path.to_path_buf(),
                    new_path: op_result.new_path.join(&op_result.renamed),
                });
            }
            // Stops after the first matching rule
            break;
        }
    }

    Ok(results)
}
