use crate::core::rules::rule::{Action, PathTemplate};
use crate::error::TookaError;
use chrono::{Datelike, Utc};
use flate2::{Compression, write::GzEncoder};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Result of a file operation, containing the new path and renamed file name
pub struct FileOperationResult {
    pub new_path: PathBuf,
    pub renamed: String,
}

/// Represents the type of file operation being performed
#[derive(Clone, Copy)]
enum Operation {
    Move,
    Copy,
    Rename,
    Compress,
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
        Action::Move { .. } => handle_with_templates(file_path, action, dry_run, Operation::Move),
        Action::Copy { .. } => handle_with_templates(file_path, action, dry_run, Operation::Copy),
        Action::Delete => handle_delete(file_path, dry_run),
        Action::Rename { .. } => {
            handle_with_templates(file_path, action, dry_run, Operation::Rename)
        }
        Action::Compress { .. } => {
            handle_with_templates(file_path, action, dry_run, Operation::Compress)
        }
        Action::Skip => {
            log::info!("Skipping file: {}", file_path.display());
            Ok(FileOperationResult {
                new_path: file_path.to_path_buf(),
                renamed: "[skipped]".into(),
            })
        }
    }
}

/// Handles file operations that involve templates for paths and renaming.
fn handle_with_templates(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
    op: Operation,
) -> Result<FileOperationResult, TookaError> {
    let (dir_path, renamed) = expand_templates(file_path, action)?;

    let target_path = match op {
        Operation::Compress => dir_path.join(format!("{renamed}.gz")),
        _ => dir_path.join(&renamed),
    };

    let create_dirs = match action {
        Action::Move { create_dirs, .. }
        | Action::Copy { create_dirs, .. }
        | Action::Compress { create_dirs, .. } => *create_dirs,
        _ => false,
    };

    if create_dirs && !dry_run {
        fs::create_dir_all(&dir_path)?;
    }

    match op {
        Operation::Move => run_fs_op(
            file_path,
            &target_path,
            dry_run,
            |src, dest| fs::rename(src, dest),
            "move",
        )?,
        Operation::Copy => run_fs_op(
            file_path,
            &target_path,
            dry_run,
            |src, dest| fs::copy(src, dest).map(|_| ()),
            "copy",
        )?,
        Operation::Rename => run_fs_op(
            file_path,
            &target_path,
            dry_run,
            |src, dest| fs::rename(src, dest),
            "rename",
        )?,
        Operation::Compress => {
            if dry_run {
                log::debug!(
                    "Dry run: would compress file {} to {}",
                    file_path.display(),
                    target_path.display()
                );
            } else {
                log::info!(
                    "Compressing file {} to {}",
                    file_path.display(),
                    target_path.display()
                );
                compress_file(file_path, &target_path)?;
            }
        }
    }

    Ok(FileOperationResult {
        new_path: target_path,
        renamed: match op {
            Operation::Compress => format!("{renamed}.gz"),
            _ => renamed,
        },
    })
}

/// Runs a file system operation (move, copy, rename) and logs the action.
fn run_fs_op<F, R>(
    src: &Path,
    dest: &Path,
    dry_run: bool,
    op: F,
    op_name: &str,
) -> Result<(), TookaError>
where
    F: Fn(&Path, &Path) -> io::Result<R>,
{
    if dry_run {
        log::debug!(
            "Dry run: would {} file from {} to {}",
            op_name,
            src.display(),
            dest.display()
        );
        Ok(())
    } else {
        log::info!(
            "{} file from {} to {}",
            capitalize(op_name),
            src.display(),
            dest.display()
        );
        op(src, dest)?;
        Ok(())
    }
}

/// Handles the delete action for a file, either performing the deletion or simulating it in dry run mode.
fn handle_delete(file_path: &Path, dry_run: bool) -> Result<FileOperationResult, TookaError> {
    if dry_run {
        log::debug!("Dry run: would delete file: {}", file_path.display());
    } else {
        log::info!("Deleting file: {}", file_path.display());
        fs::remove_file(file_path)?;
    }

    Ok(FileOperationResult {
        new_path: PathBuf::new(),
        renamed: "[deleted]".into(),
    })
}

/// Compresses a file using Gzip compression.
fn compress_file(input_path: &Path, output_path: &Path) -> Result<(), TookaError> {
    let mut input = fs::File::open(input_path)?;
    let mut output = fs::File::create(output_path)?;
    let mut encoder = GzEncoder::new(&mut output, Compression::default());

    std::io::copy(&mut input, &mut encoder)?;
    encoder.finish()?;
    Ok(())
}

/// Expands templates in the file path and returns the new path and renamed file name.
fn expand_templates(file_path: &Path, action: &Action) -> Result<(PathBuf, String), TookaError> {
    let file_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| TookaError::InvalidTemplate("Invalid file name".into()))?;

    let ext = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("dat");

    let now = Utc::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();

    let (base, path_template, rename_template): (&String, Option<&PathTemplate>, Option<&String>) =
        match action {
            Action::Move {
                destination,
                path_template,
                ..
            } => (destination, path_template.as_ref(), None),
            Action::Copy {
                destination,
                path_template,
                ..
            } => (destination, path_template.as_ref(), None),
            Action::Compress { destination, .. } => (destination, None, None),
            Action::Rename { rename_template } => {
                static EMPTY: String = String::new();
                (&EMPTY, None, Some(rename_template))
            }
            Action::Delete | Action::Skip => {
                return Err(TookaError::InvalidTemplate(
                    "No templates for Delete/Skip actions".into(),
                ));
            }
        };

    if base.is_empty() && rename_template.is_none() {
        return Err(TookaError::InvalidTemplate(
            "Missing destination or rename_template".into(),
        ));
    }

    let expanded_base = shellexpand::tilde(base).into_owned();

    let sub_path = path_template
        .map(|t| {
            t.format
                .replace("{year}", &format!("{year:04}"))
                .replace("{month}", &format!("{month:02}"))
                .replace("{day}", &format!("{day:02}"))
        })
        .unwrap_or_default();

    let full_path = Path::new(&expanded_base).join(sub_path);

    let renamed = rename_template.map_or_else(
        || format!("{file_name}.{ext}"),
        |t| {
            t.replace("{filename}", file_name)
                .replace("{year}", &format!("{year:04}"))
                .replace("{month}", &format!("{month:02}"))
                .replace("{day}", &format!("{day:02}"))
                .replace("{ext}", ext)
        },
    );

    Ok((full_path, renamed))
}

/// Helper function to capitalize the first character of a string.
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
