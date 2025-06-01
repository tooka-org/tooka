use std::{
    env,
    path::{Path, PathBuf},
};

use directories_next::{ProjectDirs, UserDirs};
use crate::context::{APP_NAME, APP_ORG, APP_QUALIFIER};
use crate::error::TookaError;

/// Returns a directory path from an env var or project dir fallback.
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

/// Returns the source folder, usually Downloads, or a fallback.
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
