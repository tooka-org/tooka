use crate::core::rules::rule::Action;
use crate::error::TookaError;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Result of a file operation, containing the new path and renamed file name
pub struct FileOperationResult {
    pub new_path: PathBuf,
    pub renamed: String,
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
        Action::Move { .. } => handle_move(file_path, action, dry_run),
        Action::Copy { .. } => handle_copy(file_path, action, dry_run),
        Action::Rename { .. } => handle_rename(file_path, action, dry_run),
        Action::Delete { .. } => handle_delete(file_path, action, dry_run),
        Action::Skip => {
            log::info!("Skipping file: {}", file_path.display());
            Ok(FileOperationResult {
                new_path: file_path.to_path_buf(),
                renamed: "[skipped]".into(),
            })
        }
    }
}

fn handle_move(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    // The Action has two attributes:
    // - to: A destination directory absolute or relative to the current working directory
    // - preserve_structure: If true, the directory structure is preserved
    // If preserve_structure is true, the file is moved to the destination directory with its original structure relative to the current working directory.
    if let Action::Move {
        to,
        preserve_structure,
    } = action
    {
        let destination = PathBuf::from(to);
        let new_path = if *preserve_structure {
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
            renamed: file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        })
    } else {
        Err(TookaError::FileOperationError(
            "Expected Move action".into(),
        ))
    }
}

fn handle_copy(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    // The Action has two attributes:
    // - to: A destination directory absolute or relative to the current working directory
    // - preserve_structure: If true, the directory structure is preserved
    if let Action::Copy {
        to,
        preserve_structure,
    } = action
    {
        let destination = PathBuf::from(to);
        let new_path = if *preserve_structure {
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
            renamed: file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
        })
    } else {
        Err(TookaError::FileOperationError(
            "Expected Copy action".into(),
        ))
    }
}

fn handle_rename(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    // The Action has one attribute:
    // - to: The new name for the file, which can be a string or a template
    if let Action::Rename { to } = action {
        let new_name = to.replace(
            "{filename}",
            file_path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or(""),
        );
        let new_path = file_path.with_file_name(&new_name);

        if dry_run {
            log::debug!("Dry run: would rename file to: {}", new_path.display());
        } else {
            log::info!("Renaming file to: {}", new_path.display());
            fs::rename(file_path, &new_path)?;
        }

        Ok(FileOperationResult {
            new_path,
            renamed: new_name,
        })
    } else {
        Err(TookaError::FileOperationError(
            "Expected Rename action".into(),
        ))
    }
}

/// Handles the delete action for a file, either performing the deletion or simulating it in dry run mode.
fn handle_delete(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
) -> Result<FileOperationResult, TookaError> {
    // The Action has one attribute:
    // - trash: If true, the file is moved to the trash instead of being deleted permanently
    if let Action::Delete { trash } = action {
        if dry_run {
            log::debug!("Dry run: would delete file: {}", file_path.display());
        } else if *trash {
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
            renamed: "[deleted]".into(),
        })
    } else {
        Err(TookaError::FileOperationError(
            "Expected Delete action".into(),
        ))
    }
}
