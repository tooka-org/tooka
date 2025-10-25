use crate::cli;
use crate::core::context;
use anyhow::{Result, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about = "🔄 Toggle the enabled/disabled state of a rule")]
pub struct ToggleArgs {
    /// ID of the rule to toggle
    #[arg(
        value_name = "ID",
        help = "The unique identifier of the rule to toggle"
    )]
    pub rule_id: String,
}

pub fn run(args: &ToggleArgs) -> Result<()> {
    cli::info(&format!("🔄 Toggling rule with ID: {}", args.rule_id));
    log::info!("Toggling rule with ID: {}", args.rule_id);

    let mut rf = context::get_locked_rules_file()?;
    let Some(rule) = rf.find_rule(&args.rule_id) else {
        cli::error(&format!("Rule with ID '{}' not found.", args.rule_id));
        log::warn!("Rule with ID '{}' not found.", args.rule_id);
        return Err(anyhow!("Rule with ID '{}' not found.", args.rule_id));
    };

    let was_enabled = rule.enabled;

    rf.toggle_rule(&args.rule_id)
        .map_err(|e| anyhow!("Failed to toggle rule with ID '{}': {}", args.rule_id, e))?;

    let status = if was_enabled { "disabled" } else { "enabled" };
    cli::success(&format!(
        "Rule with ID '{}' is now {}.",
        args.rule_id, status
    ));

    Ok(())
}
