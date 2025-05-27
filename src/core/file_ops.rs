use crate::core::rule::{Action, PathTemplate};
use chrono::{Datelike, Utc};
use flate2::{Compression, write::GzEncoder};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub struct FileOperationResult {
    pub new_path: PathBuf,
    pub renamed: String,
}

pub fn execute_action(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
) -> Result<FileOperationResult, String> {
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

#[derive(Clone, Copy)]
enum Operation {
    Move,
    Copy,
    Rename,
    Compress,
}

fn handle_with_templates(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
    op: Operation,
) -> Result<FileOperationResult, String> {
    let (dir_path, renamed) = expand_templates(file_path, action)?;

    // Build target path differently for compress operation
    let target_path = match op {
        Operation::Compress => dir_path.join(format!("{renamed}.gz")),
        _ => dir_path.join(&renamed),
    };

    // Extract create_dirs flag from the action variants that have it
    let create_dirs = match action {
        Action::Move { create_dirs, .. }
        | Action::Copy { create_dirs, .. }
        | Action::Compress { create_dirs, .. } => *create_dirs,
        Action::Rename { .. } | Action::Delete | Action::Skip => false,
    };

    if create_dirs && !dry_run {
        fs::create_dir_all(&dir_path).map_err(|e| {
            log::error!("Failed to create directories: {e}");
            e.to_string()
        })?;
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

fn run_fs_op<F, R>(
    src: &Path,
    dest: &Path,
    dry_run: bool,
    op: F,
    op_name: &str,
) -> Result<(), String>
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
        op(src, dest).map_err(|e| {
            log::error!("Failed to {} file: {e}", op_name);
            e.to_string()
        })?;
        Ok(())
    }
}

fn handle_delete(file_path: &Path, dry_run: bool) -> Result<FileOperationResult, String> {
    if dry_run {
        log::debug!("Dry run: would delete file: {}", file_path.display());
    } else {
        log::info!("Deleting file: {}", file_path.display());
        fs::remove_file(file_path).map_err(|e| {
            log::error!("Failed to delete file: {e}");
            e.to_string()
        })?;
    }

    Ok(FileOperationResult {
        new_path: PathBuf::new(),
        renamed: "[deleted]".into(),
    })
}

fn compress_file(input_path: &Path, output_path: &Path) -> Result<(), String> {
    let mut input = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut output = fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut encoder = GzEncoder::new(&mut output, Compression::default());

    std::io::copy(&mut input, &mut encoder).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn expand_templates(file_path: &Path, action: &Action) -> Result<(PathBuf, String), String> {
    let file_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid file name")?;

    let ext = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("dat");

    let now = Utc::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();

    // Extract destination and optional path_template and rename_template based on Action variant
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
            Action::Compress {
                destination,
                format: _,
                create_dirs: _,
            } => (destination, None, None),
            Action::Rename { rename_template } => {
                // Use empty string for base, None for path_template, Some(&rename_template) for rename_template
                static EMPTY: String = String::new();
                (&EMPTY, None, Some(rename_template))
            }
            Action::Delete | Action::Skip => {
                return Err("No templates for Delete/Skip actions".into());
            }
        };

    if base.is_empty() && rename_template.is_none() {
        return Err("Missing destination or rename_template".into());
    }

    // Expand tilde in base path, if any
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

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
