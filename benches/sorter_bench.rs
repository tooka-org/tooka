use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;

use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};

// Include the necessary modules directly from the source
#[path = "../src/core/mod.rs"]
mod core;

#[path = "../src/rules/mod.rs"]
mod rules;

#[path = "../src/file/mod.rs"]
mod file;

#[path = "../src/common/mod.rs"]
mod common;

#[path = "../src/utils/mod.rs"]
mod utils;

// Re-export the specific items we need for the benchmark
use crate::core::error::TookaError;
use crate::core::sorter::sort_files;
use crate::rules::rule::{Action, Conditions, MoveAction, Range, Rule};
use crate::rules::rules_file::RulesFile;

/// Generates temporary files of various types (some matching, some not)
fn generate_mixed_temp_files(base_dir: &Path, count: usize, avg_kb: usize) -> Vec<PathBuf> {
    let mut paths = vec![];
    let extensions = ["txt", "log", "data", "bin", "unknown"];

    for i in 0..count {
        let ext = extensions[i % extensions.len()];
        let subdir = base_dir.join(format!("subdir_{}", i % 5));
        create_dir_all(&subdir).unwrap();

        let file_path = subdir.join(format!("file_{i}.{ext}"));
        let mut file = File::create(&file_path).unwrap();

        let content = vec![b'x'; avg_kb * 1024];
        file.write_all(&content).unwrap();

        paths.push(file_path);
    }

    paths
}

/// Creates multiple dummy rules for known extensions
fn create_complex_dummy_rules(dest: &Path) -> RulesFile {
    let mut rules = vec![];
    let extensions = ["txt", "log", "data", "bin"];
    for ext in extensions {
        let target_dir = dest.join(format!("matched_{ext}"));
        create_dir_all(&target_dir).unwrap();

        rules.push(Rule {
            id: format!("{ext}_move"),
            name: format!("Move .{ext} files"),
            enabled: true,
            description: Some(format!("Moves all .{ext} files")),
            priority: 10,
            when: Conditions {
                any: Some(false),
                filename: Some(format!(r".*\.{ext}$")),
                extensions: Some(vec![ext.into()]),
                path: None,
                size_kb: Some(Range {
                    min: Some(1),
                    max: Some(10_000),
                }),
                mime_type: None,
                created_date: None,
                modified_date: None,
                is_symlink: None,
                metadata: None,
            },
            then: vec![Action::Move(MoveAction {
                to: target_dir.to_string_lossy().into(),
                preserve_structure: true,
            })],
        });
    }

    RulesFile { rules }
}

/// Runs the full sort operation with dynamic mixed data
fn benchmark_with_temp_data(file_count: usize, avg_kb: usize) -> Result<(), TookaError> {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().to_path_buf();
    let files = generate_mixed_temp_files(&source_path, file_count, avg_kb);
    let rules_file = create_complex_dummy_rules(&source_path);

    // Use the optimized version
    let optimized_rules = rules_file.optimized_with_filter(None)?;
    let _results = sort_files(&files, &source_path, &optimized_rules, true, None::<fn()>)
        .expect("sort_files failed");

    Ok(())
}

/// Criterion benchmark entrypoint
fn sorter_benchmarks(c: &mut Criterion) {
    let test_cases = vec![
        (10, 1),     // 10 small files
        (100, 5),    // 100 mid-sized files
        (500, 10),   // 500 larger files
        (1000, 1),   // 1000 tiny files
        (200, 50),   // 200 big files
        (100, 1024), // 100 x 1MB files
        (5000, 0),   // 5000 empty files
    ];

    for (count, size_kb) in test_cases {
        let label = format!("sort_files_{count}files_{size_kb}KB");
        c.bench_function(&label, |b| {
            b.iter(|| {
                benchmark_with_temp_data(count, size_kb).unwrap();
            });
        });
    }
}

criterion_group!(benches, sorter_benchmarks);
criterion_main!(benches);
