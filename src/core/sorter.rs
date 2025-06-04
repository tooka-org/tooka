use crate::context;
use crate::core::common::logger::log_file_operation;
use crate::core::file::{file_match, file_ops};
use crate::error::TookaError;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::rules::rules_file::RulesFile;

/// Represents the result of a file matching operation
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MatchResult {
    pub file_name: String,
    pub action: String,
    pub matched_rule_id: String,
    pub current_path: PathBuf,
    pub new_path: PathBuf,
}

/// Sorts files in the specified source directory according to the provided rules.
pub fn sort_files(
    source: String,
    rules: &str,
    dry_run: bool,
) -> Result<Vec<MatchResult>, TookaError> {
    log::debug!(
        "Starting file sorting with source: '{source}', rules: '{rules}', dry_run: {dry_run}"
    );

    let config = context::get_locked_config()
        .map_err(|e| TookaError::ConfigError(format!("Failed to get config: {e}")))?;

    let source_path = if source == "<default>" {
        config.source_folder.clone()
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
        return Err(TookaError::ConfigError(msg));
    }

    if rules == "<all>" {
        let rf = context::get_locked_rules_file()
            .map_err(|e| TookaError::RulesFileError(format!("Failed to get rules file: {e}")))?;

        if rf.rules.is_empty() {
            log::error!("No rules found to apply.");
            return Err(TookaError::RuleNotFound(
                "No rules found to apply.".to_string(),
            ));
        }
    } else {
        let rule_ids: Vec<String> = rules.split(',').map(|s| s.trim().to_string()).collect();

        if rule_ids.is_empty() {
            log::error!("No valid rule IDs provided.");
            return Err(TookaError::RuleNotFound(
                "No valid rule IDs provided.".to_string(),
            ));
        }

        context::set_filtered_rules_file(&rule_ids).map_err(|e| {
            TookaError::RulesFileError(format!("Failed to set filtered rules: {e}"))
        })?;
    }

    let rf = context::get_locked_rules_file()
        .map_err(|e| TookaError::RulesFileError(format!("Failed to get rules file: {e}")))?;

    log::debug!("Loaded {} rules", rf.rules.len());

    // Count all files in the source directory
    let entries: Vec<PathBuf> = source_path
        .read_dir()?
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

    let total_files = entries.len();
    let progress_bar = Arc::new(
        ProgressBar::new(total_files as u64).with_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files sorted",
            )
            .unwrap(),
        ),
    );

    let results: Result<Vec<_>, TookaError> = entries
        .par_iter()
        .map(|file_path| {
            let res = sort_file(file_path, &rf, dry_run);
            progress_bar.inc(1); // update bar from each thread
            res
        })
        .collect();

    progress_bar.finish_with_message("Sorting complete");

    log::debug!(
        "File sorting completed with {} results.",
        results.as_ref().map_or(0, Vec::len)
    );

    results.map(|v| v.into_iter().flatten().collect())
}

/// Processes a single file against the rules and returns the match results.
/// If multiple rules match, picks the one with the highest priority (lowest index wins on tie).
fn sort_file(
    file_path: &Path,
    rules_file: &RulesFile,
    dry_run: bool,
) -> Result<Vec<MatchResult>, TookaError> {
    log::debug!("Processing file: '{}'", file_path.display());

    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            TookaError::FileOperationError(format!(
                "Failed to get file name from path '{}'",
                file_path.display()
            ))
        })?
        .to_string();

    // Find all matching rules with their index
    let mut matching: Vec<(usize, &_, u32)> = rules_file
        .rules
        .iter()
        .enumerate()
        .filter_map(|(idx, rule)| {
            if file_match::match_rule_matcher(file_path, &rule.when) {
                Some((idx, rule, rule.priority))
            } else {
                None
            }
        })
        .collect();

    if matching.is_empty() {
        return Ok(Vec::new());
    }

    // Pick the rule with the highest priority (largest u32), break ties by lowest index
    matching.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    let (_idx, rule, _priority) = matching[0];

    log::debug!(
        "File '{}' matched rule '{}' with priority {}",
        file_name,
        rule.id,
        rule.priority
    );

    let mut results = Vec::new();

    for action in &rule.then {
        let op_result = file_ops::execute_action(file_path, action, dry_run).map_err(|e| {
            TookaError::FileOperationError(format!("Failed to execute action: {e}"))
        })?;

        let log_prefix = if dry_run { "DRY" } else { "" };
        log_file_operation(&format!(
            "{log_prefix}[{:?}] '{}' to '{}'",
            action,
            file_path.display(),
            op_result.new_path.display()
        ));

        results.push(MatchResult {
            file_name: file_name.clone(),
            action: op_result.action.clone(),
            matched_rule_id: rule.id.clone(),
            current_path: file_path.to_path_buf(),
            new_path: op_result.new_path.clone(),
        });
    }

    Ok(results)
}
