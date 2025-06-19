use std::path::PathBuf;

use anyhow::Result;
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use tooka_core::{report, sorter};

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

    let (source_path, rules_file, files) = sorter::prepare_sort(&source, &rules)?;

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files sorted",
        )
        .unwrap(),
    );

    let results = sorter::sort_files(
        files,
        source_path,
        &rules_file,
        dry_run,
        Some(|| {
            pb.inc(1);
        }),
    )?;

    pb.finish_with_message("Sorting complete");

    println!("Sorting completed.");
    log::info!("Sorting completed, found {} matches", results.len());

    if args.report.is_none() {
        println!(
            "{:<40} | {:<30} | {:<40} | New Path",
            "File", "Matched Rule", "Current Path"
        );

        for result in &results {
            println!(
                "{:<40} | {:<30} | {:<40} | {}",
                result.file_name,
                result.matched_rule_id,
                result.current_path.display(),
                result.new_path.display()
            );
        }
    }

    // Handle report generation
    if let Some(report_type) = &args.report {
        log::info!("Generating report of type: {}", report_type);
        let output_dir = args.output.as_ref().map(PathBuf::from).unwrap_or_else(|| {
            std::env::current_dir().expect("Cannot get current working directory")
        });

        report::generate_report(report_type, &output_dir, &results)?;
        println!(
            "Report generated successfully in {:?}",
            output_dir.display()
        );
    }

    Ok(())
}
