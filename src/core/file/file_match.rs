use crate::core::rules::rule::{DateRange, Range};
use crate::core::rules::{rule, rule::Conditions};
use crate::error::TookaError;

use chrono::{NaiveDate, Utc};
use glob;
use std::fs;
use std::path::Path;

/// Matches a file's name against a regular expression pattern
fn match_filename_regex(file_path: &Path, pattern: &str) -> Result<bool, TookaError> {
    let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let regex = regex::Regex::new(pattern)?;
    Ok(regex.is_match(file_name))
}

/// Matches a file against a given vector of file extensions
fn match_extensions(file_path: &Path, extensions: &[String]) -> bool {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext_str| extensions.iter().any(|ext| ext == ext_str))
        .unwrap_or(false)
}

/// Matches a file path against a glob pattern
fn match_path(file_path: &Path, pattern: &str) -> Result<bool, TookaError> {
    let file_path_str = file_path.to_string_lossy();
    let glob_pattern = glob::Pattern::new(pattern)?;
    Ok(glob_pattern.matches(&file_path_str))
}

/// Matches a file's size against a given size range in kilobytes
fn match_size_kb(metadata: &fs::Metadata, size_kb: Range) -> bool {
    let size = metadata.len();
    let min = match size_kb.min {
        Some(m) => m.saturating_mul(1024),
        None => 0,
    };
    let max = match size_kb.max {
        Some(m) => m.saturating_mul(1024),
        None => u64::MAX,
    };
    size >= min && size <= max
}

/// Matches a file's MIME type against a given MIME type string
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

/// Matches a file's metadata against a date range
fn match_date_range_created(metadata: &fs::Metadata, date_range: DateRange) -> bool {
    metadata.created().is_ok_and(|created| {
        let created_datetime: chrono::DateTime<Utc> = created.into();
        let from = NaiveDate::parse_from_str(
            date_range.from.as_deref().unwrap_or("1970-01-01"),
            "%Y-%m-%d",
        )
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let to =
            NaiveDate::parse_from_str(date_range.to.as_deref().unwrap_or("9999-12-31"), "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());
        let created_date = created_datetime.date_naive();
        created_date >= from && created_date <= to
    })
}

/// Matches a file's metadata against a date range
fn match_date_range_mod(metadata: &fs::Metadata, date_range: DateRange) -> bool {
    metadata.modified().is_ok_and(|modified| {
        let modified_datetime: chrono::DateTime<Utc> = modified.into();
        let from = NaiveDate::parse_from_str(
            date_range.from.as_deref().unwrap_or("1970-01-01"),
            "%Y-%m-%d",
        )
        .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let to =
            NaiveDate::parse_from_str(date_range.to.as_deref().unwrap_or("9999-12-31"), "%Y-%m-%d")
                .unwrap_or_else(|_| NaiveDate::from_ymd_opt(9999, 12, 31).unwrap());
        let modified_date = modified_datetime.date_naive();
        modified_date >= from && modified_date <= to
    })
}

/// Matches a file's symlink status against a boolean value
fn match_is_symlink(metadata: &fs::Metadata, is_symlink: bool) -> bool {
    metadata.file_type().is_symlink() == is_symlink
}

/// Matches a specific metadata field against the file's EXIF data
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

/// Matches a file against a set of rules defined in a RuleMatch struct
pub fn match_rule_matcher(file_path: &Path, conditions: &Conditions) -> bool {
    let metadata = match fs::symlink_metadata(file_path) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to read metadata for {}: {}", file_path.display(), e);
            return false;
        }
    };

    let matches = [
        conditions.filename.as_ref().map_or(Ok(false), |pattern| {
            match_filename_regex(file_path, pattern)
        }),
        conditions
            .extensions
            .as_ref()
            .map_or(Ok(false), |exts| Ok(match_extensions(file_path, exts))),
        conditions
            .path
            .as_ref()
            .map_or(Ok(false), |pattern| match_path(file_path, pattern)),
        conditions
            .size_kb
            .as_ref()
            .map_or(Ok(false), |size| Ok(match_size_kb(&metadata, size.clone()))),
        conditions
            .mime_type
            .as_ref()
            .map_or(Ok(false), |m| Ok(match_mime_type(file_path, m))),
        conditions
            .created_date
            .as_ref()
            .map_or(Ok(false), |date_range| {
                Ok(match_date_range_created(&metadata, date_range.clone()))
            }),
        conditions
            .modified_date
            .as_ref()
            .map_or(Ok(false), |date_range| {
                Ok(match_date_range_mod(&metadata, date_range.clone()))
            }),
        conditions
            .is_symlink
            .map_or(Ok(false), |b| Ok(match_is_symlink(&metadata, b))),
        conditions
            .metadata
            .as_ref()
            .map_or(Ok(false), |metadata_fields| {
                Ok(metadata_fields
                    .iter()
                    .all(|field| match_metadata_field(file_path, field)))
            }),
    ];

    // If any is not set, it defaults to false.
    // If it is set, and its set to true, any of the conditions must match.
    // If its not set, or set to false, all conditions must match.
    let any_conditions = conditions.any.unwrap_or(false);
    if any_conditions {
        matches.into_iter().any(|m| m.unwrap_or(false))
    } else {
        matches.into_iter().all(|m| m.unwrap_or(false))
    }
}
