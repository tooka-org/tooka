use crate::cli::display;
use crate::core::context;
use anyhow::{Result, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about = "🗑️  Remove a rule by its ID")]
pub struct RemoveArgs {
    /// ID of the rule to remove
    #[arg(
        value_name = "ID",
        help = "The unique identifier of the rule to remove"
    )]
    pub rule_id: String,
}

pub fn run(args: &RemoveArgs) -> Result<()> {
    display::info(&format!("🗑️ Removing rule with ID: {}", args.rule_id));
    log::info!("Removing rule with ID: {}", args.rule_id);

    let mut rf = context::get_locked_rules_file()?;

    if let Some(rule) = rf.find_rule(&args.rule_id) {
        log::debug!(
            "Found rule: ID={}, Name={}, Enabled={}",
            rule.id,
            rule.name,
            rule.enabled
        );

        rf.remove_rule(&args.rule_id)
            .map_err(|e| anyhow!("Failed to remove rule with ID '{}': {}", args.rule_id, e))?;

        display::success(&format!(
            "Rule with ID '{}' removed successfully.",
            args.rule_id
        ));
        Ok(())
    } else {
        let error_msg = format!("Rule with ID '{}' not found.", args.rule_id);
        display::error(&error_msg);
        log::warn!("{error_msg}");
        Err(anyhow!(error_msg))
    }
}
