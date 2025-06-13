//! Module handling file operations such as moving, copying, renaming, and deleting files.
//! Supports dry run mode to simulate actions without making changes to the filesystem.
//! Provides utilities for computing destination paths with optional preservation of
//! directory structure, and uses metadata extraction to support renaming templates.

use crate::{
    core::error::TookaError,
    rules::rule::{Action, CopyAction, DeleteAction, ExecuteAction, MoveAction, RenameAction},
    utils::rename_pattern::{evaluate_template, extract_metadata},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Result of a file operation, containing the new path of the file and the action performed.
pub struct FileOperationResult {
    pub new_path: PathBuf,
    pub action: String,
}

/// Executes a file operation specified by the given action on the provided file path.
/// Supports dry run mode, which simulates the operation without modifying the filesystem.
/// Handles Move, Copy, Rename, Delete, and Skip actions.
///
/// # Arguments
/// - `file_path`: The path of the file to operate on.
/// - `action`: The action to execute (move, copy, rename, delete, skip).
/// - `dry_run`: If true, simulates the operation without performing it.
/// - `source_path`: The base source directory, used when preserving directory structure.
///
/// # Returns
/// A `FileOperationResult` containing the new file path and action performed on success.
/// Returns a `TookaError` on failure.
pub fn execute_action(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
    source_path: &Path,
) -> Result<FileOperationResult, TookaError> {
    log::info!(
        "Executing action '{:?}' on file: {} (dry_run: {})",
        action,
        file_path.display(),
        dry_run
    );

    match action {
        Action::Move(inner) => handle_move(file_path, inner, dry_run, source_path),
        Action::Copy(inner) => handle_copy(file_path, inner, dry_run, source_path),
        Action::Rename(inner) => handle_rename(file_path, inner, dry_run),
        Action::Delete(inner) => handle_delete(file_path, inner, dry_run),
        Action::Execute(inner) => handle_execute(file_path, inner, dry_run),
        Action::Skip => {
            log::info!("Skipping file: {}", file_path.display());
            Ok(FileOperationResult {
                new_path: file_path.to_path_buf(),
                action: "skip".into(),
            })
        }
    }
}

pub(crate) fn handle_move(
    file_path: &Path,
    action: &MoveAction,
    dry_run: bool,
    source_path: &Path,
) -> Result<FileOperationResult, TookaError> {
    log::debug!(
        "Handling move action: {:?} for file: {}",
        action,
        file_path.display()
    );

    let new_path = compute_destination(file_path, action, source_path);

    if dry_run {
        log::debug!("Dry run: would move file to: {}", new_path.display());
    } else {
        log::info!("Moving file to: {}", new_path.display());
        fs::create_dir_all(new_path.parent().unwrap())?;
        fs::rename(file_path, &new_path)?;
    }

    Ok(FileOperationResult {
        new_path,
        action: "move".into(),
    })
}

pub(crate) fn handle_copy(
    file_path: &Path,
    action: &CopyAction,
    dry_run: bool,
    source_path: &Path,
) -> Result<FileOperationResult, TookaError> {
    log::debug!(
        "Handling copy action: {:?} for file: {}",
        action,
        file_path.display()
    );

    let new_path = compute_destination(file_path, action, source_path);

    if dry_run {
        log::debug!("Dry run: would copy file to: {}", new_path.display());
    } else {
        log::info!("Copying file to: {}", new_path.display());
        fs::create_dir_all(new_path.parent().unwrap())?;
        fs::copy(file_path, &new_path)?;
    }

    Ok(FileOperationResult {
        new_path,
        action: "copy".into(),
    })
}

pub(crate) fn handle_rename(
    file_path: &Path,
    action: &RenameAction,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    log::debug!(
        "Handling rename action: {:?} for file: {}",
        action,
        file_path.display()
    );

    let metadata = extract_metadata(file_path)?;

    let new_name = evaluate_template(&action.to, file_path, &metadata)?;
    log::debug!("New file name: {}", new_name);

    let new_path = file_path.with_file_name(new_name);

    if dry_run {
        log::debug!("Dry run: would rename file to: {}", new_path.display());
    } else {
        log::info!("Renaming file to: {}", new_path.display());
        fs::rename(file_path, &new_path)?;
    }

    Ok(FileOperationResult {
        new_path,
        action: "rename".into(),
    })
}

/// Handles the delete action for a file, either performing the deletion or simulating it in dry run mode.
pub(crate) fn handle_delete(
    file_path: &Path,
    action: &DeleteAction,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    log::debug!(
        "Handling delete action: {:?} for file: {}",
        action,
        file_path.display()
    );

    if dry_run {
        log::debug!("Dry run: would delete file: {}", file_path.display());
    } else if action.trash {
        log::info!("Moving file to trash: {}", file_path.display());

        trash::delete(file_path).map_err(|e| {
            TookaError::FileOperationError(format!("Failed to move file to trash: {}", e))
        })?;
    } else {
        log::info!("Deleting file permanently: {}", file_path.display());
        fs::remove_file(file_path)?;
    }

    Ok(FileOperationResult {
        new_path: "[deleted]".into(),
        action: "delete".into(),
    })
}

/// Handles the execute action for a file, executing a command or script specified in the action.
pub(crate) fn handle_execute(
    file_path: &Path,
    action: &ExecuteAction,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    log::debug!(
        "Handling execute action: {:?} for file: {}",
        action,
        file_path.display()
    );

    if dry_run {
        log::debug!(
            "Dry run: would execute command: {} with arguments: {}",
            action.command,
            action.args.join(" ")
        );
    } else {
        log::info!("Executing command: {}", action.command);
        let output = std::process::Command::new(&action.command)
            .args(&action.args)
            .output()
            .map_err(|e| {
                TookaError::FileOperationError(format!("Failed to execute command: {}", e))
            })?;

        if !output.status.success() {
            return Err(TookaError::FileOperationError(format!(
                "Command failed with status: {}",
                output.status
            )));
        }
    }

    Ok(FileOperationResult {
        new_path: file_path.to_path_buf(),
        action: "execute".into(),
    })
}

pub(crate) fn compute_destination<A>(file_path: &Path, action: &A, source_path: &Path) -> PathBuf
where
    A: HasToAndPreserveStructure,
{
    log::debug!("Computing destination for file: {}", file_path.display(),);
    let to = action.to();
    let preserve_structure = action.preserve_structure();

    let destination = if to.starts_with('.') {
        log::debug!("Destination is a relative path: {}", to);
        PathBuf::from(to)
    } else if to.starts_with('~') {
        log::debug!("Destination is a home directory path: {}", to);
        let home_dir = std::env::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
        let stripped = to.trim_start_matches('~').trim_start_matches('/');
        home_dir.join(stripped)
    } else {
        log::debug!("Destination is an absolute path: {}", to);
        PathBuf::from("/").join(to.trim_start_matches('/'))
    };

    if preserve_structure {
        log::debug!(
            "Preserving directory structure for file: {}",
            file_path.display()
        );
        // Preserve the directory structure using the source_path as the base
        let relative_path = file_path.strip_prefix(source_path).unwrap_or(file_path);
        destination.join(relative_path)
    } else {
        log::debug!(
            "Not preserving directory structure for file: {}",
            file_path.display()
        );
        // Move to the destination directory directly
        destination.join(file_path.file_name().unwrap_or_default())
    }
}

pub(crate) trait HasToAndPreserveStructure {
    fn to(&self) -> &str;
    fn preserve_structure(&self) -> bool;
}

impl HasToAndPreserveStructure for MoveAction {
    fn to(&self) -> &str {
        &self.to
    }
    fn preserve_structure(&self) -> bool {
        self.preserve_structure
    }
}

impl HasToAndPreserveStructure for CopyAction {
    fn to(&self) -> &str {
        &self.to
    }
    fn preserve_structure(&self) -> bool {
        self.preserve_structure
    }
}
