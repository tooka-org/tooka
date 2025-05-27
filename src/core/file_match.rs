use crate::core::rule;
use crate::core::rule::Match as RuleMatch;
use chrono::{NaiveDate, Utc};
use glob;
use std::fs;
use std::path::Path;

fn match_extensions(file_path: &Path, extensions: &[String]) -> bool {
    log::debug!(
        "Matching extensions {:?} for file: {}",
        extensions,
        file_path.display()
    );
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext_str| extensions.iter().any(|ext| ext == ext_str))
        .unwrap_or(false)
}

fn match_mime_type(file_path: &Path, mime_type: &str) -> bool {
    log::debug!(
        "Matching MIME type '{}' for file: {}",
        mime_type,
        file_path.display()
    );
    mime_guess::from_path(file_path)
        .first()
        .is_some_and(|mime| {
            let mime_essence = mime.essence_str();
            if let Some(prefix) = mime_type.strip_suffix("/*") {
                // Handle wildcard MIME type, e.g., image/*
                log::debug!("Handling wildcard MIME type: {}", mime_type);
                mime_essence.starts_with(prefix)
            } else {
                log::debug!("Exact MIME type match: {}", mime_type);
                mime_essence == mime_type
            }
        })
}

fn match_pattern(file_path: &Path, pattern: &str) -> bool {
    let file_path_str = file_path.to_string_lossy();
    log::debug!(
        "Matching glob pattern '{}' for file path: '{}'",
        pattern,
        file_path_str
    );

    match glob::Pattern::new(pattern) {
        Ok(glob_pattern) => {
            let res = glob_pattern.matches(&file_path_str);
            log::debug!("Glob pattern match result: {}", res);
            res
        }
        Err(e) => {
            log::error!("Invalid glob pattern '{}': {}", pattern, e);
            false
        }
    }
}

fn match_metadata(file_path: &Path, metadata_match: &rule::MetadataMatch) -> bool {
    log::debug!("Matching metadata for file: {}", file_path.display());

    if metadata_match.exif_date {
        if let Ok(file) = fs::File::open(file_path) {
            let mut bufreader = std::io::BufReader::new(file);
            match exif::Reader::new().read_from_container(&mut bufreader) {
                Ok(exif) => {
                    if exif
                        .get_field(exif::Tag::DateTime, exif::In::PRIMARY)
                        .is_none()
                    {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        } else {
            return false;
        }
    }

    for field in &metadata_match.fields {
        if !match_metadata_field(file_path, field) {
            return false;
        }
    }

    true
}

fn match_metadata_field(file_path: &Path, field: &rule::MetadataField) -> bool {
    let file = match fs::File::open(file_path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut bufreader = std::io::BufReader::new(file);
    let exif = match exif::Reader::new().read_from_container(&mut bufreader) {
        Ok(e) => e,
        Err(_) => return false,
    };

    for f in exif.fields() {
        let tag_name = format!("EXIF:{:?}", f.tag);
        if tag_name != field.key {
            continue;
        }
        if let Some(ref pattern_str) = field.value {
            let pattern = match glob::Pattern::new(pattern_str) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let value = f.display_value().with_unit(&exif).to_string();
            if pattern.matches(&value) {
                return true;
            }
        }
    }
    false
}

fn match_older_than_days(metadata: &fs::Metadata, days: u32) -> bool {
    if let Ok(modified) = metadata.modified() {
        let modified_utc: chrono::DateTime<Utc> = modified.into();
        let age = Utc::now().signed_duration_since(modified_utc);
        age.num_days() >= i64::from(days)
    } else {
        false
    }
}

fn match_size_greater_than_kb(metadata: &fs::Metadata, min_kb: u64) -> bool {
    metadata.len() >= min_kb * 1024
}

fn match_created_between(metadata: &fs::Metadata, range: &rule::DateRange) -> bool {
    if let Ok(created) = metadata.created() {
        let created_date = chrono::DateTime::<Utc>::from(created).date_naive();
        let from = NaiveDate::parse_from_str(&range.from, "%Y-%m-%d").unwrap_or_else(|_| {
            log::warn!("Failed to parse 'from' date: {}", &range.from);
            NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
        });
        let to = NaiveDate::parse_from_str(&range.to, "%Y-%m-%d").unwrap_or_else(|_| {
            log::warn!("Failed to parse 'to' date: {}", &range.to);
            NaiveDate::from_ymd_opt(9999, 12, 31).unwrap()
        });
        created_date >= from && created_date <= to
    } else {
        false
    }
}

fn match_filename_regex(file_path: &Path, pattern: &str) -> bool {
    let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let regex = match regex::Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => {
            log::warn!("Invalid filename regex '{}': {}", pattern, e);
            return false;
        }
    };
    regex.is_match(file_name)
}

fn match_is_symlink(metadata: &fs::Metadata, is_symlink: bool) -> bool {
    metadata.file_type().is_symlink() == is_symlink
}

fn match_file_owner(metadata: &fs::Metadata, owner: &str) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use users::get_user_by_uid;
        if let Some(user) = get_user_by_uid(metadata.uid()) {
            // Compare username as OsStr to &str safely
            return user.name().to_str() == Some(owner);
        }
    }
    #[cfg(windows)]
    {
        log::warn!("Owner matching not implemented on Windows.");
    }
    false
}

pub fn match_rule_matcher(file_path: &Path, matcher: &RuleMatch) -> bool {
    if let Some(ref exts) = matcher.extensions {
        if !match_extensions(file_path, exts) {
            log::info!(
                "File '{}' does not match required extensions: {:?}",
                file_path.display(),
                exts
            );
            return false;
        }
    }

    if let Some(ref mime) = matcher.mime_type {
        if !match_mime_type(file_path, mime) {
            log::info!(
                "File '{}' does not match required MIME type: {}",
                file_path.display(),
                mime
            );
            return false;
        }
    }

    if let Some(ref pattern) = matcher.pattern {
        if !match_pattern(file_path, pattern) {
            log::info!(
                "File '{}' does not match required pattern: {}",
                file_path.display(),
                pattern
            );
            return false;
        }
    }

    if let Some(ref metadata) = matcher.metadata {
        if !match_metadata(file_path, metadata) {
            log::info!(
                "File '{}' does not match required metadata: {:?}",
                file_path.display(),
                metadata
            );
            return false;
        }
    }

    let metadata = match fs::symlink_metadata(file_path) {
        Ok(meta) => meta,
        Err(e) => {
            log::warn!("Failed to get metadata for {}: {}", file_path.display(), e);
            return false;
        }
    };

    if let Some(days) = matcher.older_than_days {
        if !match_older_than_days(&metadata, days) {
            log::info!(
                "File '{}' is not older than {} days",
                file_path.display(),
                days
            );
            return false;
        }
    }
    if let Some(size) = matcher.size_greater_than_kb {
        if !match_size_greater_than_kb(&metadata, size) {
            log::info!(
                "File '{}' is not larger than {} KB",
                file_path.display(),
                size
            );
            return false;
        }
    }
    if let Some(ref range) = matcher.created_between {
        if !match_created_between(&metadata, range) {
            log::info!(
                "File '{}' does not have creation date within range: {:?}",
                file_path.display(),
                range
            );
            return false;
        }
    }
    if let Some(ref regex) = matcher.filename_regex {
        if !match_filename_regex(file_path, regex) {
            log::info!(
                "File '{}' does not match filename regex: {}",
                file_path.display(),
                regex
            );
            return false;
        }
    }
    if let Some(is_symlink) = matcher.is_symlink {
        if !match_is_symlink(&metadata, is_symlink) {
            log::info!(
                "File '{}' is not a symlink as required: {}",
                file_path.display(),
                is_symlink
            );
            return false;
        }
    }
    if let Some(ref owner) = matcher.owner {
        if !match_file_owner(&metadata, owner) {
            log::info!(
                "File '{}' does not match owner: {}",
                file_path.display(),
                owner
            );
            return false;
        }
    }
    log::debug!(
        "File '{}' matches rule matcher: {:?}",
        file_path.display(),
        matcher
    );

    true
}
