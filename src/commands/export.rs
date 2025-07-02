use crate::core::context;
use anyhow::{Result, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about = "📤 Export a rule to a YAML file")]
pub struct ExportArgs {
    /// ID of the rule to export
    #[arg(value_name = "ID", help = "The unique identifier of the rule to export")]
    pub id: String,

    /// Output file path, optional; defaults to stdout
    #[arg(long, help = "Output file path (defaults to stdout if not specified)")]
    pub output: Option<String>,
}

pub fn run(args: ExportArgs) -> Result<()> {
    let output_path = args.output;

    log::info!("Exporting rule with ID: {}", args.id);

    let rf = context::get_locked_rules_file()?;

    rf.export_rule(&args.id, output_path.as_deref())
        .map_err(|e| anyhow!("Failed to export rule with ID {}: {}", args.id, e))?;

    if output_path.is_some() {
        println!("Rule exported successfully!");
        log::info!("Rule exported successfully to: {:?}", output_path);
    } else {
        log::info!("Rule exported successfully to stdout");
    }

    Ok(())
}
