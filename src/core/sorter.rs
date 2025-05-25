use rayon::prelude::*;
use std::path::PathBuf;

use crate::core::config;
use crate::core::file_match;
use crate::core::file_ops;
use crate::core::logger::log_file_operation;
use crate::core::rules::{self, RulesFile};

pub struct MatchResult {
    pub file_name: String,
    pub matched_rule_id: String,
    pub current_path: PathBuf,
    pub new_path: PathBuf,
}

pub fn sort_files(
    source: String,
    rules: String,
    dry_run: bool,
) -> Result<Vec<MatchResult>, String> {
    log::debug!(
        "Starting file sorting with source: '{}', rules: '{}', dry_run: {}",
        source,
        rules,
        dry_run
    );
    let source_path = if source == "<default>" {
        config::Config::load().unwrap().source_folder
    } else {
        PathBuf::from(source)
    };
    log::debug!("Source path resolved to: '{}'", source_path.display());

    if !source_path.exists() || !source_path.is_dir() {
        log::error!(
            "Source path '{}' does not exist or is not a directory.",
            source_path.display()
        );
        return Err(format!(
            "Source path '{}' does not exist or is not a directory.",
            source_path.display()
        ));
    }
    log::debug!("Source path '{}' is valid.", source_path.display());

    let rules_file = if rules == "<all>" {
        let rf = rules::load_rules()
            .map_err(|e| e.to_string())
            .expect("Failed to load rules");
        if rf.rules.is_empty() {
            log::error!("No rules found to apply.");
            return Err("No rules found to apply.".to_string());
        }
        rf
    } else {
        let rule_ids: Vec<String> = rules.split(',').map(|s| s.trim().to_string()).collect();
        if rule_ids.is_empty() {
            log::error!("No valid rule IDs provided.");
            return Err("No valid rule IDs provided.".to_string());
        }
        rules::load_rules_from_ids(rule_ids)
            .map_err(|e| e.to_string())
            .expect("Failed to load rules from IDs")
    };
    log::debug!("Loaded {:?} rules", rules_file.rules.len());

    let entries: Vec<PathBuf> = source_path
        .read_dir()
        .map_err(|e| e.to_string())
        .expect("Failed to read source directory")
        .filter_map(|res| match res {
            Ok(entry) => match entry.file_type() {
                Ok(ft) if ft.is_file() => Some(entry.path()),
                _ => None,
            },
            Err(_) => None,
        })
        .collect();
    log::debug!("Found {} files in source directory.", entries.len());

    // Use Rayon to process files in parallel
    let results: Result<Vec<_>, _> = entries
        .par_iter()
        .map(|file_path| sort_file(file_path, &rules_file, dry_run))
        .collect();
    log::debug!(
        "File sorting completed with {} results.",
        results.as_ref().map_or(0, |r| r.len())
    );

    // Flatten Vec<Vec<MatchResult>> into Vec<MatchResult>
    results.map(|vec_of_vec| vec_of_vec.into_iter().flatten().collect())
}

fn sort_file(
    file_path: &PathBuf,
    rules_file: &RulesFile,
    dry_run: bool,
) -> Result<Vec<MatchResult>, String> {
    log::debug!("Processing file: '{}'", file_path.display());
    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            format!(
                "Failed to get file name from path '{}'",
                file_path.display()
            )
        })?
        .to_string();

    let mut results = Vec::new();

    for rule in &rules_file.rules {
        if file_match::match_rule_matcher(file_path, &rule.r#match) {
            log::debug!("File '{}' matched rule '{}'", file_name, rule.id);
            // For each action in the rule, execute it and collect results
            for action in &rule.actions {
                let op_result = file_ops::execute_action(file_path, action, dry_run)?;
                if dry_run {
                    log_file_operation(&format!(
                        "DRY-{}] '{}' to '{}'",
                        action.r#type,
                        file_path.display(),
                        op_result.new_path.display()
                    ));
                } else {
                    log_file_operation(&format!(
                        "[{}] '{}' to '{}'",
                        action.r#type,
                        file_path.display(),
                        op_result.new_path.display()
                    ));
                }
                results.push(MatchResult {
                    file_name: file_name.clone(),
                    matched_rule_id: rule.id.clone(),
                    current_path: file_path.to_path_buf(),
                    new_path: op_result.new_path.join(&op_result.renamed),
                });
            }
            // Stops after the first matching rule
            log::debug!(
                "Stopping after first matching rule for file '{}'",
                file_name
            );
            break;
        }
    }

    Ok(results)
}
