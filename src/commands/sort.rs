use std::path::PathBuf;

use crate::core::{report, sorter};
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "Manually runs the sorter on the source folder")]
pub struct SortArgs {
    /// Override default source folder
    #[arg(long)]
    pub source: Option<String>,
    /// Comma-separated rule IDs to run
    #[arg(long)]
    pub rules: Option<String>,
    /// Output report format: pdf, csv, json
    #[arg(long)]
    pub report: Option<String>,
    /// Output directory for the report
    #[arg(long)]
    pub output: Option<String>,
    /// Simulate the sorting without making changes
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

pub fn run(args: SortArgs) -> Result<()> {
    println!("Running sort...");
    log::info!(
        "Running sort with source: {:?}, rules: {:?}, dry_run: {}",
        args.source,
        args.rules,
        args.dry_run
    );

    let source = args.source.unwrap_or_else(|| "<default>".to_string());
    let rules = args.rules.unwrap_or_else(|| "<all>".to_string());
    let dry_run = args.dry_run;

    let results = sorter::sort_files(source.clone(), &rules, dry_run).map_err(|e| {
        anyhow::anyhow!(
            "{}: {}",
            format!("Failed to sort files from source: {source}"),
            e
        )
    })?;

    println!("Sorting completed. Results:");
    log::info!("Sorting completed, found {} matches", results.len());

    for match_result in &results {
        println!(
            "File: {}, Matched: {}, Current Path: {}, New Path: {}",
            match_result.file_name,
            match_result.matched_rule_id,
            match_result.current_path.display(),
            match_result.new_path.display()
        );
    }

    // Handle report generation
    if let Some(report_type) = &args.report {
        let output_dir = args.output.as_ref().map(PathBuf::from).unwrap_or_else(|| {
            std::env::current_dir().expect("Cannot get current working directory")
        });

        report::generate_report(report_type, &output_dir, &results)?;
    }

    Ok(())
}
