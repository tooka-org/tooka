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
    println!("Exporting rule ID: {}", args.id);
    println!("Output path: {}", args.output);

    match rules::export_rule(&args.id, &args.output) {
        Ok(_) => println!("✅ Rule exported successfully!"),
        Err(e) => eprintln!("❌ Error exporting rule: {}", e),
    }
}
