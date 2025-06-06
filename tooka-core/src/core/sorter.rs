use super::{context, error::TookaError};
use crate::{
    common::logger::log_file_operation,
    file::{file_match, file_ops},
    rules::rules_file::RulesFile,
};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Represents the result of a file matching operation
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MatchResult {
    /// The name of the file that matched a rule
    pub file_name: String,
    /// The action taken on the file (e.g., move, copy, delete)
    pub action: String,
    /// The ID of the rule that matched the file
    pub matched_rule_id: String,
    /// The current path of the file before any operation
    pub current_path: PathBuf,
    /// The new path where the file will be moved or copied to
    pub new_path: PathBuf,
}

/// Sorts files in the specified source directory according to the provided rules.
pub fn sort_files<F>(
    files: Vec<PathBuf>,
    source_path: PathBuf,
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
            let res = sort_file(file_path, rules_file, dry_run, &source_path);
            if let Some(ref cb) = *progress {
                cb();
            }
            res
        })
        .collect();

    results.map(|v| v.into_iter().flatten().collect())
}

/// Processes a single file against the rules and returns the match results.
/// If multiple rules match, picks the one with the highest priority (lowest index wins on tie).
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
        let op_result =
            file_ops::execute_action(file_path, action, dry_run, source_path).map_err(|e| {
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

/// Recursively collects all files in the given directory.
fn collect_files_recursively(dir: &Path) -> Result<Vec<PathBuf>, TookaError> {
    if !dir.exists() || !dir.is_dir() {
        return Err(TookaError::ConfigError(format!(
            "Path '{}' does not exist or is not a directory.",
            dir.display()
        )));
    }

    let mut entries = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            match entry.file_type() {
                Ok(ft) if ft.is_file() => entries.push(path),
                Ok(ft) if ft.is_dir() => dirs.push(path),
                _ => {}
            }
        }
    }

    Ok(entries)
}

/// Prepares the sorting operation by resolving config, rules, and collecting files.
pub fn prepare_sort(
    source: &str,
    rules: &str,
) -> Result<(PathBuf, RulesFile, Vec<PathBuf>), TookaError> {
    let config = context::get_locked_config()
        .map_err(|e| TookaError::ConfigError(format!("Failed to get config: {e}")))?;

    let source_path = if source == "<default>" {
        config.source_folder.clone()
    } else {
        PathBuf::from(source)
    };

    if rules == "<all>" {
        let rf = context::get_locked_rules_file()
            .map_err(|e| TookaError::RulesFileError(format!("Failed to get rules file: {e}")))?;

        if rf.rules.is_empty() {
            return Err(TookaError::RuleNotFound(
                "No rules found to apply.".to_string(),
            ));
        }
    } else {
        let rule_ids: Vec<String> = rules.split(',').map(|s| s.trim().to_string()).collect();

        if rule_ids.is_empty() {
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

    let files = collect_files_recursively(&source_path)?;

    Ok((source_path, rf.clone(), files))
}
