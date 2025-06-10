#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
        path::{Path, PathBuf},
    };
    use tempfile::tempdir;
    use tooka_core::{
        error::TookaError,
        rule::{Action, Conditions, MoveAction, Range, Rule},
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

        // We expect 4 out of 5 files to match (unknown extension should not match any rule)
        assert_eq!(results.len(), 4);

        for result in results {
            let expected_ext = result
                .file_name
                .split('.')
                .last()
                .expect("File should have extension");

            assert_eq!(result.action, "move");
            assert!(result.matched_rule_id.contains(expected_ext));
            assert!(
                result
                    .new_path
                    .starts_with(base_path.join(format!("out_{}", expected_ext)))
            );
        }

        Ok(())
    }
}
