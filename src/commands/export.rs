use clap::Args;
use crate::core::rules;

#[derive(Args)]
#[command(about = "Exports a single rule by ID to a YAML file")]
pub struct ExportArgs {
    /// ID of the rule to export
    #[arg(long, required = true)]
    pub id: String,

    /// Output file path
    #[arg(long, required = true)]
    pub output: String,
}

pub fn run(args: ExportArgs) {
    println!("📤 Exporting rule with ID: {}", args.id);
    log::info!("Exporting rule with ID: {} to {}", args.id, args.output);

    match rules::export_rule(&args.id, &args.output) {
        Ok(_) => {
            println!("✅ Rule exported successfully!"); 
            log::info!("Rule exported successfully to: {}", args.output);
        },
        Err(e) => {
            println!("❌ Error exporting rule: {}", e);
            log::error!("Error exporting rule with ID {}: {}", args.id, e);
        }
    }
}
