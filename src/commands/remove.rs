use crate::core::rules;
use clap::Args;

#[derive(Args)]
#[command(about = "Removes a single rule by ID")]
pub struct RemoveArgs {
    /// ID of the rule to remove
    pub rule_id: String,
}

pub fn run(args: RemoveArgs) {
    println!("Removing rule ID: {}", args.rule_id);
    log::info!("Removing rule with ID: {}", args.rule_id);

    match rules::find_rule(&args.rule_id) {
        Ok(Some(rule)) => {
            log::debug!(
                "Found rule: ID={}, Name={}, Enabled={}",
                rule.id,
                rule.name,
                rule.enabled
            );
            match rules::remove_rule(&args.rule_id) {
                Ok(_) => {
                    println!("✅ Rule removed successfully!");
                    log::info!("Rule with ID '{}' removed successfully.", args.rule_id);
                }
                Err(e) => {
                    println!("❌ Error removing rule: {}", e);
                    log::error!("Error removing rule with ID '{}': {}", args.rule_id, e);
                }
            }
        }
        Ok(None) => {
            println!("❌ Rule with ID '{}' not found.", args.rule_id);
            log::warn!("Rule with ID '{}' not found.", args.rule_id);
        }
        Err(e) => {
            println!("❌ Error finding rule: {}", e);
            log::error!("Error finding rule with ID '{}': {}", args.rule_id, e);
        }
    }
}
