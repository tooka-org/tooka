use crate::cli::display;
use crate::core::context;
use anyhow::Result;
use clap::Args;

#[derive(Args)]
#[command(about = "📋 List all current rules with their metadata")]
pub struct ListArgs;

pub fn run(_args: ListArgs) -> Result<()> {
    log::info!("Listing all rules...");

    let rf = context::get_locked_rules_file()?;
    let rules_list = rf.list_rules();

    if rules_list.is_empty() {
        display::warning("No rules found.");
        display::info("Use `tooka add` to create your first rule.");
        return Ok(());
    }

    display::header(&format!("📋 Found {} rules", rules_list.len()));
    display::rule_table_header();

    for rule in &rules_list {
        log::debug!(
            "Rule ID: {}, Name: {}, Enabled: {}",
            rule.id,
            rule.name,
            rule.enabled
        );
        display::rule_table_row(&rule.id, &rule.name, rule.enabled);
    }

    println!();
    display::success("Rules listed successfully!");

    Ok(())
}
