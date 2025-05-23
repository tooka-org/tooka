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

    /// Simulate the sorting without making changes
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

pub fn run(args: SortArgs) {
    println!("Running sort...");
    println!("Source Folder: {:?}", args.source.unwrap_or_else(|| "<default>".to_string()));
    println!("Rule IDs: {:?}", args.rules.unwrap_or_else(|| "<all>".to_string()));
    println!("Dry Run: {}", args.dry_run);

    // TODO: Implement sorting logic
}
