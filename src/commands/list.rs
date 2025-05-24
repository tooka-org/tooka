use clap::Args;
use crate::core::rules;

#[derive(Args)]
#[command(about = "Lists all current rules with their metadata")]
pub struct ListArgs;

pub fn run(_args: ListArgs) {
    println!("Listing all rules...");
    match rules::list_rules() {
        Ok(rules) => {
            if rules.is_empty() {
                println!("No rules found.");
            } else {
                for rule in rules {
                    println!("ID: {}, Name: {}, Enabled: {}", rule.id, rule.name, rule.enabled);
                }
            }
        },
        Err(e) => eprintln!("❌ Error listing rules: {}", e),
    }
}
