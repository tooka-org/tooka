use crate::core::rules;
use clap::Args;

#[derive(Args)]
#[command(about = "Lists all current rules with their metadata")]
pub struct ListArgs;

pub fn run(_args: ListArgs) {
    log::info!("Listing all rules...");

    let rules_list = rules::list_rules();

    if rules_list.is_empty() {
        log::info!("No rules found.");
        println!("No rules found.");
        return;
    }
    log::info!("Found {} rules.", rules_list.len());
    for rule in &rules_list {
        log::debug!(
            "Rule ID: {}, Name: {}, Enabled: {}",
            rule.id,
            rule.name,
            rule.enabled
        );
    }
}
