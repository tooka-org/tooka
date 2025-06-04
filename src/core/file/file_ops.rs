use crate::core::rules::rule::{Action, CopyAction, DeleteAction, MoveAction, RenameAction};
use crate::error::TookaError;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Result of a file operation, containing the new path and renamed file name
pub struct FileOperationResult {
    pub new_path: PathBuf,
    pub action: String,
}

/// Executes a file operation based on the provided action and file path.
/// If `dry_run` is true, it simulates the operation without making changes.
pub fn execute_action(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    log::info!(
        "Executing action '{:?}' on file: {} (dry_run: {})",
        action,
        file_path.display(),
        dry_run
    );

    match action {
        Action::Move(inner) => handle_move(file_path, inner, dry_run),
        Action::Copy(inner) => handle_copy(file_path, inner, dry_run),
        Action::Rename(inner) => handle_rename(file_path, inner, dry_run),
        Action::Delete(inner) => handle_delete(file_path, inner, dry_run),
        Action::Skip => {
            log::info!("Skipping file: {}", file_path.display());
            Ok(FileOperationResult {
                new_path: file_path.to_path_buf(),
                action: "skip".into(),
            })
        }
    }
}

fn handle_move(
    file_path: &Path,
    action: &MoveAction,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    // The Action has two attributes:
    // - to: A destination directory absolute or relative to the current working directory
    // - preserve_structure: If true, the directory structure is preserved
    // If preserve_structure is true, the file is moved to the destination directory with its original structure relative to the current working directory.
    let destination = PathBuf::from(&action.to);
    let new_path = if action.preserve_structure {
        // Preserve the directory structure
        let relative_path = file_path.strip_prefix(std::env::current_dir()?)?;
        destination.join(relative_path)
    } else {
        // Move to the destination directory directly
        destination.join(file_path.file_name().unwrap_or_default())
    };

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

fn handle_copy(
    file_path: &Path,
    action: &CopyAction,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    // The Action has two attributes:
    // - to: A destination directory absolute or relative to the current working directory
    // - preserve_structure: If true, the directory structure is preserved
    let destination = PathBuf::from(&action.to);
    let new_path = if action.preserve_structure {
        // Preserve the directory structure
        let relative_path = file_path.strip_prefix(std::env::current_dir()?)?;
        destination.join(relative_path)
    } else {
        // Copy to the destination directory directly
        destination.join(file_path.file_name().unwrap_or_default())
    };

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

fn handle_rename(
    file_path: &Path,
    action: &RenameAction,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    // The Action has one attribute:
    // - to: The new name for the file, which can be a string or a template
    let new_name = &action.to.replace(
        "{filename}",
        file_path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or(""),
    );
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
fn handle_delete(
    file_path: &Path,
    action: &DeleteAction,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    // The Action has one attribute:
    // - trash: If true, the file is moved to the trash instead of being deleted permanently
    if dry_run {
        log::debug!("Dry run: would delete file: {}", file_path.display());
    } else if action.trash {
        log::info!("Moving file to trash: {}", file_path.display());
        // Implement trash logic here, e.g., using a crate like `trash`
        trash::delete(file_path).map_err(|e| {
            TookaError::FileOperationError(format!("Failed to move file to trash: {}", e))
        })?;
    } else {
        log::info!("Deleting file permanently: {}", file_path.display());
        fs::remove_file(file_path)?;
    }

    Ok(FileOperationResult {
        new_path: PathBuf::new(),
        action: "delete".into(),
    })
}
