use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use directories_next::UserDirs;

fn tooka_cmd(args: &[&str]) -> (String, String) {
    let tooka_bin = assert_cmd_bin();
    let output = Command::new(&tooka_bin)
        .args(args)
        .output()
        .expect("failed to execute tooka");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        panic!("Command {:?} failed\nstdout:\n{}\nstderr:\n{}", args, stdout, stderr);
    }

    (stdout, stderr)
}

fn assert_cmd_bin() -> PathBuf {
    let target_dir = Path::new("target").join("debug");
    let bin = target_dir.join("tooka");
    assert!(bin.exists(), "Binary not built: {}", bin.display());
    bin
}

fn create_test_rule(rule_id: &str) -> PathBuf {
    let rule_content = format!(
    r#"
    id: "{rule_id}"
    name: "Test Rule"
    enabled: true
    match:
    all:
        - extensions: [".txt"]
    actions:
    - type: move
        destination: "$HOME/Documents/Sorted"
        create_dirs: true
    "#);
    let path = env::temp_dir().join(format!("{rule_id}.yaml"));
    fs::write(&path, rule_content).unwrap();
    path
}

#[test]
fn test_add_and_remove_rule() {
    let rule_id = "test-add-remove";
    let rule_path = create_test_rule(rule_id);

    // Add rule
    let (out, _) = tooka_cmd(&["add", rule_path.to_str().unwrap()]);
    assert!(out.contains("✅"));

    // Remove rule
    let (out, _) = tooka_cmd(&["remove", rule_id]);
    assert!(out.contains("✅"));
}

#[test]
fn test_list_rules() {
    let rule_id = "test-list";
    let rule_path = create_test_rule(rule_id);

    tooka_cmd(&["add", rule_path.to_str().unwrap()]);
    let (out, _) = tooka_cmd(&["list"]);
    assert!(out.contains(rule_id));
    tooka_cmd(&["remove", rule_id]);
}

#[test]
fn test_toggle_rule() {
    let rule_id = "test-toggle";
    let rule_path = create_test_rule(rule_id);

    tooka_cmd(&["add", rule_path.to_str().unwrap()]);
    let (out1, _) = tooka_cmd(&["toggle", rule_id]);
    assert!(out1.contains("✅"));

    let (out2, _) = tooka_cmd(&["toggle", rule_id]);
    assert!(out2.contains("✅"));

    tooka_cmd(&["remove", rule_id]);
}

#[test]
fn test_export_rule() {
    let rule_id = "test-export";
    let rule_path = create_test_rule(rule_id);
    let export_path = env::temp_dir().join("exported_rule.yaml");

    tooka_cmd(&["add", rule_path.to_str().unwrap()]);
    tooka_cmd(&["export", rule_id, export_path.to_str().unwrap()]);
    assert!(export_path.exists());

    let exported_content = fs::read_to_string(export_path).unwrap();
    assert!(exported_content.contains(rule_id));

    tooka_cmd(&["remove", rule_id]);
}

#[test]
fn test_config_outputs() {
    let (out, _) = tooka_cmd(&["config", "--show"]);
    assert!(out.contains("rules_file") || out.contains("default_scan_dir"));
}

#[test]
fn test_sort_command() {
    let rule_id = "test-sort";
    let rule_path = create_test_rule(rule_id);
    tooka_cmd(&["add", rule_path.to_str().unwrap()]);

    // Create dummy file in Downloads
    let user_dirs = UserDirs::new().expect("Failed to get user directories");
    let downloads = user_dirs
        .download_dir()
        .expect("No download directory found");
    let dummy_file = downloads.join("sort_test_file.txt");
    let mut f = File::create(&dummy_file).unwrap();
    writeln!(f, "Dummy sort content").unwrap();
    assert!(dummy_file.exists());

    // Dry run
    let (dry_out, _) = tooka_cmd(&["sort", "--dry-run"]);
    assert!(dry_out.contains("sort_test_file.txt"));

    // Real run
    let (_real_out, _) = tooka_cmd(&["sort"]);
    let sorted_file = UserDirs::new()
        .expect("Failed to get user directories")
        .document_dir()
        .expect("No document directory found")
        .join("Sorted/sort_test_file.txt");

    assert!(sorted_file.exists());

    // Cleanup
    fs::remove_file(sorted_file).ok();
    tooka_cmd(&["remove", rule_id]);
}
