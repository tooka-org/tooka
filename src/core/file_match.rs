use std::path::{Path, PathBuf};
use chrono::Utc;
use crate::core::rules;
use crate::core::rules::{Match as RuleMatch};
use std::fs;
use glob;

fn match_extensions(file_path: &Path, extensions: Vec<String>) -> bool {
    if let Some(extension) = file_path.extension() {
        if let Some(extension_str) = extension.to_str() {
            return extensions.iter().any(|ext| ext == extension_str);
        }
    }
    false
}

fn match_mime_type(file_path: &PathBuf, mime_type: String) -> bool {
    if let Some(mime) = mime_guess::from_path(file_path).first() {
        let mime_essence = mime.essence_str();
        if let Some(idx) = mime_type.find("/*") {
            // Handle wildcard, e.g., "image/*"
            let prefix = &mime_type[..idx];
            return mime_essence.starts_with(prefix) && mime_essence.chars().nth(prefix.len()) == Some('/');
        } else {
            // Exact match
            return mime_essence == mime_type;
        }
    }
    false
}

fn match_pattern(file_path: &Path, pattern: String) -> bool {
    let file_name = file_path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let regex = match regex::Regex::new(&pattern) {
        Ok(re) => re,
        Err(_) => return false, // Invalid regex pattern
    };
    regex.is_match(file_name)
}

fn match_metadata(file_path: &PathBuf, metadata_match: &rules::MetadataMatch) -> bool {
    // Check EXIF date if requested
    if metadata_match.exif_date {
        if let Ok(file) = fs::File::open(file_path) {
            let mut bufreader = std::io::BufReader::new(file);
            if let Ok(exif) = exif::Reader::new().read_from_container(&mut bufreader) {
                if exif.get_field(exif::Tag::DateTime, exif::In::PRIMARY).is_none() {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        }
    }

    // Check additional metadata fields
    for field in &metadata_match.fields {
            if !match_metadata_field(file_path, field) {
                return false;
            }
    }

    true
}

fn match_metadata_field(file_path: &PathBuf, field: &rules::MetadataField) -> bool {
    // Try to read EXIF metadata and match the field
    if let Ok(file) = fs::File::open(file_path) {
        let mut bufreader = std::io::BufReader::new(file);
        if let Ok(exif) = exif::Reader::new().read_from_container(&mut bufreader) {
            // Try to find the field by key
            for f in exif.fields() {
                let tag_name = format!("EXIF:{:?}", f.tag);
                if tag_name == field.key {
                    let value = f.display_value().with_unit(&exif).to_string();
                    // Use glob pattern matching for the pattern
                    if let Some(ref pattern_str) = field.pattern {
                        if let Ok(pattern) = glob::Pattern::new(pattern_str) {
                            if pattern.matches(&value) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn match_conditions(file_path: &PathBuf, conditions: &rules::Conditions) -> bool {
    let metadata = match fs::symlink_metadata(file_path) {
        Ok(m) => m,
        Err(_) => return false,
    };

    // older_than_days
    if let Some(days) = conditions.older_than_days {
        if let Ok(modified) = metadata.modified() {
            let modified_datetime: chrono::DateTime<Utc> = modified.into();
            let age = Utc::now().signed_duration_since(modified_datetime);
            if age.num_days() < days as i64 {
                return false;
            }
        } else {
            return false;
        }
    }

    // size_greater_than_kb
    if let Some(min_kb) = conditions.size_greater_than_kb {
        if metadata.len() < min_kb * 1024 {
            return false;
        }
    }

    // created_between
    if let Some(ref range) = conditions.created_between {
        if let Ok(created) = metadata.created() {
            let created_date = chrono::DateTime::<Utc>::from(created).date_naive();
            let from_date = match chrono::NaiveDate::parse_from_str(&range.from, "%Y-%m-%d") {
                Ok(date) => date,
                Err(_) => return false,
            };
            let to_date = match chrono::NaiveDate::parse_from_str(&range.to, "%Y-%m-%d") {
                Ok(date) => date,
                Err(_) => return false,
            };
            if created_date < from_date || created_date > to_date {
                return false;
            }
        } else {
            return false;
        }
    }

    // filename_regex
    if let Some(ref pattern) = conditions.filename_regex {
        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let regex = match regex::Regex::new(pattern) {
            Ok(re) => re,
            Err(_) => return false,
        };
        if !regex.is_match(file_name) {
            return false;
        }
    }

    // is_symlink
    if let Some(is_symlink) = conditions.is_symlink {
        if metadata.file_type().is_symlink() != is_symlink {
            return false;
        }
    }

    // owner (cross-platform)
    if let Some(ref owner) = conditions.owner {
        if !match_file_owner(&metadata, owner) {
            return false;
        }
    }

    true
}

fn match_file_owner(metadata: &fs::Metadata, owner: &str) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let uid = metadata.uid();
        if let Some(user) = users::get_user_by_uid(uid) {
            return user.name() == owner;
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let sid = metadata.sid();
        if let Ok(user) = users::get_user_by_sid(sid) {
            return user.name() == owner;
        }
    }
    false
}

pub fn match_rule_matcher(file_path: &PathBuf, matcher: &RuleMatch) -> bool {
    // Match extensions
    if let Some(ref exts) = matcher.extensions {
        if !match_extensions(file_path, exts.clone()) {
            return false;
        }
    }

    // Match MIME type
    if let Some(ref mime) = matcher.mime_type {
        if !match_mime_type(file_path, mime.clone()) {
            return false;
        }
    }

    // Match pattern
    if let Some(ref pattern) = matcher.pattern {
        if !match_pattern(file_path, pattern.clone()) {
            return false;
        }
    }

    // Match metadata
    if let Some(ref metadata) = matcher.metadata {
        if !match_metadata(file_path, metadata) {
            return false;
        }
    }

    // Match conditions
    if let Some(ref conditions) = matcher.conditions {
        if !match_conditions(file_path, conditions) {
            return false;
        }
    }

    // Match ALL sub-matchers
    if let Some(ref all_matchers) = matcher.all {
        for sub_matcher in all_matchers {
            if !match_rule_matcher(file_path, sub_matcher) {
                return false;
            }
        }
    }

    // Match ANY sub-matchers
    if let Some(ref any_matchers) = matcher.any {
        if !any_matchers.iter().any(|sub| match_rule_matcher(file_path, sub)) {
            return false;
        }
    }

    true
}