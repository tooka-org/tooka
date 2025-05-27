use crate::context;
use clap::Args;

#[derive(Args)]
#[command(about = "Lists all current rules with their metadata")]
pub struct ListArgs;

pub fn run(_args: ListArgs) {
    log::info!("Listing all rules...");
    let rf = context::get_rules_file();
    let rf = match rf.lock() {
        Ok(guard) => guard,
        Err(_) => {
            println!("Failed to lock rules file");
            log::error!("Failed to lock rules file");
            return;
        }
    };

    let rules_list = rf.list_rules();

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
