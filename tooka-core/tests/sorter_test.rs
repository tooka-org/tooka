#[cfg(test)]
mod tests {
    use std::{
        env,
        fs::{self, File},
        io::Write,
        path::{Path, PathBuf},
    };
    use tempfile::tempdir;
    use tooka_core::{
        report::generate_report,
        error::TookaError,
        rule::{Action, Conditions, DeleteAction, MoveAction, Range, Rule},
        rules_file::RulesFile,
        sorter::{MatchResult, sort_files},
    };

    fn create_temp_files(base_dir: &Path) -> Vec<PathBuf> {
        let files_info = vec![
            ("document.txt", "txt"),
            ("logfile.log", "log"),
            ("binary.bin", "bin"),
            ("random.data", "data"),
            ("note.unknown", "unknown"),
        ];

        let mut paths = vec![];

        for (name, ext) in files_info.iter() {
            let path = base_dir.join(name);
            let mut file = File::create(&path).unwrap();
            file.write_all(b"example content").unwrap();
            assert_eq!(path.extension().unwrap(), *ext);
            paths.push(path);
        }

        paths
    }

    fn create_dummy_rules(dest: &Path) -> RulesFile {
        let extensions = ["txt", "log", "data", "bin"];
        let mut rules = vec![];

        for ext in extensions {
            let to = dest.join(format!("out_{}", ext));
            fs::create_dir_all(&to).unwrap();

            rules.push(Rule {
                id: format!("rule_{}", ext),
                name: format!("Rule for .{}", ext),
                enabled: true,
                description: None,
                priority: 5,
                when: Conditions {
                    any: Some(false),
                    filename: Some(format!(r".*\.{}$", ext)),
                    extensions: Some(vec![ext.to_string()]),
                    path: None,
                    size_kb: Some(Range {
                        min: Some(0),
                        max: Some(1000),
                    }),
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![Action::Move(MoveAction {
                    to: to.to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            });
        }

        RulesFile { rules }
    }

    #[test]
    fn test_sort_files_with_rules() -> Result<(), TookaError> {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        let test_files = create_temp_files(base_path);
        let rules = create_dummy_rules(base_path);

        let results: Vec<MatchResult> = sort_files(
            test_files.clone(),
            base_path.to_path_buf(),
            &rules,
            true,
            None::<fn()>,
        )?;

        // We expect all 5 files to be processed: 4 matched, 1 skipped
        assert_eq!(results.len(), 5);

        let mut matched = 0;
        let mut skipped = 0;

        for result in results {
            if result.matched_rule_id == "none" {
                skipped += 1;
                assert_eq!(result.action, "skip");
            } else {
                matched += 1;
                let ext = result
                    .file_name
                    .split('.')
                    .last()
                    .expect("File should have extension");
                assert_eq!(result.action, "move");
                assert!(result.matched_rule_id.contains(ext));
                assert!(
                    result
                        .new_path
                        .starts_with(base_path.join(format!("out_{}", ext)))
                );
            }
        }

        assert_eq!(matched, 4);
        assert_eq!(skipped, 1);

        Ok(())
    }

    #[test]
    fn test_rule_priority_with_multiple_matches() -> Result<(), TookaError> {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // Create one test file
        let file_path = base_path.join("important.log");
        File::create(&file_path)?.write_all(b"content")?;

        // Create two matching rules for .log
        let rules = vec![
            Rule {
                id: "high_priority".to_string(),
                name: "High priority rule".to_string(),
                enabled: true,
                description: None,
                priority: 10,
                when: Conditions {
                    any: Some(false),
                    filename: Some(r".*\.log$".to_string()),
                    extensions: Some(vec!["log".to_string()]),
                    path: None,
                    size_kb: None,
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![Action::Move(MoveAction {
                    to: base_path.join("high").to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            },
            Rule {
                id: "low_priority".to_string(),
                name: "Low priority rule".to_string(),
                enabled: true,
                description: None,
                priority: 1,
                when: Conditions {
                    any: Some(false),
                    filename: Some(r".*\.log$".to_string()),
                    extensions: Some(vec!["log".to_string()]),
                    path: None,
                    size_kb: None,
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![Action::Move(MoveAction {
                    to: base_path.join("low").to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            },
        ];

        // Make sure target directories exist
        fs::create_dir_all(base_path.join("high"))?;
        fs::create_dir_all(base_path.join("low"))?;

        let rules_file = RulesFile { rules };

        let results = sort_files(
            vec![file_path.clone()],
            base_path.to_path_buf(),
            &rules_file,
            true,
            None::<fn()>,
        )?;
        assert_eq!(results.len(), 1);

        let result = &results[0];
        assert_eq!(result.matched_rule_id, "high_priority");
        assert!(result.new_path.starts_with(base_path.join("high")));

        Ok(())
    }

    #[test]
    fn test_rule_with_multiple_actions() -> Result<(), TookaError> {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // Create one test file
        let file_path = base_path.join("to_delete.txt");
        File::create(&file_path)?.write_all(b"content")?;

        let target_dir = base_path.join("moved");
        fs::create_dir_all(&target_dir)?;

        let rules = RulesFile {
            rules: vec![Rule {
                id: "multi_action".to_string(),
                name: "Move and delete".to_string(),
                enabled: true,
                description: None,
                priority: 5,
                when: Conditions {
                    any: Some(false),
                    filename: Some(r".*\.txt$".to_string()),
                    extensions: Some(vec!["txt".to_string()]),
                    path: None,
                    size_kb: None,
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![
                    Action::Move(MoveAction {
                        to: target_dir.to_string_lossy().to_string(),
                        preserve_structure: false,
                    }),
                    Action::Delete(DeleteAction { trash: false }),
                ],
            }],
        };

        let results = sort_files(
            vec![file_path.clone()],
            base_path.to_path_buf(),
            &rules,
            true,
            None::<fn()>,
        )?;
        assert_eq!(results.len(), 2); // One for move, one for delete

        assert_eq!(results[0].action, "move");
        assert!(results[0].new_path.starts_with(&target_dir));

        assert_eq!(results[1].action, "delete");

        Ok(())
    }

    #[test]
    fn test_sort_and_generate_pdf_report() -> Result<(), TookaError> {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // Generate 30+ files with predictable extensions
        let extensions = ["txt", "log", "bin"];
        let mut test_files = vec![];
        for i in 0..30 {
            let ext = extensions[i % extensions.len()];
            let name = format!("file_{:02}.{}", i, ext);
            let path = base_path.join(&name);
            let mut file = File::create(&path).unwrap();
            writeln!(file, "Content for {}", name).unwrap();
            test_files.push(path);
        }

        // Create matching rules
        let mut rules = vec![];
        for ext in extensions {
            let target = base_path.join(format!("target_{}", ext));
            fs::create_dir_all(&target).unwrap();

            rules.push(Rule {
                id: format!("rule_{}", ext),
                name: format!("Rule for .{}", ext),
                enabled: true,
                description: Some(format!("Handles .{} files", ext)),
                priority: 1,
                when: Conditions {
                    any: Some(false),
                    filename: Some(format!(r".*\.{}$", ext)),
                    extensions: Some(vec![ext.to_string()]),
                    path: None,
                    size_kb: None,
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![Action::Move(MoveAction {
                    to: target.to_string_lossy().to_string(),
                    preserve_structure: false,
                }),
                Action::Delete(DeleteAction { trash: false }),
                ],
            });
        }

        let rules_file = RulesFile { rules };

        // Perform sort
        let results = sort_files(
            test_files.clone(),
            base_path.to_path_buf(),
            &rules_file,
            true,
            None::<fn()>,
        )?;

        // Output PDF to current working directory
        let output_dir = env::current_dir().unwrap();
        generate_report("pdf", &output_dir, &results)?;

        // Confirm the file exists and has size > 0
        let output_path = output_dir.join("tooka_report.pdf");
        assert!(output_path.exists());
        let metadata = fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 0);

        println!("✅ PDF report written to: {}", output_path.display());
        // Clean up the generated pdf report
        // Note: REMOVE THIS LINE TO KEEP THE REPORT
        fs::remove_file(output_path).unwrap_or_else(|_| {
            eprintln!("Failed to remove the generated PDF report.");
        });
        Ok(())
    }
}
