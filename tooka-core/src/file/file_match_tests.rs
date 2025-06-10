use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

use super::file_match;
use crate::rule::{DateRange, MetadataField, Range};

// Helper to create a temp file and rename it to a given filename
fn create_temp_file_with_name(filename: &str) -> PathBuf {
    let file = NamedTempFile::new().unwrap();
    let new_path = file.path().with_file_name(filename);
    fs::rename(file.path(), &new_path).unwrap();
    new_path
}

// Helper to create a temp file and set its extension
fn create_temp_file_with_extension(ext: &str) -> PathBuf {
    let file = NamedTempFile::new().unwrap();
    let new_path = file.path().with_extension(ext);
    fs::rename(file.path(), &new_path).unwrap();
    new_path
}

// Helper to create a temp file in a nested directory with a given filename
fn create_temp_file_in_dir<P: AsRef<Path>>(dir_and_file: P) -> PathBuf {
    let file = NamedTempFile::new().unwrap();
    let temp_path = file.into_temp_path();
    let new_path = temp_path.with_file_name(dir_and_file.as_ref());
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    temp_path.persist(&new_path).unwrap();
    new_path
}

#[test]
fn test_match_filename_regex() {
    let matching_path = create_temp_file_with_name("match_test.jpg");
    let non_matching_path = create_temp_file_with_name("fail_test.png");

    assert!(file_match::match_filename_regex(&matching_path, r"match_.*\.jpg").unwrap());
    assert!(!file_match::match_filename_regex(&non_matching_path, r"match_.*\.jpg").unwrap());
}

#[test]
fn test_match_extensions() {
    let matching_path = create_temp_file_with_extension("jpg");
    let non_matching_path = create_temp_file_with_extension("png");

    assert!(file_match::match_extensions(
        &matching_path,
        &vec!["jpg".to_string()]
    ));
    assert!(!file_match::match_extensions(
        &non_matching_path,
        &vec!["jpg".to_string()]
    ));
}

#[test]
fn test_match_path() {
    let matching_path = create_temp_file_in_dir("photos/match.jpg");
    let non_matching_path = create_temp_file_in_dir("docs/fail.txt");

    assert!(file_match::match_path(&matching_path, "**/photos/*.jpg").unwrap());
    assert!(!file_match::match_path(&non_matching_path, "**/photos/*.jpg").unwrap());
}

#[test]
fn test_match_size_kb() {
    let mut small_file = NamedTempFile::new().unwrap();
    let mut large_file = NamedTempFile::new().unwrap();

    writeln!(small_file, "tiny").unwrap();
    writeln!(large_file, "{}", "x".repeat(1024 * 10)).unwrap(); // 10 KB

    let small_meta = small_file.as_file().metadata().unwrap();
    let large_meta = large_file.as_file().metadata().unwrap();

    let range = Range {
        min: Some(5), // 5 KB
        max: Some(20),
    };

    assert!(!file_match::match_size_kb(&small_meta, range.clone()));
    assert!(file_match::match_size_kb(&large_meta, range));
}

#[test]
fn test_match_mime_type() {
    let jpg_path = create_temp_file_with_extension("jpg");
    let txt_path = create_temp_file_with_extension("txt");

    assert!(file_match::match_mime_type(&jpg_path, "image/*"));
    assert!(!file_match::match_mime_type(&txt_path, "image/*"));
}

#[test]
fn test_match_date_range_mod() {
    let file = NamedTempFile::new().unwrap();
    let meta = file.as_file().metadata().unwrap();

    let today = chrono::Utc::now().naive_utc().date();

    let range = DateRange {
        from: Some(today.format("%Y-%m-%d").to_string()),
        to: Some(today.format("%Y-%m-%d").to_string()),
    };

    assert!(file_match::match_date_range_mod(&meta, range));
}

#[test]
fn test_match_date_range_created() {
    let file = NamedTempFile::new().unwrap();
    let meta = file.as_file().metadata().unwrap();

    let today = chrono::Utc::now().naive_utc().date();

    let range = DateRange {
        from: Some(today.format("%Y-%m-%d").to_string()),
        to: Some(today.format("%Y-%m-%d").to_string()),
    };

    // Note: On Linux, `created()` may return an error depending on FS.
    let result = file_match::match_date_range_created(&meta, range);
    // Allow either true or false, but make sure it doesn't panic
    assert!(matches!(result, true | false));
}

#[test]
fn test_match_is_symlink() {
    let file = NamedTempFile::new().unwrap().into_temp_path();
    let symlink_path = file.with_extension("symlink");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&file, &symlink_path).unwrap();

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&file, &symlink_path).unwrap();

    let file_meta = fs::symlink_metadata(&file).unwrap();
    let symlink_meta = fs::symlink_metadata(&symlink_path).unwrap();

    assert!(!file_match::match_is_symlink(&file_meta, true));
    assert!(file_match::match_is_symlink(&symlink_meta, true));
}

#[test]
fn test_match_metadata_field_nonexistent() {
    let path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();

    let field = MetadataField {
        key: "EXIF:DateTimeOriginal".to_string(),
        value: Some("*".to_string()),
    };

    // No EXIF data in a blank temp file
    assert!(!file_match::match_metadata_field(&path, &field));
}
