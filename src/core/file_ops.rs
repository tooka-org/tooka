use std::fs;
use std::path::{Path, PathBuf};
use chrono::{Datelike, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::core::rules::Action;

pub struct FileOperationResult {
    pub new_path: PathBuf,
    pub renamed: String,
}

/// Dispatches the appropriate action handler.
pub fn execute_action(
    file_path: &Path,
    action: &Action,
    dry_run: bool,
) -> Result<FileOperationResult, String> {
    log::info!(
        "Executing action '{}' on file: {:?} (dry_run: {})",
        action.r#type, file_path, dry_run
    );
    match action.r#type.as_str() {
        "move" => handle_move(file_path, action, dry_run),
        "copy" => handle_copy(file_path, action, dry_run),
        "delete" => handle_delete(file_path, dry_run),
        "rename" => handle_rename(file_path, action, dry_run),
        "compress" => handle_compress(file_path, action, dry_run),
        "skip" => {
            log::info!("Skipping file: {:?}", file_path);
            Ok(FileOperationResult {
                new_path: PathBuf::from(file_path),
                renamed: String::from("[skipped]"),
            })
        },
        other => {
            log::error!("Unsupported action type: {}", other);
            Err(format!("Unsupported action type: {}", other))
        },
    }
}

fn expand_templates(
    file_path: &Path,
    action: &Action,
) -> Result<(PathBuf, String), String> {
    log::debug!("Expanding templates for file: {:?}", file_path);

    let file_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid file name")?;

    let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("dat");

    let now = Utc::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();

    let base = action.destination.as_ref().ok_or("Missing destination")?;
    let expanded_base = shellexpand::tilde(base).into_owned();

    let sub_path = if let Some(ref template) = action.path_template {
        template
            .format
            .replace("{year}", &format!("{:04}", year))
            .replace("{month}", &format!("{:02}", month))
            .replace("{day}", &format!("{:02}", day))
    } else {
        "".into()
    };

    let full_path = Path::new(&expanded_base).join(sub_path);

    let renamed = action
        .rename_template
        .as_ref()
        .map(|t| {
            t.replace("{filename}", file_name)
                .replace("{year}", &format!("{:04}", year))
                .replace("{month}", &format!("{:02}", month))
                .replace("{day}", &format!("{:02}", day))
                .replace("{ext}", ext)
        })
        .unwrap_or_else(|| format!("{}.{}", file_name, ext));

    log::debug!(
        "Expanded path: {:?}, renamed: {}",
        full_path, renamed
    );

    Ok((full_path, renamed))
}

fn handle_move(file_path: &Path, action: &Action, dry_run: bool) -> Result<FileOperationResult, String> {
    let (dir_path, renamed) = expand_templates(file_path, action)?;

    if action.create_dirs.unwrap_or(false) && !dry_run {
        log::info!("Creating directories: {:?}", dir_path);
        if let Err(e) = fs::create_dir_all(&dir_path) {
            log::error!("Failed to create directories: {}", e);
            return Err(e.to_string());
        }
    }

    let target_path = dir_path.join(&renamed);

    if !dry_run {
        log::info!("Moving file from {:?} to {:?}", file_path, target_path);
        if let Err(e) = fs::rename(file_path, &target_path) {
            log::error!("Failed to move file: {}", e);
            return Err(e.to_string());
        }
    } else {
        log::debug!("Dry run: would move file from {:?} to {:?}", file_path, target_path);
    }

    Ok(FileOperationResult {
        new_path: target_path,
        renamed,
    })
}

fn handle_copy(file_path: &Path, action: &Action, dry_run: bool) -> Result<FileOperationResult, String> {
    let (dir_path, renamed) = expand_templates(file_path, action)?;

    if action.create_dirs.unwrap_or(false) && !dry_run {
        log::info!("Creating directories: {:?}", dir_path);
        if let Err(e) = fs::create_dir_all(&dir_path) {
            log::error!("Failed to create directories: {}", e);
            return Err(e.to_string());
        }
    }

    let target_path = dir_path.join(&renamed);

    if !dry_run {
        log::info!("Copying file from {:?} to {:?}", file_path, target_path);
        if let Err(e) = fs::copy(file_path, &target_path) {
            log::error!("Failed to copy file: {}", e);
            return Err(e.to_string());
        }
    } else {
        log::debug!("Dry run: would copy file from {:?} to {:?}", file_path, target_path);
    }

    Ok(FileOperationResult {
        new_path: target_path,
        renamed,
    })
}

fn handle_delete(file_path: &Path, dry_run: bool) -> Result<FileOperationResult, String> {
    if !dry_run {
        log::info!("Deleting file: {:?}", file_path);
        if let Err(e) = fs::remove_file(file_path) {
            log::error!("Failed to delete file: {}", e);
            return Err(e.to_string());
        }
    } else {
        log::debug!("Dry run: would delete file: {:?}", file_path);
    }

    Ok(FileOperationResult {
        new_path: PathBuf::new(),
        renamed: String::from("[deleted]"),
    })
}

fn handle_compress(file_path: &Path, action: &Action, dry_run: bool) -> Result<FileOperationResult, String> {
    let (dir_path, renamed) = expand_templates(file_path, action)?;

    if action.create_dirs.unwrap_or(false) && !dry_run {
        log::info!("Creating directories: {:?}", dir_path);
        if let Err(e) = fs::create_dir_all(&dir_path) {
            log::error!("Failed to create directories: {}", e);
            return Err(e.to_string());
        }
    }

    let target_path = dir_path.join(format!("{}.gz", renamed));

    if !dry_run {
        log::info!("Compressing file {:?} to {:?}", file_path, target_path);
        let mut input = fs::File::open(file_path).map_err(|e| {
            log::error!("Failed to open input file: {}", e);
            e.to_string()
        })?;
        let mut output = fs::File::create(&target_path).map_err(|e| {
            log::error!("Failed to create output file: {}", e);
            e.to_string()
        })?;
        let mut encoder = GzEncoder::new(&mut output, Compression::default());

        std::io::copy(&mut input, &mut encoder).map_err(|e| {
            log::error!("Failed to compress file: {}", e);
            e.to_string()
        })?;
        encoder.finish().map_err(|e| {
            log::error!("Failed to finish compression: {}", e);
            e.to_string()
        })?;
    } else {
        log::debug!("Dry run: would compress file {:?} to {:?}", file_path, target_path);
    }

    Ok(FileOperationResult {
        new_path: target_path,
        renamed: format!("{}.gz", renamed),
    })
}

fn handle_rename(file_path: &Path, action: &Action, dry_run: bool) -> Result<FileOperationResult, String> {
    let (dir_path, renamed) = expand_templates(file_path, action)?;

    if action.create_dirs.unwrap_or(false) && !dry_run {
        log::info!("Creating directories: {:?}", dir_path);
        if let Err(e) = fs::create_dir_all(&dir_path) {
            log::error!("Failed to create directories: {}", e);
            return Err(e.to_string());
        }
    }

    let target_path = dir_path.join(&renamed);

    if !dry_run {
        log::info!("Renaming file from {:?} to {:?}", file_path, target_path);
        if let Err(e) = fs::rename(file_path, &target_path) {
            log::error!("Failed to rename file: {}", e);
            return Err(e.to_string());
        }
    } else {
        log::debug!("Dry run: would rename file from {:?} to {:?}", file_path, target_path);
    }

    Ok(FileOperationResult {
        new_path: target_path,
        renamed,
    })
}