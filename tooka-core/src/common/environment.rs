//! Environment-aware directory resolution utilities for the Tooka application.
//!
//! This module provides helper functions to resolve user-specific directories,
//! such as config and data paths, using environment variables and system conventions.
//!
//! It includes logic to fall back to default locations if environment variables
//! or system directories are unavailable.

use crate::{
    core::context::{APP_NAME, APP_ORG, APP_QUALIFIER},
    core::error::TookaError,
};
use directories_next::{ProjectDirs, UserDirs};
use std::{
    env,
    path::{Path, PathBuf},
};

/// Returns a directory path from an environment variable or fallback.
///
/// Prefers the value of the given environment variable. If not set, uses a
/// standard project directory (like config or data). Falls back to `$HOME/<fallback_subdir>/<app>`
/// if none are found.
///
/// # Errors
/// Returns [`TookaError`] if path resolution fails.
pub fn get_dir_with_env<F>(
    env_var: &str,
    project_dir_fn: F,
    home: &Path,
    fallback_subdir: &str,
) -> Result<PathBuf, TookaError>
where
    F: Fn(&ProjectDirs) -> &Path,
{
    if let Ok(path) = env::var(env_var).map(PathBuf::from) {
        return Ok(path);
    }

    if let Some(proj_dirs) = ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME) {
        return Ok(project_dir_fn(&proj_dirs).to_path_buf());
    }

    // Fallback
    let fallback = home.join(fallback_subdir).join(APP_NAME.to_lowercase());
    log::warn!("Using fallback for {}: {}", env_var, fallback.display());
    Ok(fallback)
}

/// Returns the source folder path, usually pointing to the Downloads directory.
///
/// Uses `TOOKA_SOURCE_FOLDER` if set. Otherwise tries to locate the system's
/// Downloads directory. Falls back to `$HOME/Downloads` if necessary.
///
/// # Errors
/// Returns [`TookaError`] if path resolution fails.
pub fn get_source_folder(home: &Path) -> Result<PathBuf, TookaError> {
    if let Ok(path) = env::var("TOOKA_SOURCE_FOLDER").map(PathBuf::from) {
        return Ok(path);
    }

    if let Some(user_dirs) = UserDirs::new() {
        if let Some(downloads) = user_dirs.download_dir() {
            return Ok(downloads.to_path_buf());
        }
    }

    // Fallback to ~/Downloads
    let fallback = home.join("Downloads");
    log::warn!(
        "Could not find user downloads dir; using fallback: {}",
        fallback.display()
    );
    Ok(fallback)
}
