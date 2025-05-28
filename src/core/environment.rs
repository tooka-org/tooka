use std::{env, io, path::{Path, PathBuf}};
use directories_next::{ProjectDirs, UserDirs};
use crate::context::{APP_NAME, APP_ORG, APP_QUALIFIER};

/// Returns a directory path from an env var or project dir fallback.
pub fn get_dir_with_env<F>(env_var: &str, project_dir_fn: F, home: &Path, fallback_subdir: &str) -> PathBuf
where
    F: Fn(&ProjectDirs) -> &Path,
{
    env::var(env_var)
        .map(PathBuf::from)
        .or_else(|_| {
            ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
                .map(|d| project_dir_fn(&d).to_path_buf())
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "ProjectDirs unavailable"))
        })
        .unwrap_or_else(|_| {
            let fallback = home.join(fallback_subdir).join(APP_NAME.to_lowercase());
            log::warn!("Using fallback for {}: {}", env_var, fallback.display());
            fallback
        })
}

/// Returns the downloads folder or fallback.
pub fn get_source_folder(home: &Path) -> PathBuf {
    env::var("TOOKA_SOURCE_FOLDER")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            UserDirs::new()
                .and_then(|d| d.download_dir().map(Path::to_path_buf))
                .unwrap_or_else(|| {
                    let fallback = home.join("Downloads");
                    log::warn!("Could not find downloads dir; using fallback: {}", fallback.display());
                    fallback
                })
        })
}