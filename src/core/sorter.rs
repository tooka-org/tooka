//! File sorting logic for the Tooka application.
//!
//! This module handles sorting files according to rules loaded from a rules file.
//! It supports recursively collecting files, matching files against rules, and
//! executing actions such as move, copy, or delete. Sorting operations can be
//! performed in parallel with progress callbacks and dry-run support.

use super::error::TookaError;
use crate::{
    common::logger::log_file_operation,
    file::{file_match, file_ops},
    rules::rules_file::RulesFile,
};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

/// Result of matching a file against a rule and executing an action.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MatchResult {
    /// File name matched by the rule.
    pub file_name: String,
    /// Action performed on the file (e.g., move, copy, delete).
    pub action: String,
    /// ID of the rule that matched.
    pub matched_rule_id: String,
    /// File's original path.
    pub current_path: PathBuf,
    /// Destination path after action.
    pub new_path: PathBuf,
}

/// Sorts a batch of files using optimized rules processing.
///
/// # Arguments
/// * `files` - Files to sort.
/// * `source_path` - Base directory of source files.
/// * `rules_file` - Rules file with pre-sorted rules to apply.
/// * `dry_run` - If true, actions are logged but not performed.
/// * `on_progress` - Optional callback invoked after each file processed.
///
/// # Returns
/// List of matching results for files that matched any rule.
///
/// # Errors
/// Returns `TookaError` if file operations fail.
pub fn sort_files<F>(
    files: &[PathBuf],
    source_path: &Path,
    rules_file: &RulesFile,
    dry_run: bool,
    on_progress: Option<F>,
) -> Result<Vec<MatchResult>, TookaError>
where
    F: Fn() + Send + Sync,
{
    let progress = Arc::new(on_progress.map(|f| Arc::new(f)));

    let results: Result<Vec<_>, TookaError> = files
        .par_iter()
        .map(|file_path| {
            let res = sort_file(file_path, rules_file, dry_run, source_path);
            if let Some(ref cb) = *progress {
                cb();
            }
            res
        })
        .collect();

    results.map(|v| v.into_iter().flatten().collect())
}

/// Processes a single file against rules and returns the match results.
/// Uses pre-sorted rules for better performance with early termination.
fn sort_file(
    file_path: &Path,
    rules_file: &RulesFile,
    dry_run: bool,
    source_path: &Path,
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
        })?;

    // Since rules are pre-sorted by priority, we can take the first match
    let Some(rule) = rules_file
        .rules
        .iter()
        .find(|rule| file_match::match_rule_matcher(file_path, &rule.when))
    else {
        log::debug!("No matching rules found for file '{file_name}'");
        return Ok(vec![MatchResult {
            file_name: file_name.to_string(),
            action: "skip".to_string(),
            matched_rule_id: "none".to_string(),
            current_path: file_path.to_path_buf(),
            new_path: file_path.to_path_buf(),
        }]);
    };

    log::debug!(
        "File '{}' matched rule '{}' with priority {}",
        file_name,
        rule.id,
        rule.priority
    );

    let mut results = Vec::with_capacity(rule.then.len());
    let mut current_path = file_path.to_path_buf();

    for (i, action) in rule.then.iter().enumerate() {
        let op_result = file_ops::execute_action(&current_path, action, dry_run, source_path)
            .map_err(|e| {
                TookaError::FileOperationError(format!("Failed to execute action: {e}"))
            })?;

        let log_prefix = if dry_run { "DRY" } else { "" };
        log_file_operation(&format!(
            "{log_prefix}[{action:?}] '{}' to '{}'",
            current_path.display(),
            op_result.new_path.display()
        ));

        results.push(MatchResult {
            file_name: file_name.to_string(),
            action: op_result.action.clone(),
            matched_rule_id: rule.id.clone(),
            current_path: current_path.clone(),
            new_path: op_result.new_path.clone(),
        });

        if op_result.action == "delete" {
            if i + 1 < rule.then.len() {
                log::warn!(
                    "File was deleted, skipping {} remaining action(s).",
                    rule.then.len() - (i + 1)
                );
            }
            break;
        }

        current_path.clone_from(&op_result.new_path);
    }

    Ok(results)
}

/// Recursively collects all files in the given directory using optimized traversal
pub fn collect_files(dir: &Path) -> Result<Vec<PathBuf>, TookaError> {
    if !dir.exists() || !dir.is_dir() {
        return Err(TookaError::ConfigError(format!(
            "Path '{}' does not exist or is not a directory.",
            dir.display()
        )));
    }

    let files: Result<Vec<PathBuf>, std::io::Error> = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .par_bridge()
        .filter_map(|entry| match entry {
            Ok(e) if e.file_type().is_file() => Some(Ok(e.path().to_path_buf())),
            Ok(_) => None, // Skip directories
            Err(err) => {
                log::warn!("Error reading directory entry: {err}");
                None // Skip problematic entries instead of failing
            }
        })
        .collect();

    files.map_err(|e| TookaError::FileOperationError(format!("Failed to collect files: {e}")))
}
