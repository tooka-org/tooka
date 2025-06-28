use crate::core::context;
use anyhow::{Result, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about = "Toggles the state of a rule by its ID")]
pub struct ToggleArgs {
    /// ID of the rule to toggle
    #[arg(value_name = "ID")]
    pub rule_id: String,
}

pub fn run(args: &ToggleArgs) -> Result<()> {
    log::info!("Toggling rule with ID: {}", args.rule_id);

    let mut rf = context::get_locked_rules_file()?;

    let rule = rf.find_rule(&args.rule_id);

    if rule.is_none() {
        log::warn!("Rule with ID '{}' not found.", args.rule_id);
        return Err(anyhow!("Rule with ID '{}' not found.", args.rule_id));
    }

    rf.toggle_rule(&args.rule_id)
        .map_err(|e| anyhow!("Failed to toggle rule with ID '{}': {}", args.rule_id, e))?;

    println!("Rule with ID '{}' toggled successfully.", args.rule_id);

    Ok(())
}
