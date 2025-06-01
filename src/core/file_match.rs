use crate::core::rule;
use crate::core::rule::Match as RuleMatch;
use crate::error::TookaError;

use chrono::{NaiveDate, Utc};
use glob;
use std::fs;
use std::path::Path;

fn match_extensions(file_path: &Path, extensions: &[String]) -> bool {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext_str| extensions.iter().any(|ext| ext == ext_str))
        .unwrap_or(false)
}

fn match_mime_type(file_path: &Path, mime_type: &str) -> bool {
    mime_guess::from_path(file_path)
        .first()
        .is_some_and(|mime| {
            let mime_essence = mime.essence_str();
            if let Some(prefix) = mime_type.strip_suffix("/*") {
                mime_essence.starts_with(prefix)
            } else {
                mime_essence == mime_type
            }
        })
}

fn match_pattern(file_path: &Path, pattern: &str) -> Result<bool, TookaError> {
    let file_path_str = file_path.to_string_lossy();
    let glob_pattern = glob::Pattern::new(pattern)?;
    Ok(glob_pattern.matches(&file_path_str))
}

fn match_metadata(file_path: &Path, metadata_match: &rule::MetadataMatch) -> bool {
    (
        if metadata_match.exif_date {
            let file = match fs::File::open(file_path) {
                Ok(f) => f,
                Err(_) => return false,
            };
            let mut reader = std::io::BufReader::new(file);
            match exif::Reader::new().read_from_container(&mut reader) {
                Ok(exif) => exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY).is_some(),
                Err(_) => false,
            }
        } else {
            true
        }
    ) && metadata_match.fields.iter().all(|field| match_metadata_field(file_path, field))
}

fn match_metadata_field(file_path: &Path, field: &rule::MetadataField) -> bool {
    let file = match fs::File::open(file_path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut reader = std::io::BufReader::new(file);
    let exif = match exif::Reader::new().read_from_container(&mut reader) {
        Ok(e) => e,
        Err(_) => return false,
    };

    exif.fields().any(|f| {
        let tag_name = format!("EXIF:{:?}", f.tag);
        if tag_name != field.key {
            return false;
        }

        match &field.value {
            Some(pattern_str) => glob::Pattern::new(pattern_str)
                .map(|p| p.matches(&f.display_value().with_unit(&exif).to_string()))
                .unwrap_or(false),
            None => true,
        }
    })
}

fn match_older_than_days(metadata: &fs::Metadata, days: u32) -> bool {
    metadata.modified().map_or(false, |modified| {
        let modified_datetime: chrono::DateTime<Utc> = modified.into();
        let age = Utc::now().signed_duration_since(modified_datetime);
        age.num_days() >= i64::from(days)
    })
}

fn match_size_greater_than_kb(metadata: &fs::Metadata, min_kb: u64) -> bool {
    metadata.len() >= min_kb * 1024
}

fn match_created_between(metadata: &fs::Metadata, range: &rule::DateRange) -> bool {
    metadata.created().map_or(false, |created| {
        let created_date = chrono::DateTime::<Utc>::from(created).date_naive();
        let from = NaiveDate::parse_from_str(&range.from, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let to = NaiveDate::parse_from_str(&range.to, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());
        created_date >= from && created_date <= to
    })
}

fn match_filename_regex(file_path: &Path, pattern: &str) -> Result<bool, TookaError> {
    let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let regex = regex::Regex::new(pattern)?;
    Ok(regex.is_match(file_name))
}

fn match_is_symlink(metadata: &fs::Metadata, is_symlink: bool) -> bool {
    metadata.file_type().is_symlink() == is_symlink
}

fn match_file_owner(metadata: &fs::Metadata, owner: &str) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use users::get_user_by_uid;
        get_user_by_uid(metadata.uid())
            .and_then(|user| user.name().to_str().map(|s| s.to_owned()))
            .map(|name| name == owner)
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        log::warn!("Owner matching not implemented on Windows.");
        false
    }
}

pub fn match_rule_matcher(file_path: &Path, matcher: &RuleMatch) -> bool {
    let metadata = match fs::symlink_metadata(file_path) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to read metadata for {}: {}", file_path.display(), e);
            return false;
        }
    };

    let matches = [
        matcher.extensions.as_ref().map(|exts| match_extensions(file_path, exts)),
        matcher.mime_type.as_ref().map(|m| match_mime_type(file_path, m)),
        matcher.pattern.as_ref().map(|p| match_pattern(file_path, p).unwrap_or(false)),
        matcher.metadata.as_ref().map(|m| match_metadata(file_path, m)),
        matcher.older_than_days.map(|d| match_older_than_days(&metadata, d)),
        matcher.size_greater_than_kb.map(|s| match_size_greater_than_kb(&metadata, s)),
        matcher.created_between.as_ref().map(|r| match_created_between(&metadata, r)),
        matcher.filename_regex.as_ref().map(|r| match_filename_regex(file_path, r).unwrap_or(false)),
        matcher.is_symlink.map(|b| match_is_symlink(&metadata, b)),
        matcher.owner.as_ref().map(|o| match_file_owner(&metadata, o)),
    ];

    if matches.iter().all(|res| res.unwrap_or(true)) {
        log::debug!("File '{}' matched all conditions", file_path.display());
        true
    } else {
        log::info!("File '{}' did not match one or more conditions", file_path.display());
        false
    }
}
