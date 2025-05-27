use clap::Args;

use crate::context;

#[derive(Args)]
#[command(about = "Toggles the state of a rule by its ID")]
pub struct ToggleArgs {
    /// ID of the rule to toggle
    pub rule_id: String,
}

pub fn run(args: &ToggleArgs) {
    log::info!("Toggling rule with ID: {}", args.rule_id);
    let rf = context::get_rules_file();
    let mut rf = match rf.lock() {
        Ok(guard) => guard,
        Err(_) => {
            println!("Failed to lock rules file");
            log::error!("Failed to lock rules file");
            return;
        }
    };

    match rf.find_rule(&args.rule_id) {
        Some(rule) if rule.id == args.rule_id => {
            log::debug!(
                "Found rule: ID={}, Name={}, Enabled={}",
                rule.id,
                rule.name,
                rule.enabled
            );
            match rf.toggle_rule(&args.rule_id) {
                Ok(()) => {
                    println!("Rule toggled successfully!");
                    log::info!("Rule with ID '{}' toggled successfully.", args.rule_id);
                }
                Err(e) => {
                    println!("Error toggling rule: {e}");
                    log::error!("Error toggling rule with ID '{}': {}", args.rule_id, e);
                }
            }
        }
        Some(_) => {
            println!("Rule found, but ID does not match '{}'.", args.rule_id);
            log::warn!("Rule found, but ID does not match '{}'.", args.rule_id);
        }
        None => {
            println!("Rule with ID '{}' not found.", args.rule_id);
            log::warn!("Rule with ID '{}' not found.", args.rule_id);
        }
    }
}
