use crate::core::rules;
use clap::Args;

#[derive(Args)]
#[command(about = "Toggles the state of a rule by its ID")]
pub struct ToggleArgs {
    /// ID of the rule to toggle
    pub rule_id: String,
}

pub fn run(args: &ToggleArgs) {
    log::info!("Toggling rule with ID: {}", args.rule_id);

    match rules::find_rule(&args.rule_id) {
        Some(rule) if rule.id == args.rule_id => {
            log::debug!(
                "Found rule: ID={}, Name={}, Enabled={}",
                rule.id,
                rule.name,
                rule.enabled
            );
            match rules::toggle_rule(&args.rule_id) {
                Ok(()) => {
                    println!("✅ Rule toggled successfully!");
                    log::info!("Rule with ID '{}' toggled successfully.", args.rule_id);
                }
                Err(e) => {
                    println!("❌ Error toggling rule: {e}");
                    log::error!("Error toggling rule with ID '{}': {}", args.rule_id, e);
                }
            }
        }
        Some(_) => {
            println!("❌ Rule found, but ID does not match '{}'.", args.rule_id);
            log::warn!("Rule found, but ID does not match '{}'.", args.rule_id);
        }
        None => {
            println!("❌ Rule with ID '{}' not found.", args.rule_id);
            log::warn!("Rule with ID '{}' not found.", args.rule_id);
        }
    }
}
