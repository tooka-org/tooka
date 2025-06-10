use crate::core::error::TookaError;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use exif::{In, Reader, Tag};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Evaluates a template string with metadata and file information.
pub(crate) fn evaluate_template(
    template: &str,
    file_path: &Path,
    metadata: &HashMap<String, String>,
) -> Result<String, TookaError> {
    let re = Regex::new(r"\{\{(.*?)\}\}").unwrap();
    let file_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let mut result = template.to_string();

    for caps in re.captures_iter(template) {
        let full_match = &caps[0];
        let expr = &caps[1];

        let mut parts = expr.split('|');
        let key = parts.next().unwrap().trim();
        let filters: Vec<&str> = parts.collect();

        let raw_value = if key == "filename" {
            file_name.clone()
        } else if let Some(metadata_key) = key.strip_prefix("metadata.") {
            metadata
                .get(metadata_key)
                .cloned()
                .unwrap_or_else(String::new)
        } else {
            String::new()
        };

        let final_value = apply_filters(raw_value, &filters);
        result = result.replace(full_match, &final_value);
    }

    Ok(result)
}

fn apply_filters(value: String, filters: &[&str]) -> String {
    let mut val = value;
    for filter in filters {
        if let Some(fmt) = filter.strip_prefix("date:") {
            let parsed = DateTime::parse_from_rfc3339(&val)
                .map(|dt| dt.with_timezone(&Local))
                .or_else(|_| {
                    NaiveDateTime::parse_from_str(&val, "%Y:%m:%d %H:%M:%S")
                        .map(|dt| Local.from_local_datetime(&dt).unwrap())
                });

            if let Ok(datetime) = parsed {
                val = datetime.format(fmt).to_string();
            }
        }
    }
    val
}

/// Returns metadata fields for use in templating
pub(crate) fn extract_metadata(file_path: &Path) -> Result<HashMap<String, String>, TookaError> {
    let mut map = HashMap::new();

    // File system metadata
    let metadata = fs::metadata(file_path)?;

    if let Ok(modified) = metadata.modified() {
        let dt: DateTime<Local> = modified.into();
        map.insert("modified".into(), dt.to_rfc3339());
    }

    if let Ok(created) = metadata.created() {
        let dt: DateTime<Local> = created.into();
        map.insert("created".into(), dt.to_rfc3339());
    }

    map.insert("size".into(), metadata.len().to_string());

    // Attempt to parse EXIF (for JPEG/PNG)
    if let Ok(file) = fs::File::open(file_path) {
        if let Ok(reader) = Reader::new().read_from_container(&mut std::io::BufReader::new(file)) {
            for field in reader.fields() {
                let tag = format!("{:?}", field.tag);
                let ifd = format!("{:?}", field.ifd_num);
                let key = format!("EXIF:{}:{}", ifd, tag); // e.g., EXIF:In:DateTime
                let value = field.display_value().with_unit(&reader).to_string();
                map.insert(key, value);
            }

            // Common aliases for convenience
            if let Some(field) = reader.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
                map.insert("EXIF:DateTime".into(), field.display_value().to_string());
            }
        }
    }

    Ok(map)
}
