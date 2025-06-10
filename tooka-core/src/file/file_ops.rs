use crate::{
    core::error::TookaError,
    rules::rule::{Action, CopyAction, DeleteAction, MoveAction, RenameAction},
    utils::rename_pattern::{evaluate_template, extract_metadata},
};
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
        new_path: PathBuf::new(),
        action: "delete".into(),
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
