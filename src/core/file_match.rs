use crate::core::rules;
use crate::core::rules::Match as RuleMatch;
use chrono::Utc;
use glob;
use std::fs;
use std::path::{Path, PathBuf};

fn match_extensions(file_path: &Path, extensions: &[String]) -> bool {
    log::debug!(
        "Matching extensions {:?} for file: {}",
        extensions,
        file_path.display()
    );
    if let Some(extension) = file_path.extension() {
        if let Some(extension_str) = extension.to_str() {
            return extensions.iter().any(|ext| ext == extension_str);
        }
    }
    false
}

fn match_mime_type(file_path: &PathBuf, mime_type: &str) -> bool {
    log::debug!(
        "Matching MIME type '{}' for file: {}",
        mime_type,
        file_path.display()
    );
    if let Some(mime) = mime_guess::from_path(file_path).first() {
        let mime_essence = mime.essence_str();
        if let Some(idx) = mime_type.find("/*") {
            log::debug!("Handling wildcard MIME type: {mime_type}");
            // Handle wildcard, e.g., "image/*"
            let prefix = &mime_type[..idx];
            return mime_essence.starts_with(prefix)
                && mime_essence.chars().nth(prefix.len()) == Some('/');
        }
        log::debug!("Exact MIME type match: {mime_type}");
        // Exact match
        return mime_essence == mime_type;
    }
    false
}

fn match_pattern(file_path: &Path, pattern: &str) -> bool {
    log::debug!(
        "Matching pattern '{}' for file: {}",
        pattern,
        file_path.display()
    );
    let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    log::debug!("Extracted file name for pattern matching: '{file_name}'");
    let regex = match regex::Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => {
            log::error!("Invalid regex pattern '{pattern}': {e}");
            return false; // Invalid regex pattern
        }
    };
    let res = regex.is_match(file_name);
    log::debug!("Pattern match result for file '{file_name}': {res}");
    res
}

fn match_metadata(file_path: &PathBuf, metadata_match: &rules::MetadataMatch) -> bool {
    // Check EXIF date if requested
    log::debug!("Matching metadata for file: {}", file_path.display());
    if metadata_match.exif_date {
        if let Ok(file) = fs::File::open(file_path) {
            let mut bufreader = std::io::BufReader::new(file);
            if let Ok(exif) = exif::Reader::new().read_from_container(&mut bufreader) {
                log::debug!("EXIF metadata found for file: {}", file_path.display());
                if exif
                    .get_field(exif::Tag::DateTime, exif::In::PRIMARY)
                    .is_none()
                {
                    log::debug!("No EXIF date found in file: {}", file_path.display());
                    return false;
                }
                log::debug!("EXIF date found in file: {}", file_path.display());
            } else {
                log::debug!(
                    "Failed to read EXIF metadata for file: {}",
                    file_path.display()
                );
                return false;
            }
        } else {
            log::debug!(
                "Failed to open file for EXIF metadata: {}",
                file_path.display()
            );
            return false;
        }
    }

    // Check additional metadata fields
    log::debug!(
        "Matching additional metadata fields for file: {}",
        file_path.display()
    );
    for field in &metadata_match.fields {
        if !match_metadata_field(file_path, field) {
            log::debug!(
                "Metadata field '{}' did not match for file: {}",
                field.key,
                file_path.display()
            );
            return false;
        }
    }
    log::debug!(
        "All metadata fields matched for file: {}",
        file_path.display()
    );
    true
}

fn match_metadata_field(file_path: &PathBuf, field: &rules::MetadataField) -> bool {
    log::debug!(
        "Matching metadata field '{}' for file: {}",
        field.key,
        file_path.display()
    );
    // Try to read EXIF metadata and match the field
    if let Ok(file) = fs::File::open(file_path) {
        log::debug!("Opened file for EXIF metadata: {}", file_path.display());
        let mut bufreader = std::io::BufReader::new(file);
        if let Ok(exif) = exif::Reader::new().read_from_container(&mut bufreader) {
            log::debug!(
                "EXIF metadata read successfully for file: {}",
                file_path.display()
            );
            // Try to find the field by key
            for f in exif.fields() {
                let tag_name = format!("EXIF:{:?}", f.tag);
                if tag_name == field.key {
                    log::debug!("Found matching EXIF field: {tag_name}");
                    let value = f.display_value().with_unit(&exif).to_string();
                    // Use glob pattern matching for the pattern
                    if let Some(ref pattern_str) = field.pattern {
                        log::debug!("Matching value '{value}' against pattern '{pattern_str}'");
                        if let Ok(pattern) = glob::Pattern::new(pattern_str) {
                            if pattern.matches(&value) {
                                log::debug!("Value '{value}' matches pattern '{pattern_str}'");
                                return true;
                            }
                            log::debug!("Value '{value}' does not match pattern '{pattern_str}'");
                        }
                    }
                }
            }
        }
    }
    log::debug!(
        "No matching metadata field '{}' found for file: {}",
        field.key,
        file_path.display()
    );
    false
}

fn match_conditions(file_path: &PathBuf, conditions: &rules::Conditions) -> bool {
    log::debug!("Matching conditions for file: {}", file_path.display());
    let metadata = match fs::symlink_metadata(file_path) {
        Ok(m) => m,
        Err(e) => {
            log::warn!(
                "Failed to get metadata for file {}: {}",
                file_path.display(),
                e
            );
            return false;
        }
    };

    // older_than_days
    if let Some(days) = conditions.older_than_days {
        log::debug!("Checking if file is older than {days} days");
        if let Ok(modified) = metadata.modified() {
            let modified_datetime: chrono::DateTime<Utc> = modified.into();
            let age = Utc::now().signed_duration_since(modified_datetime);
            log::debug!("File age in days: {}", age.num_days());
            if age.num_days() < i64::from(days) {
                log::debug!("File is not old enough ({} < {})", age.num_days(), days);
                return false;
            }
        } else {
            log::warn!(
                "Failed to get modified time for file: {}",
                file_path.display()
            );
            return false;
        }
    }

    // size_greater_than_kb
    if let Some(min_kb) = conditions.size_greater_than_kb {
        log::debug!("Checking if file size is greater than {min_kb} KB");
        if metadata.len() < min_kb * 1024 {
            log::debug!(
                "File size {} bytes is less than {} KB",
                metadata.len(),
                min_kb
            );
            return false;
        }
    }

    // created_between
    if let Some(ref range) = conditions.created_between {
        log::debug!(
            "Checking if file was created between {} and {}",
            range.from,
            range.to
        );
        if let Ok(created) = metadata.created() {
            let created_date = chrono::DateTime::<Utc>::from(created).date_naive();
            let from_date = match chrono::NaiveDate::parse_from_str(&range.from, "%Y-%m-%d") {
                Ok(date) => date,
                Err(e) => {
                    log::warn!("Failed to parse 'from' date '{}': {}", range.from, e);
                    return false;
                }
            };
            let to_date = match chrono::NaiveDate::parse_from_str(&range.to, "%Y-%m-%d") {
                Ok(date) => date,
                Err(e) => {
                    log::warn!("Failed to parse 'to' date '{}': {}", range.to, e);
                    return false;
                }
            };
            log::debug!("File created date: {created_date}");
            if created_date < from_date || created_date > to_date {
                log::debug!(
                    "File creation date {created_date} not in range {from_date} - {to_date}"
                );
                return false;
            }
        } else {
            log::warn!(
                "Failed to get created time for file: {}",
                file_path.display()
            );
            return false;
        }
    }

    // filename_regex
    if let Some(ref pattern) = conditions.filename_regex {
        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        log::debug!("Checking if file name '{file_name}' matches regex '{pattern}'");
        let regex = match regex::Regex::new(pattern) {
            Ok(re) => re,
            Err(e) => {
                log::warn!("Invalid regex pattern '{pattern}': {e}");
                return false;
            }
        };
        if !regex.is_match(file_name) {
            log::debug!("File name '{file_name}' does not match regex '{pattern}'");
            return false;
        }
    }

    // is_symlink
    if let Some(is_symlink) = conditions.is_symlink {
        log::debug!("Checking if file is symlink: expected {is_symlink}");
        if metadata.file_type().is_symlink() != is_symlink {
            log::debug!(
                "Symlink status does not match for file: {}",
                file_path.display()
            );
            return false;
        }
    }

    // owner (cross-platform)
    if let Some(ref owner) = conditions.owner {
        log::debug!("Checking if file owner matches '{owner}'");
        if !match_file_owner(&metadata, owner) {
            log::debug!("File owner does not match '{owner}'");
            return false;
        }
    }

    log::debug!("All conditions matched for file: {}", file_path.display());
    true
}

fn match_file_owner(metadata: &fs::Metadata, owner: &str) -> bool {
    #[cfg(unix)]
    {
        log::debug!("Matching file owner for Unix-like system: {owner}");
        use std::os::unix::fs::MetadataExt;
        let uid = metadata.uid();
        if let Some(user) = users::get_user_by_uid(uid) {
            return user.name() == owner;
        }
    }
    #[cfg(windows)]
    {
        log::warn!("Owner matching not implemented on Windows.");
        return false;
    }
    log::debug!("File owner does not match for file: {metadata:?}");
    false
}

pub fn match_rule_matcher(file_path: &PathBuf, matcher: &RuleMatch) -> bool {
    log::debug!(
        "Starting match_rule_matcher for file: {}",
        file_path.display()
    );

    // Match extensions
    if let Some(ref exts) = matcher.extensions {
        log::debug!(
            "Checking extensions {exts:?} for file: {}",
            file_path.display()
        );
        if !match_extensions(file_path, &exts.clone()) {
            log::debug!("Extension match failed for file: {}", file_path.display());
            return false;
        }
        log::debug!(
            "Extension match succeeded for file: {}",
            file_path.display()
        );
    }

    // Match MIME type
    if let Some(ref mime) = matcher.mime_type {
        log::debug!(
            "Checking MIME type '{mime}' for file: {}",
            file_path.display()
        );
        if !match_mime_type(file_path, &mime.clone()) {
            log::debug!("MIME type match failed for file: {}", file_path.display());
            return false;
        }
        log::debug!(
            "MIME type match succeeded for file: {}",
            file_path.display()
        );
    }

    // Match pattern
    if let Some(ref pattern) = matcher.pattern {
        log::debug!(
            "Checking pattern '{pattern}' for file: {}",
            file_path.display()
        );
        if !match_pattern(file_path, &pattern.clone()) {
            log::debug!("Pattern match failed for file: {}", file_path.display());
            return false;
        }
        log::debug!("Pattern match succeeded for file: {}", file_path.display());
    }

    // Match metadata
    if let Some(ref metadata) = matcher.metadata {
        log::debug!("Checking metadata for file: {}", file_path.display());
        if !match_metadata(file_path, metadata) {
            log::debug!("Metadata match failed for file: {}", file_path.display());
            return false;
        }
        log::debug!("Metadata match succeeded for file: {}", file_path.display());
    }

    // Match conditions
    if let Some(ref conditions) = matcher.conditions {
        log::debug!("Checking conditions for file: {}", file_path.display());
        if !match_conditions(file_path, conditions) {
            log::debug!("Conditions match failed for file: {}", file_path.display());
            return false;
        }
        log::debug!(
            "Conditions match succeeded for file: {}",
            file_path.display()
        );
    }

    // Match ALL sub-matchers
    if let Some(ref all_matchers) = matcher.all {
        log::debug!(
            "Checking ALL sub-matchers for file: {}",
            file_path.display()
        );
        for (i, sub_matcher) in all_matchers.iter().enumerate() {
            log::debug!(
                "Checking ALL sub-matcher {i} for file: {}",
                file_path.display()
            );
            if !match_rule_matcher(file_path, sub_matcher) {
                log::debug!(
                    "ALL sub-matcher {i} failed for file: {}",
                    file_path.display()
                );
                return false;
            }
            log::debug!(
                "ALL sub-matcher {i} succeeded for file: {}",
                file_path.display()
            );
        }
        log::debug!(
            "ALL sub-matchers succeeded for file: {}",
            file_path.display()
        );
    }

    // Match ANY sub-matchers
    if let Some(ref any_matchers) = matcher.any {
        log::debug!(
            "Checking ANY sub-matchers for file: {}",
            file_path.display()
        );
        if !any_matchers.iter().enumerate().any(|(i, sub)| {
            let res = match_rule_matcher(file_path, sub);
            log::debug!(
                "ANY sub-matcher {i} result: {res} for file: {}",
                file_path.display()
            );
            res
        }) {
            log::debug!("ANY sub-matchers failed for file: {}", file_path.display());
            return false;
        }
        log::debug!(
            "ANY sub-matchers succeeded for file: {}",
            file_path.display()
        );
    }

    log::debug!("All matchers succeeded for file: {}", file_path.display());
    true
}
