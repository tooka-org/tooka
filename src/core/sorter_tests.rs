#[cfg(test)]
mod tests {
    use crate::core::error::TookaError;
    use crate::core::sorter::{collect_files, sort_files, MatchResult};
    use crate::rules::rule::{Action, Conditions, CopyAction, MoveAction, Rule};
    use crate::rules::rules_file::RulesFile;
    use crate::utils::gen_pdf::generate_pdf;
    use std::fs::{File, create_dir_all};
    use std::io::Write;
    use tempfile::tempdir;

    /// Helper function to create a test file with content
    fn create_test_file(path: &std::path::Path, content: &str) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Helper function to create multiple test files with different extensions
    fn create_test_files(base_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        let test_data = [
            ("test1.txt", "text content"),
            ("test2.log", "log content"),
            ("test3.data", "data content"),
            ("test4.bin", "binary content"),
            ("test5.unknown", "unknown content"),
        ];

        for (filename, content) in test_data {
            let file_path = base_dir.join(filename);
            create_test_file(&file_path, content).unwrap();
            files.push(file_path);
        }

        files
    }

    /// Helper function to create test rules
    fn create_test_rules(dest_dir: &std::path::Path) -> RulesFile {
        let txt_dir = dest_dir.join("txt_files");
        let log_dir = dest_dir.join("log_files");
        let data_dir = dest_dir.join("data_files");

        create_dir_all(&txt_dir).unwrap();
        create_dir_all(&log_dir).unwrap();
        create_dir_all(&data_dir).unwrap();

        let rules = vec![
            Rule {
                id: "txt_rule".to_string(),
                name: "Move txt files".to_string(),
                enabled: true,
                description: Some("Move all .txt files to txt_files directory".to_string()),
                priority: 1,
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
                then: vec![Action::Move(MoveAction {
                    to: txt_dir.to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            },
            Rule {
                id: "log_rule".to_string(),
                name: "Copy log files".to_string(),
                enabled: true,
                description: Some("Copy all .log files to log_files directory".to_string()),
                priority: 2,
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
                then: vec![Action::Copy(CopyAction {
                    to: log_dir.to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            },
            Rule {
                id: "data_rule".to_string(),
                name: "Move data files".to_string(),
                enabled: true,
                description: Some("Move all .data files to data_files directory".to_string()),
                priority: 3,
                when: Conditions {
                    any: Some(false),
                    filename: Some(r".*\.data$".to_string()),
                    extensions: Some(vec!["data".to_string()]),
                    path: None,
                    size_kb: None,
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![Action::Move(MoveAction {
                    to: data_dir.to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            },
        ];

        RulesFile { rules }
    }

    #[test]
    fn test_sort_files_basic() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create test files
        let files = create_test_files(&source_path);

        // Create rules
        let rules_file = create_test_rules(&source_path);

        // Sort files in dry run mode
        let results = sort_files(
            &files,
            &source_path,
            &rules_file,
            true,
            None::<fn()>,
        )
        .expect("sort_files should succeed");

        // Check that we got results for all files
        assert_eq!(results.len(), files.len());

        // Check that txt file was matched by txt_rule
        let txt_result = results.iter().find(|r| r.file_name == "test1.txt").unwrap();
        assert_eq!(txt_result.matched_rule_id, "txt_rule");
        assert_eq!(txt_result.action, "move");

        // Check that log file was matched by log_rule
        let log_result = results.iter().find(|r| r.file_name == "test2.log").unwrap();
        assert_eq!(log_result.matched_rule_id, "log_rule");
        assert_eq!(log_result.action, "copy");

        // Check that data file was matched by data_rule
        let data_result = results
            .iter()
            .find(|r| r.file_name == "test3.data")
            .unwrap();
        assert_eq!(data_result.matched_rule_id, "data_rule");
        assert_eq!(data_result.action, "move");

        // Check that files without matching rules are skipped
        let unknown_result = results
            .iter()
            .find(|r| r.file_name == "test5.unknown")
            .unwrap();
        assert_eq!(unknown_result.matched_rule_id, "none");
        assert_eq!(unknown_result.action, "skip");
    }

    #[test]
    fn test_sort_files_actual_execution() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create test files
        let files = create_test_files(&source_path);

        // Create rules
        let rules_file = create_test_rules(&source_path);

        // Sort files with actual execution (not dry run)
        let results = sort_files(
            &files,
            &source_path,
            &rules_file,
            false,
            None::<fn()>,
        )
        .expect("sort_files should succeed");

        // Check that txt file was moved
        let txt_result = results.iter().find(|r| r.file_name == "test1.txt").unwrap();
        assert!(
            txt_result.new_path.exists(),
            "txt file should be moved to new location"
        );
        assert!(
            !source_path.join("test1.txt").exists(),
            "txt file should be moved from original location"
        );

        // Check that log file was copied (original should still exist)
        let log_result = results.iter().find(|r| r.file_name == "test2.log").unwrap();
        assert!(
            log_result.new_path.exists(),
            "log file should be copied to new location"
        );
        assert!(
            source_path.join("test2.log").exists(),
            "log file should still exist in original location"
        );

        // Check that data file was moved
        let data_result = results
            .iter()
            .find(|r| r.file_name == "test3.data")
            .unwrap();
        assert!(
            data_result.new_path.exists(),
            "data file should be moved to new location"
        );
        assert!(
            !source_path.join("test3.data").exists(),
            "data file should be moved from original location"
        );
    }

    #[test]
    fn test_sort_files_with_priority() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create a test file that could match multiple rules
        let test_file = source_path.join("test.txt");
        create_test_file(&test_file, "test content").unwrap();

        // Create rules with different priorities
        let high_priority_dir = source_path.join("high_priority");
        let low_priority_dir = source_path.join("low_priority");
        create_dir_all(&high_priority_dir).unwrap();
        create_dir_all(&low_priority_dir).unwrap();

        let rules = vec![
            Rule {
                id: "low_priority_rule".to_string(),
                name: "Low priority rule".to_string(),
                enabled: true,
                description: None,
                priority: 1, // Lower priority (lower number)
                when: Conditions {
                    any: Some(false),
                    filename: Some(r".*\.txt$".to_string()),
                    extensions: None,
                    path: None,
                    size_kb: None,
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![Action::Move(MoveAction {
                    to: low_priority_dir.to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            },
            Rule {
                id: "high_priority_rule".to_string(),
                name: "High priority rule".to_string(),
                enabled: true,
                description: None,
                priority: 10, // Higher priority (higher number)
                when: Conditions {
                    any: Some(false),
                    filename: Some(r".*\.txt$".to_string()),
                    extensions: None,
                    path: None,
                    size_kb: None,
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![Action::Move(MoveAction {
                    to: high_priority_dir.to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            },
        ];

        let rules_file = RulesFile { rules };
        let optimized_rules = rules_file.optimized_with_filter(None).unwrap();

        // Sort the file
        let results = sort_files(
            &[test_file],
            &source_path,
            &optimized_rules,
            true,
            None::<fn()>,
        )
        .expect("sort_files should succeed");

        // Should match the high priority rule
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matched_rule_id, "high_priority_rule");
    }

    #[test]
    fn test_sort_files_with_progress_callback() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create test files
        let files = create_test_files(&source_path);
        let rules_file = create_test_rules(&source_path);

        // Track progress
        let progress_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let progress_count_clone = progress_count.clone();

        let progress_callback = move || {
            progress_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        };

        // Sort files with progress callback
        let results = sort_files(
            &files,
            &source_path,
            &rules_file,
            true,
            Some(progress_callback),
        )
        .expect("sort_files should succeed");

        // Check that progress callback was called for each file
        assert_eq!(
            progress_count.load(std::sync::atomic::Ordering::SeqCst),
            files.len()
        );
        assert_eq!(results.len(), files.len());
    }

    #[test]
    fn test_sort_files_multiple_actions() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create a test file
        let test_file = source_path.join("test.txt");
        create_test_file(&test_file, "test content").unwrap();

        // Create directories for multiple actions
        let copy_dir = source_path.join("copy_dest");
        let move_dir = source_path.join("move_dest");
        create_dir_all(&copy_dir).unwrap();
        create_dir_all(&move_dir).unwrap();

        // Create rule with multiple actions
        let rules = vec![Rule {
            id: "multi_action_rule".to_string(),
            name: "Multi action rule".to_string(),
            enabled: true,
            description: None,
            priority: 1,
            when: Conditions {
                any: Some(false),
                filename: Some(r".*\.txt$".to_string()),
                extensions: None,
                path: None,
                size_kb: None,
                mime_type: None,
                created_date: None,
                modified_date: None,
                is_symlink: None,
                metadata: None,
            },
            then: vec![
                Action::Copy(CopyAction {
                    to: copy_dir.to_string_lossy().to_string(),
                    preserve_structure: false,
                }),
                Action::Move(MoveAction {
                    to: move_dir.to_string_lossy().to_string(),
                    preserve_structure: false,
                }),
            ],
        }];

        let rules_file = RulesFile { rules };

        // Sort the file
        let results = sort_files(
            &[test_file],
            &source_path,
            &rules_file,
            true,
            None::<fn()>,
        )
        .expect("sort_files should succeed");

        // Should have two results for the two actions
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].action, "copy");
        assert_eq!(results[1].action, "move");
        assert_eq!(results[0].matched_rule_id, "multi_action_rule");
        assert_eq!(results[1].matched_rule_id, "multi_action_rule");
    }

    #[test]
    fn test_collect_files() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path();

        // Create test files in subdirectories
        let sub_dir = source_path.join("subdir");
        create_dir_all(&sub_dir).unwrap();

        let files = [
            source_path.join("file1.txt"),
            source_path.join("file2.log"),
            sub_dir.join("file3.data"),
        ];

        for file in &files {
            create_test_file(file, "content").unwrap();
        }

        // Collect files
        let collected = collect_files(source_path).expect("collect_files should succeed");

        // Should find all files
        assert_eq!(collected.len(), 3);

        // Check that all created files are found
        for file in &files {
            assert!(
                collected.contains(file),
                "Should find file: {}",
                file.display()
            );
        }
    }

    #[test]
    fn test_collect_files_nonexistent_directory() {
        let temp_dir = tempdir().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");

        // Should return an error for nonexistent directory
        let result = collect_files(&nonexistent_path);
        assert!(result.is_err());

        if let Err(TookaError::ConfigError(msg)) = result {
            assert!(msg.contains("does not exist"));
        } else {
            panic!("Expected ConfigError");
        }
    }

    #[test]
    fn test_sort_files_empty_file_list() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();
        let rules_file = create_test_rules(&source_path);

        // Sort empty file list
        let results = sort_files(&[], &source_path, &rules_file, true, None::<fn()>)
            .expect("sort_files should succeed with empty list");

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_sort_files_disabled_rules() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create test file
        let test_file = source_path.join("test.txt");
        create_test_file(&test_file, "test content").unwrap();

        // Create disabled rule
        let rules = vec![Rule {
            id: "disabled_rule".to_string(),
            name: "Disabled rule".to_string(),
            enabled: false, // Disabled
            description: None,
            priority: 1,
            when: Conditions {
                any: Some(false),
                filename: Some(r".*\.txt$".to_string()),
                extensions: None,
                path: None,
                size_kb: None,
                mime_type: None,
                created_date: None,
                modified_date: None,
                is_symlink: None,
                metadata: None,
            },
            then: vec![Action::Move(MoveAction {
                to: source_path.join("dest").to_string_lossy().to_string(),
                preserve_structure: false,
            })],
        }];

        let rules_file = RulesFile { rules };

        // optimized_with_filter should fail when no enabled rules exist
        let result = rules_file.optimized_with_filter(None);
        assert!(result.is_err());

        if let Err(TookaError::RuleNotFound(msg)) = result {
            assert!(msg.contains("No enabled rules found"));
        } else {
            panic!("Expected RuleNotFound error");
        }
    }

    #[test]
    fn test_sort_files_mixed_enabled_disabled_rules() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create test file
        let test_file = source_path.join("test.txt");
        create_test_file(&test_file, "test content").unwrap();

        // Create destination directories
        let enabled_dir = source_path.join("enabled_dest");
        let disabled_dir = source_path.join("disabled_dest");
        create_dir_all(&enabled_dir).unwrap();
        create_dir_all(&disabled_dir).unwrap();

        // Create mixed enabled/disabled rules
        let rules = vec![
            Rule {
                id: "disabled_rule".to_string(),
                name: "Disabled rule".to_string(),
                enabled: false, // Disabled
                description: None,
                priority: 10, // Higher priority but disabled
                when: Conditions {
                    any: Some(false),
                    filename: Some(r".*\.txt$".to_string()),
                    extensions: None,
                    path: None,
                    size_kb: None,
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![Action::Move(MoveAction {
                    to: disabled_dir.to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            },
            Rule {
                id: "enabled_rule".to_string(),
                name: "Enabled rule".to_string(),
                enabled: true, // Enabled
                description: None,
                priority: 5, // Lower priority but enabled
                when: Conditions {
                    any: Some(false),
                    filename: Some(r".*\.txt$".to_string()),
                    extensions: None,
                    path: None,
                    size_kb: None,
                    mime_type: None,
                    created_date: None,
                    modified_date: None,
                    is_symlink: None,
                    metadata: None,
                },
                then: vec![Action::Move(MoveAction {
                    to: enabled_dir.to_string_lossy().to_string(),
                    preserve_structure: false,
                })],
            },
        ];

        let rules_file = RulesFile { rules };
        let optimized_rules = rules_file.optimized_with_filter(None).unwrap();

        // Sort the file
        let results = sort_files(
            &[test_file.clone()],
            &source_path,
            &optimized_rules,
            true,
            None::<fn()>,
        )
        .expect("sort_files should succeed");

        // Should match only the enabled rule
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matched_rule_id, "enabled_rule");
        assert_eq!(results[0].action, "move");
        assert!(
            results[0]
                .new_path
                .to_string_lossy()
                .contains("enabled_dest")
        );
    }

    #[test]
    fn test_pdf_generation() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create test files
        let files = create_test_files(&source_path);
        let rules_file = create_test_rules(&source_path);

        // Sort files to get MatchResults
        let results = sort_files(
            &files,
            &source_path,
            &rules_file,
            true, // dry run
            None::<fn()>,
        )
        .expect("sort_files should succeed");

        // Ensure we have some results to generate a PDF from
        assert!(!results.is_empty(), "Should have some match results");

        // Generate PDF
        let pdf_path = temp_dir.path().join("test_report.pdf");
        generate_pdf(&pdf_path, &results).expect("PDF generation should succeed");

        // Verify PDF file was created
        assert!(pdf_path.exists(), "PDF file should be created");

        // Check that the PDF file has reasonable size (not empty)
        let metadata = std::fs::metadata(&pdf_path).expect("Should be able to read PDF metadata");
        assert!(metadata.len() > 500, "PDF should have reasonable size (> 500 bytes), actual size: {} bytes", metadata.len());

        // Print the path for manual inspection
        println!("Generated PDF at: {}", pdf_path.display());
        println!("PDF size: {} bytes", metadata.len());
        println!("Match results count: {}", results.len());

        // Verify the results contain expected data
        let txt_result = results.iter().find(|r| r.file_name == "test1.txt");
        let log_result = results.iter().find(|r| r.file_name == "test2.log");
        let data_result = results.iter().find(|r| r.file_name == "test3.data");
        let unknown_result = results.iter().find(|r| r.file_name == "test5.unknown");

        assert!(txt_result.is_some(), "Should have txt result");
        assert!(log_result.is_some(), "Should have log result");
        assert!(data_result.is_some(), "Should have data result");
        assert!(unknown_result.is_some(), "Should have unknown result");

        if let Some(txt) = txt_result {
            assert_eq!(txt.action, "move");
            assert_eq!(txt.matched_rule_id, "txt_rule");
        }

        if let Some(log) = log_result {
            assert_eq!(log.action, "copy");
            assert_eq!(log.matched_rule_id, "log_rule");
        }

        if let Some(data) = data_result {
            assert_eq!(data.action, "move");
            assert_eq!(data.matched_rule_id, "data_rule");
        }

        if let Some(unknown) = unknown_result {
            assert_eq!(unknown.action, "skip");
            assert_eq!(unknown.matched_rule_id, "none");
        }
    }

    #[test]
    fn test_pdf_generation_with_large_dataset() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().to_path_buf();

        // Create multiple results to test pagination and grouping
        let mut mock_results = Vec::new();
        
        // Create results for multiple rules with different actions
        for i in 0..50 {
            mock_results.push(MatchResult {
                file_name: format!("file{}.txt", i),
                current_path: source_path.join(format!("file{}.txt", i)),
                new_path: source_path.join("txt_files").join(format!("file{}.txt", i)),
                matched_rule_id: "txt_rule".to_string(),
                action: "move".to_string(),
            });
        }

        for i in 0..30 {
            mock_results.push(MatchResult {
                file_name: format!("log{}.log", i),
                current_path: source_path.join(format!("log{}.log", i)),
                new_path: source_path.join("log_files").join(format!("log{}.log", i)),
                matched_rule_id: "log_rule".to_string(),
                action: "copy".to_string(),
            });
        }

        for i in 0..20 {
            mock_results.push(MatchResult {
                file_name: format!("data{}.data", i),
                current_path: source_path.join(format!("data{}.data", i)),
                new_path: source_path.join("data_files").join(format!("data{}.data", i)),
                matched_rule_id: "data_rule".to_string(),
                action: "move".to_string(),
            });
        }

        for i in 0..10 {
            mock_results.push(MatchResult {
                file_name: format!("executable{}.exe", i),
                current_path: source_path.join(format!("executable{}.exe", i)),
                new_path: source_path.join("executed").join(format!("executed_{}.exe", i)),
                matched_rule_id: "execute_rule".to_string(),
                action: "execute".to_string(),
            });
        }

        for i in 0..15 {
            mock_results.push(MatchResult {
                file_name: format!("unknown{}.unknown", i),
                current_path: source_path.join(format!("unknown{}.unknown", i)),
                new_path: source_path.join(format!("unknown{}.unknown", i)), // Same path for skip
                matched_rule_id: "none".to_string(),
                action: "skip".to_string(),
            });
        }

        // Generate PDF with large dataset
        let pdf_path = temp_dir.path().join("large_test_report.pdf");
        generate_pdf(&pdf_path, &mock_results).expect("PDF generation should succeed with large dataset");

        // Verify PDF file was created
        assert!(pdf_path.exists(), "Large PDF file should be created");

        // Check that the PDF file has reasonable size
        let metadata = std::fs::metadata(&pdf_path).expect("Should be able to read large PDF metadata");
        assert!(metadata.len() > 5000, "Large PDF should have substantial size (> 5KB), actual size: {} bytes", metadata.len());

        println!("Generated large PDF at: {}", pdf_path.display());
        println!("Large PDF size: {} bytes", metadata.len());
        println!("Total match results: {}", mock_results.len());

        // Verify we have the expected number of results
        assert_eq!(mock_results.len(), 125); // 50+30+20+10+15

        // Verify different action types are present
        let move_count = mock_results.iter().filter(|r| r.action == "move").count();
        let copy_count = mock_results.iter().filter(|r| r.action == "copy").count();
        let execute_count = mock_results.iter().filter(|r| r.action == "execute").count();
        let skip_count = mock_results.iter().filter(|r| r.action == "skip").count();

        assert_eq!(move_count, 70); // txt + data files
        assert_eq!(copy_count, 30); // log files
        assert_eq!(execute_count, 10); // executable files
        assert_eq!(skip_count, 15); // unknown files
    }

    #[test]
    fn test_pdf_generation_for_inspection() {
        // Create PDF in the project directory for easy inspection
        let pdf_path = std::path::Path::new("test_report_refactored.pdf");
        
        // Create mock results with various actions to test all features
        let mut mock_results = Vec::new();
        let base_path = std::path::Path::new("/example/source");
        
        // Create results for different rule types and actions
        for i in 0..15 {
            mock_results.push(MatchResult {
                file_name: format!("document_{}.txt", i),
                current_path: base_path.join(format!("documents/document_{}.txt", i)),
                new_path: base_path.join("organized/documents").join(format!("document_{}.txt", i)),
                matched_rule_id: "document_organization_rule".to_string(),
                action: "move".to_string(),
            });
        }

        for i in 0..12 {
            mock_results.push(MatchResult {
                file_name: format!("backup_{}.log", i),
                current_path: base_path.join(format!("logs/backup_{}.log", i)),
                new_path: base_path.join("archive/logs").join(format!("backup_{}.log", i)),
                matched_rule_id: "log_backup_rule".to_string(),
                action: "copy".to_string(),
            });
        }

        for i in 0..8 {
            mock_results.push(MatchResult {
                file_name: format!("temp_{}.tmp", i),
                current_path: base_path.join(format!("temp/temp_{}.tmp", i)),
                new_path: base_path.join("temp").join(format!("temp_{}.tmp", i)), // Same path for delete
                matched_rule_id: "cleanup_rule".to_string(),
                action: "delete".to_string(),
            });
        }

        for i in 0..6 {
            mock_results.push(MatchResult {
                file_name: format!("old_file_{}.dat", i),
                current_path: base_path.join(format!("data/old_file_{}.dat", i)),
                new_path: base_path.join("data").join(format!("new_file_{}.dat", i)),
                matched_rule_id: "rename_rule".to_string(),
                action: "rename".to_string(),
            });
        }

        for i in 0..5 {
            mock_results.push(MatchResult {
                file_name: format!("script_{}.sh", i),
                current_path: base_path.join(format!("scripts/script_{}.sh", i)),
                new_path: base_path.join("executed").join(format!("executed_script_{}.result", i)),
                matched_rule_id: "script_execution_rule".to_string(),
                action: "execute".to_string(),
            });
        }

        for i in 0..10 {
            mock_results.push(MatchResult {
                file_name: format!("unknown_{}.xyz", i),
                current_path: base_path.join(format!("misc/unknown_{}.xyz", i)),
                new_path: base_path.join("misc").join(format!("unknown_{}.xyz", i)), // Same path for skip
                matched_rule_id: "none".to_string(),
                action: "skip".to_string(),
            });
        }

        // Generate PDF
        generate_pdf(&pdf_path, &mock_results).expect("PDF generation should succeed");

        // Verify PDF file was created
        assert!(pdf_path.exists(), "PDF file should be created");

        // Check file size
        let metadata = std::fs::metadata(&pdf_path).expect("Should be able to read PDF metadata");
        
        println!("Generated PDF for inspection at: {}", pdf_path.display());
        println!("PDF size: {} bytes", metadata.len());
        println!("Total match results: {}", mock_results.len());
        
        // Show action breakdown
        let move_count = mock_results.iter().filter(|r| r.action == "move").count();
        let copy_count = mock_results.iter().filter(|r| r.action == "copy").count();
        let delete_count = mock_results.iter().filter(|r| r.action == "delete").count();
        let rename_count = mock_results.iter().filter(|r| r.action == "rename").count();
        let execute_count = mock_results.iter().filter(|r| r.action == "execute").count();
        let _skip_count = mock_results.iter().filter(|r| r.action == "skip").count();
        
        println!("Action breakdown:");
        println!("  Move: {}", move_count);
        println!("  Copy: {}", copy_count);
        println!("  Delete: {}", delete_count);
        println!("  Rename: {}", rename_count);
        println!("  Execute: {}", execute_count);
        println!("  Skip: {}", _skip_count);

        assert!(metadata.len() > 2000, "PDF should be substantial for inspection");
    }

    #[test]
    fn test_pdf_generation_with_long_paths() {
        // Create PDF with extremely long paths to test wrapping
        let pdf_path = std::path::Path::new("test_report_long_paths.pdf");
        
        let mut mock_results = Vec::new();
        
        // Create results with very long paths
        mock_results.push(MatchResult {
            file_name: "document_with_very_very_very_long_filename_that_should_be_handled_properly.txt".to_string(),
            current_path: std::path::PathBuf::from("/home/user/documents/projects/rust/my_awesome_project/src/very/deeply/nested/directories/with/extremely/long/path/names/that/go/on/and/on/document_with_very_very_very_long_filename_that_should_be_handled_properly.txt"),
            new_path: std::path::PathBuf::from("/home/user/organized_files/documents/text_files/2024/august/important_documents/document_with_very_very_very_long_filename_that_should_be_handled_properly.txt"),
            matched_rule_id: "document_organization_with_very_long_rule_name".to_string(),
            action: "move".to_string(),
        });

        mock_results.push(MatchResult {
            file_name: "short.log".to_string(),
            current_path: std::path::PathBuf::from("/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z/very/long/path/structure/short.log"),
            new_path: std::path::PathBuf::from("/backup/logs/short.log"),
            matched_rule_id: "log_backup".to_string(),
            action: "copy".to_string(),
        });

        mock_results.push(MatchResult {
            file_name: "file_in_normal_path.dat".to_string(),
            current_path: std::path::PathBuf::from("/home/user/data/file_in_normal_path.dat"),
            new_path: std::path::PathBuf::from("/home/user/archived/file_in_normal_path.dat"),
            matched_rule_id: "normal_rule".to_string(),
            action: "move".to_string(),
        });

        // Generate PDF
        generate_pdf(&pdf_path, &mock_results).expect("PDF generation should succeed");

        // Verify PDF file was created
        assert!(pdf_path.exists(), "PDF file should be created");

        // Check file size
        let metadata = std::fs::metadata(&pdf_path).expect("Should be able to read PDF metadata");
        
        println!("Generated PDF with long paths at: {}", pdf_path.display());
        println!("PDF size: {} bytes", metadata.len());
        println!("Total match results: {}", mock_results.len());

        assert!(metadata.len() > 1000, "PDF should be substantial for inspection");
    }
}
