use clap::Args;
use crate::core::rules;

#[derive(Args)]
#[command(about = "Removes a single rule by ID")]
pub struct RemoveArgs {
    /// ID of the rule to remove
    pub rule_id: String,
}

pub fn run(args: RemoveArgs) {
    println!("Removing rule ID: {}", args.rule_id);
    
    match rules::find_rule(&args.rule_id) {
        Ok(Some(rule)) => {
            println!("Found rule: ID={}, Name={}, Enabled={}", rule.id, rule.name, rule.enabled);
            match rules::remove_rule(&args.rule_id) {
                Ok(_) => println!("✅ Rule removed successfully!"),
                Err(e) => eprintln!("❌ Error removing rule: {}", e),
            }
        },
        Ok(None) => {
            eprintln!("❌ Rule with ID '{}' not found.", args.rule_id);
        },
        Err(e) => eprintln!("❌ Error finding rule: {}", e),
    }
}
