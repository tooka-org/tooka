use clap::Args;
use crate::core::rules;

#[derive(Args)]
#[command(about = "Adds a new rule by importing a YAML snippet file")]
pub struct AddArgs {
    /// Path to the rule YAML file
    #[arg(value_name = "FILE")]
    pub file: String,
}

pub fn run(args: AddArgs) {
    println!("📥 Adding rule from file: {}", args.file);

    match rules::add_rule_from_file(&args.file) {
        Ok(_) => println!("✅ Rule added successfully!"),
        Err(e) => eprintln!("❌ Error adding rule: {}", e),
    }
}
