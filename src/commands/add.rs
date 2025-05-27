use crate::context;
use clap::Args;

#[derive(Args)]
#[command(about = "Adds a new rule by importing a YAML snippet file")]
pub struct AddArgs {
    /// Path to the rule YAML file
    #[arg(value_name = "FILE")]
    pub file: String,
}

pub fn run(args: &AddArgs) {
    println!("Adding rule from file: {}", args.file);
    log::info!("Adding rule from file: {}", args.file);
    let rf = context::get_rules_file();
    let mut rf = match rf.lock() {
        Ok(guard) => guard,
        Err(_) => {
            println!("Failed to lock rules file");
            log::error!("Failed to lock rules file");
            return;
        }
    };

    match rf.add_rule_from_file(&args.file) {
        Ok(()) => {
            println!("Rule added successfully!");
            log::info!("Rule added successfully from file: {}", args.file);
        }
        Err(e) => {
            eprintln!("Error adding rule: {e}");
            log::error!("Error adding rule from file {}: {}", args.file, e);
        }
    }
}
