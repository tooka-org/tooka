mod commands;
mod core;
mod globals;

use clap::Parser;

#[derive(clap::Parser)]
#[clap(name = "tooka", version, about = "Tooka CLI", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Config(commands::config::ConfigArgs),
    Add(commands::add::AddArgs),
    Export(commands::export::ExportArgs),
    List(commands::list::ListArgs),
    Remove(commands::remove::RemoveArgs),
    Sort(commands::sort::SortArgs),
}

fn main() {
    let cli = Cli::parse();
    let config = core::config::Config::load();
    if let Err(e) = config {
        eprintln!("Error loading configuration: {}", e);
        std::process::exit(1);
    }

    match cli.command {
        Commands::Config(args) => commands::config::run(args),
        Commands::Add(args) => commands::add::run(args),
        Commands::Export(args) => commands::export::run(args),
        Commands::List(args) => commands::list::run(args),
        Commands::Remove(args) => commands::remove::run(args),
        Commands::Sort(args) => commands::sort::run(args),
    }
}

