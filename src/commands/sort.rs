use std::path::PathBuf;

use crate::cli::display;
use crate::common::config::Config;
use crate::core::{report, sorter};
use crate::rules::rules_file::RulesFile;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use indicatif::ProgressBar;

#[derive(Args)]
#[command(about = "🚀 Sort files in the source folder using defined rules")]
pub struct SortArgs {
    /// Override default source folder
    #[arg(long, help = "Override the default source folder path")]
    pub source: Option<String>,
    /// Comma-separated rule IDs to run
    #[arg(
        long,
        help = "Comma-separated list of rule IDs to execute (use '<all>' for all rules)"
    )]
    pub rules: Option<String>,
    /// Output report format: pdf, csv, json
    #[arg(
        long,
        help = "Generate a report in the specified format (pdf, csv, json)"
    )]
    pub report: Option<String>,
    /// Output directory for the report
    #[arg(long, help = "Directory where the report will be saved")]
    pub output: Option<String>,
    /// Simulate the sorting without making changes
    #[arg(
        long,
        default_value_t = false,
        help = "Preview what would happen without actually moving files"
    )]
    pub dry_run: bool,
}

pub fn run(args: SortArgs) -> Result<()> {
    if args.dry_run {
        display::warning("🔍 Running in dry-run mode - no files will be moved");
    } else {
        display::info("🚀 Starting file sorting...");
    }

    log::info!(
        "Running sort with source: {:?}, rules: {:?}, dry_run: {}",
        args.source,
        args.rules,
        args.dry_run
    );

    // Load config and rules directly instead of using global context
    let config = Config::load()?;
    let source_path = if let Some(source) = args.source {
        if source == "<default>" {
            config.source_folder.clone()
        } else {
            PathBuf::from(source)
        }
    } else {
        config.source_folder.clone()
    };

    let rules_file = RulesFile::load()?;

    // Parse rule filter
    let rule_filter = args.rules.as_ref().and_then(|r| {
        if r == "<all>" {
            None
        } else {
            Some(
                r.split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>(),
            )
        }
    });

    let optimized_rules = rules_file.optimized_with_filter(rule_filter.as_deref())?;

    // Collect files first to show progress bar
    let files = sorter::collect_files(&source_path)?;

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(display::progress_style());

    // Use the main sort_files function with optimized rules
    let results = sorter::sort_files(
        &files,
        &source_path,
        &optimized_rules,
        args.dry_run,
        Some(|| {
            pb.inc(1);
        }),
    )?;

    pb.finish_with_message("✅ Sorting complete");

    display::success("Sorting completed successfully!");
    log::info!("Sorting completed, found {} matches", results.len());

    if args.report.is_none() && !results.is_empty() {
        display::header("📁 Sorted Files");

        println!(
            "{} | {} | {} | {}",
            "File".bright_cyan().bold(),
            "Matched Rule".bright_cyan().bold(),
            "Current Path".bright_cyan().bold(),
            "New Path".bright_cyan().bold()
        );
        println!("{}", "─".repeat(120).bright_black());

        for result in &results {
            println!(
                "{:<40} | {:<30} | {:<40} | {}",
                result.file_name.bright_white(),
                result.matched_rule_id.green(),
                result.current_path.display().to_string().yellow(),
                result.new_path.display().to_string().blue()
            );
        }
    } else if results.is_empty() {
        display::info("No files matched the sorting rules.");
    }

    // Handle report generation
    if let Some(report_type) = &args.report {
        log::info!("Generating report of type: {report_type}");
        let output_dir = args.output.as_ref().map_or_else(
            || std::env::current_dir().expect("Cannot get current working directory"),
            PathBuf::from,
        );

        report::generate_report(report_type, &output_dir, &results)?;
        display::success(&format!(
            "Report generated successfully in {}",
            output_dir.display()
        ));
    }

    Ok(())
}
