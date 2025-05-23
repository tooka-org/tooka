use clap::Args;

#[derive(Args)]
#[command(about = "Removes a single rule by ID")]
pub struct RemoveArgs {
    /// ID of the rule to remove
    pub rule_id: String,
}

pub fn run(args: RemoveArgs) {
    println!("Removing rule ID: {}", args.rule_id);
    // TODO: Implement removal logic
}
