use rayon::prelude::*;
use std::path::{Path, PathBuf};

use crate::context;
use crate::core::{file_match, file_ops, logger::log_file_operation, rules_file::RulesFile};

pub struct MatchResult {
    pub file_name: String,
    pub matched_rule_id: String,
    pub current_path: PathBuf,
    pub new_path: PathBuf,
}

pub fn sort_files(source: String, rules: &str, dry_run: bool) -> Result<Vec<MatchResult>, String> {
    log::debug!(
        "Starting file sorting with source: '{source}', rules: '{rules}', dry_run: {dry_run}"
    );

    let config = context::get_config();
    let source_path = if source == "<default>" {
        config
            .lock()
            .map_err(|_| "Failed to lock config")?
            .source_folder
            .clone()
    } else {
        PathBuf::from(source)
    };

    log::debug!("Source path resolved to: '{}'", source_path.display());

    if !source_path.exists() || !source_path.is_dir() {
        let msg = format!(
            "Source path '{}' does not exist or is not a directory.",
            source_path.display()
        );
        log::error!("{msg}");
        return Err(msg);
    }

    if rules == "<all>" {
        let rf = context::get_rules_file();
        let rf_guard = rf.lock().map_err(|_| "Failed to lock rules file")?;

        if rf_guard.rules.is_empty() {
            log::error!("No rules found to apply.");
            return Err("No rules found to apply.".to_string());
        }

        drop(rf_guard); // Release the lock
    } else {
        let rule_ids: Vec<String> = rules.split(',').map(|s| s.trim().to_string()).collect();

        if rule_ids.is_empty() {
            log::error!("No valid rule IDs provided.");
            return Err("No valid rule IDs provided.".to_string());
        }

        context::set_filtered_rules_file(&rule_ids)
            .map_err(|e| format!("Failed to set filtered rules: {e}"))?;
    }

    let rf = context::get_rules_file();
    let rf = rf.lock().map_err(|_| "Failed to lock rules file")?;

    log::debug!("Loaded {} rules", rf.rules.len());

    let entries: Vec<PathBuf> = source_path
        .read_dir()
        .map_err(|e| e.to_string())?
        .filter_map(|res| match res {
            Ok(entry) => entry
                .file_type()
                .ok()
                .filter(|ft| ft.is_file())
                .map(|_| entry.path()),
            Err(_) => None,
        })
        .collect();

    log::debug!("Found {} files in source directory.", entries.len());

    let results: Result<Vec<_>, _> = entries
        .par_iter()
        .map(|file_path| sort_file(file_path, &rf, dry_run))
        .collect();

    log::debug!(
        "File sorting completed with {} results.",
        results.as_ref().map_or(0, Vec::len)
    );

    results.map(|v| v.into_iter().flatten().collect())
}

fn sort_file(
    file_path: &Path,
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
        log::debug!(
            "Checking if file '{}' matches rule '{}'",
            file_name,
            rule.id
        );

        let is_match = if rule.match_all {
            log::debug!("Rule '{}' requires all matchers to match", rule.id);
            rule.matches
                .iter()
                .all(|matcher| file_match::match_rule_matcher(file_path, matcher))
        } else {
            log::debug!("Rule '{}' requires any matcher to match", rule.id);
            rule.matches
                .iter()
                .any(|matcher| file_match::match_rule_matcher(file_path, matcher))
        };

        if is_match {
            log::debug!("File '{}' matched rule '{}'", file_name, rule.id);

            for action in &rule.actions {
                let op_result = file_ops::execute_action(file_path, action, dry_run)
                    .map_err(|e| format!("Failed to execute action: {e}"))?;

                let log_prefix = if dry_run { "DRY" } else { "" };
                log_file_operation(&format!(
                    "{log_prefix}[{:?}] '{}' to '{}'",
                    action,
                    file_path.display(),
                    op_result.new_path.display()
                ));

                results.push(MatchResult {
                    file_name: file_name.clone(),
                    matched_rule_id: rule.id.clone(),
                    current_path: file_path.to_path_buf(),
                    new_path: op_result.new_path.join(&op_result.renamed),
                });
            }

            log::debug!("Stopping after first matching rule for file '{file_name}'");
            break;
        }
    }

    Ok(results)
}
