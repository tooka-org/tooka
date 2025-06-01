use crate::context;
use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Exports a single rule by ID to a YAML file")]
pub struct ExportArgs {
    /// ID of the rule to export
    #[arg(value_name = "ID")]
    pub id: String,

    /// Output file path
    #[arg(long)]
    pub output: Option<String>,
}

pub fn run(args: ExportArgs) -> Result<()> {
    let output_path = args
        .output
        .unwrap_or_else(|| format!("rule-{}.yaml", args.id));

    log::info!("Exporting rule with ID: {} to {}", args.id, output_path);
    
    let rf = context::get_locked_rules_file()?;

    rf.export_rule(&args.id, Some(&output_path))
        .map_err(|e| anyhow!("Failed to export rule with ID {}: {}", args.id, e))?;

    println!("Rule exported successfully!");
    log::info!("Rule exported successfully to: {output_path}");

    Ok(())
}
