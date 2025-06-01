use crate::context;
use anyhow::{Result, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about = "Lists all current rules with their metadata")]
pub struct ListArgs;

pub fn run(_args: ListArgs) -> Result<()> {
    log::info!("Listing all rules...");

    let rf = context::get_locked_rules_file()?;

    let rules_list = rf.list_rules();

    if rules_list.is_empty() {
        log::warn!("No rules found.");
        return Err(anyhow!("No rules found."));
    }

    log::info!("Found {} rules.", rules_list.len());

    for rule in &rules_list {
        log::debug!(
            "Rule ID: {}, Name: {}, Enabled: {}",
            rule.id,
            rule.name,
            rule.enabled
        );
        println!(
            "Rule ID: {}, Name: {}, Enabled: {}",
            rule.id, rule.name, rule.enabled
        );
    }

    Ok(())
}
