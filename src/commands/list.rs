use clap::Args;
use crate::core::rules;

#[derive(Args)]
#[command(about = "Lists all current rules with their metadata")]
pub struct ListArgs;

pub fn run(_args: ListArgs) {
    log::info!("Listing all rules...");

    match rules::list_rules() {
        Ok(rules) => {
            if rules.is_empty() {
                log::info!("No rules found.");
                println!("No rules found.");
            } else {
                for rule in rules {
                    println!("ID: {}, Name: {}, Enabled: {}", rule.id, rule.name, rule.enabled);
                }
            }
        },
        Err(e) => {
            println!("❌ Error listing rules: {}", e);
            log::error!("Error listing rules: {}", e);
        }
    }
}
