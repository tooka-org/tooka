//! File matching utilities for Tooka.
//!
//! This module provides functions to match files against various criteria,
//! including filename patterns, extensions, paths, sizes, MIME types, dates,
//! symlink status, EXIF metadata, and combined rule conditions.

use crate::{
    core::error::TookaError,
    rules::rule::{self, Conditions, DateRange, Range},
    utils::date_parser::parse_date,
};

use chrono::{NaiveDate, Utc};
use exif::Reader;
use glob::{self, Pattern};
use std::fs;
use std::io::BufReader;
use std::path::Path;

const MIN_DATE: (i32, u32, u32) = (1970, 1, 1);
const MAX_DATE: (i32, u32, u32) = (9999, 12, 31);

/// Matches a file's name against a regular expression pattern
pub(crate) fn match_filename_regex(file_path: &Path, pattern: &str) -> Result<bool, TookaError> {
    log::debug!(
        "Matching file: {} against pattern: {}",
        file_path.display(),
        pattern
    );
    let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let regex = regex::Regex::new(pattern)?;
    Ok(regex.is_match(file_name))
}

/// Matches a file against a given vector of file extensions
pub(crate) fn match_extensions(file_path: &Path, extensions: &[String]) -> bool {
    log::debug!(
        "Matching file: {} against extensions: {:?}",
        file_path.display(),
        extensions
    );
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext_str| extensions.iter().any(|ext| ext == ext_str))
}

/// Matches a file path against a glob pattern
pub(crate) fn match_path(file_path: &Path, pattern: &str) -> Result<bool, TookaError> {
    log::debug!(
        "Matching file: {} against glob pattern: {}",
        file_path.display(),
        pattern
    );
    let file_path_str = file_path.to_string_lossy();
    let glob_pattern = glob::Pattern::new(pattern)?;
    Ok(glob_pattern.matches(&file_path_str))
}

/// Matches a file's size against a given size range in kilobytes
pub(crate) fn match_size_kb(metadata: &fs::Metadata, size_kb: &Range) -> bool {
    log::debug!(
        "Matching file size: {} against range: {:?}",
        metadata.len(),
        size_kb
    );
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
pub(crate) fn match_mime_type(file_path: &Path, mime_type: &str) -> bool {
    log::debug!(
        "Matching file: {} against MIME type: {}",
        file_path.display(),
        mime_type
    );
    mime_guess::from_path(file_path)
        .first()
        .is_some_and(|mime| {
            let mime_essence = mime.essence_str();
            mime_type
                .strip_suffix("/*")
                .map_or(mime_essence == mime_type, |prefix| {
                    mime_essence.starts_with(prefix)
                })
        })
}

/// Helper function to parse date with fallback
fn parse_date_with_fallback(date_str: &str, fallback: NaiveDate) -> NaiveDate {
    parse_date(date_str).map_or_else(
        |_| {
            log::warn!("Invalid date format: {date_str}, using fallback");
            fallback
        },
        |dt| dt.date_naive(),
    )
}

/// Matches a file's metadata against a date range
pub(crate) fn match_date_range_created(metadata: &fs::Metadata, date_range: &DateRange) -> bool {
    log::debug!("Matching against created date range: {date_range:?}");

    metadata.created().is_ok_and(|created| {
        let created_datetime: chrono::DateTime<Utc> = created.into();
        let created_date = created_datetime.date_naive();

        let min_date = NaiveDate::from_ymd_opt(MIN_DATE.0, MIN_DATE.1, MIN_DATE.2)
            .expect("MIN_DATE should be valid");
        let max_date = NaiveDate::from_ymd_opt(MAX_DATE.0, MAX_DATE.1, MAX_DATE.2)
            .expect("MAX_DATE should be valid");

        let from = date_range.from.as_ref().map_or_else(
            || min_date,
            |from_str| parse_date_with_fallback(from_str, min_date),
        );

        let to = date_range.to.as_ref().map_or_else(
            || max_date,
            |to_str| parse_date_with_fallback(to_str, max_date),
        );

        created_date >= from && created_date <= to
    })
}

/// Matches a file's metadata against a date range
pub(crate) fn match_date_range_mod(metadata: &fs::Metadata, date_range: &DateRange) -> bool {
    log::debug!("Matching against modified date range: {date_range:?}");

    metadata.modified().is_ok_and(|modified| {
        let modified_datetime: chrono::DateTime<Utc> = modified.into();
        let modified_date = modified_datetime.date_naive();

        let min_date = NaiveDate::from_ymd_opt(MIN_DATE.0, MIN_DATE.1, MIN_DATE.2)
            .expect("MIN_DATE should be valid");
        let max_date = NaiveDate::from_ymd_opt(MAX_DATE.0, MAX_DATE.1, MAX_DATE.2)
            .expect("MAX_DATE should be valid");

        let from = date_range.from.as_ref().map_or_else(
            || min_date,
            |from_str| parse_date_with_fallback(from_str, min_date),
        );

        let to = date_range.to.as_ref().map_or_else(
            || max_date,
            |to_str| parse_date_with_fallback(to_str, max_date),
        );

        modified_date >= from && modified_date <= to
    })
}

/// Matches a file's symlink status against a boolean value
pub(crate) fn match_is_symlink(metadata: &fs::Metadata, is_symlink: bool) -> bool {
    log::debug!(
        "Matching symlink status: {} against expected: {}",
        metadata.file_type().is_symlink(),
        is_symlink
    );
    metadata.file_type().is_symlink() == is_symlink
}

/// Matches a specific metadata field (e.g., EXIF) against a file
pub(crate) fn match_metadata_field(file_path: &Path, field: &rule::MetadataField) -> bool {
    log::debug!(
        "Checking metadata field match for key '{}' on file '{}'",
        field.key,
        file_path.display()
    );

    let file = match fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            log::warn!("Failed to open file '{}': {}", file_path.display(), e);
            return false;
        }
    };

    let mut reader = BufReader::new(file);
    let exif = match Reader::new().read_from_container(&mut reader) {
        Ok(r) => r,
        Err(e) => {
            log::debug!("No EXIF data found in '{}': {}", file_path.display(), e);
            return false;
        }
    };

    let requested_key = field.key.to_lowercase();

    for f in exif.fields() {
        let exif_key = format!("EXIF:{:?}", f.tag).to_lowercase();
        let value_str = f.display_value().with_unit(&exif).to_string();

        if exif_key == requested_key {
            log::debug!("Found EXIF key match: '{exif_key}'");

            if let Some(pattern_str) = &field.value {
                match Pattern::new(pattern_str) {
                    Ok(pattern) => {
                        let is_match = pattern.matches(&value_str);
                        log::debug!(
                            "Comparing EXIF value '{value_str}' with pattern '{pattern_str}': {is_match}"
                        );
                        return is_match;
                    }
                    Err(e) => {
                        log::warn!("Invalid glob pattern '{pattern_str}': {e}");
                        return false;
                    }
                }
            }
            log::debug!("EXIF key '{exif_key}' matched without value filter");
            return true;
        }
    }

    log::debug!(
        "No matching EXIF key '{}' found in file '{}'",
        field.key,
        file_path.display()
    );

    false
}

/// Matches a file against all specified conditions in a rule.
///
/// Uses OR logic if `conditions.any` is true; otherwise AND logic.
pub fn match_rule_matcher(file_path: &Path, conditions: &Conditions) -> bool {
    log::debug!(
        "Matching file: {} against conditions: {:?}",
        file_path.display(),
        conditions
    );
    let metadata = match fs::symlink_metadata(file_path) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to read metadata for {}: {}", file_path.display(), e);
            return false;
        }
    };
    log::debug!("File metadata: {metadata:?}");

    let matches = [
        conditions
            .filename
            .as_ref()
            .map_or(Ok(true), |pattern| match_filename_regex(file_path, pattern)),
        conditions
            .extensions
            .as_ref()
            .map_or(Ok(true), |exts| Ok(match_extensions(file_path, exts))),
        conditions
            .path
            .as_ref()
            .map_or(Ok(true), |pattern| match_path(file_path, pattern)),
        conditions
            .size_kb
            .as_ref()
            .map_or(Ok(true), |size| Ok(match_size_kb(&metadata, size))),
        conditions
            .mime_type
            .as_ref()
            .map_or(Ok(true), |m| Ok(match_mime_type(file_path, m))),
        conditions
            .created_date
            .as_ref()
            .map_or(Ok(true), |date_range| {
                Ok(match_date_range_created(&metadata, date_range))
            }),
        conditions
            .modified_date
            .as_ref()
            .map_or(Ok(true), |date_range| {
                Ok(match_date_range_mod(&metadata, date_range))
            }),
        conditions
            .is_symlink
            .map_or(Ok(true), |b| Ok(match_is_symlink(&metadata, b))),
        conditions
            .metadata
            .as_ref()
            .map_or(Ok(true), |metadata_fields| {
                Ok(metadata_fields
                    .iter()
                    .all(|field| match_metadata_field(file_path, field)))
            }),
    ];
    let any_conditions = conditions.any.unwrap_or(false);
    log::debug!("Conditions any: {any_conditions}, matches: {matches:?}");
    if any_conditions {
        log::debug!("Using OR logic for conditions");
        matches.into_iter().any(|m| m.unwrap_or(false))
    } else {
        log::debug!("Using AND logic for conditions");
        matches.into_iter().all(|m| m.unwrap_or(false))
    }
}
