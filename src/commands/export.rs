use crate::globals;
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

pub fn run(args: ExportArgs) {
    println!("Exporting rule with ID: {}", args.id);
    let output_path = args
        .output
        .unwrap_or_else(|| format!("rule-{}.yaml", args.id));
    log::info!("Exporting rule with ID: {} to {}", args.id, output_path);
    let rf = globals::get_rules_file();
    let rf = match rf.lock() {
        Ok(guard) => guard,
        Err(_) => {
            println!("Failed to lock rules file");
            log::error!("Failed to lock rules file");
            return;
        }
    };

    match rf.export_rule(&args.id, Some(&output_path)) {
        Ok(()) => {
            println!("Rule exported successfully!");
            log::info!("Rule exported successfully to: {output_path}");
        }
        Err(e) => {
            println!("Error exporting rule: {e}");
            log::error!("Error exporting rule with ID {}: {}", args.id, e);
        }
    }
}
