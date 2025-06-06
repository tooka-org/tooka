use anyhow::Result;
use clap::Args;
use tooka_core::context;

#[derive(Args)]
#[command(about = "Adds a new rule by importing a YAML snippet file")]
pub struct AddArgs {
    /// Path to the rule YAML file
    #[arg(value_name = "FILE")]
    pub file: String,

    /// Optional flag to overwrite existing rules
    #[arg(long, default_value_t = false)]
    pub overwrite: bool,
}

pub fn run(args: &AddArgs) -> Result<()> {
    log::info!("Adding rule from file: {}", args.file);

    let mut rf = context::get_locked_rules_file()?;

    rf.add_rule_from_file(&args.file, args.overwrite)
        .map_err(|e| anyhow::anyhow!("Failed to add rule from file: {}: {}", args.file, e))?;

    println!("Rule added successfully!");
    log::info!("Rule added successfully from file: {}", args.file);

    Ok(())
}
