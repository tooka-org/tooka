use crate::cli::display;
use crate::core::context;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "📝 Add a new rule by importing a YAML file")]
pub struct AddArgs {
    /// Path to the rule YAML file
    #[arg(
        value_name = "FILE",
        help = "Path to the YAML file containing the rule definition"
    )]
    pub file: String,

    /// Optional flag to overwrite existing rules
    #[arg(
        long,
        default_value_t = false,
        help = "Overwrite existing rule if it already exists"
    )]
    pub overwrite: bool,
}

pub fn run(args: &AddArgs) -> Result<()> {
    display::info(&format!("📝 Adding rule from file: {}", args.file));
    log::info!("Adding rule from file: {}", args.file);

    let mut rf = context::get_locked_rules_file()?;

    rf.add_rule_from_file(&args.file, args.overwrite)
        .map_err(|e| anyhow::anyhow!("Failed to add rule from file: {}: {}", args.file, e))?;

    display::success("Rule added successfully!");
    log::info!("Rule added successfully from file: {}", args.file);

    Ok(())
}
