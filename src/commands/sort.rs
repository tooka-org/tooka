use clap::Args;
use crate::core::{logger::init_ops_logger, sorter};

#[derive(Args)]
#[command(about = "Manually runs the sorter on the source folder")]
pub struct SortArgs {
    /// Override default source folder
    #[arg(long)]
    pub source: Option<String>,

    /// Comma-separated rule IDs to run
    #[arg(long)]
    pub rules: Option<String>,

    /// Simulate the sorting without making changes
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

pub fn run(args: SortArgs) {
    println!("Running sort...");

    init_ops_logger().expect("Failed to initialize operations logger");

    let source = args.source.unwrap_or_else(|| "<default>".to_string());
    let rules = args.rules.unwrap_or_else(|| "<all>".to_string());
    let dry_run = args.dry_run;

    let results = sorter::sort_files(source, rules, dry_run);
    match results {
        Ok(matches) => {
            for match_result in matches {
                println!("File: {}, Matched: {}, Current Path: {}, New Path: {}", 
                         match_result.file_name, 
                         match_result.matched_rule_id, 
                        match_result.current_path.display(),
                         match_result.new_path.display());
            }
        },
        Err(e) => {
            eprintln!("Error during sorting: {}", e);
        }
    }
}
