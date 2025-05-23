use clap::Args;

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

    // TODO: Implement actual export logic here
}
