use clap::Args;

#[derive(Args)]
#[command(about = "Lists all current rules with their metadata")]
pub struct ListArgs;

pub fn run(_args: ListArgs) {
    println!("Listing all rules...");
    // TODO: Implement actual listing logic
}
