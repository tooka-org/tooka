mod commands;
mod core;
mod globals;

use core::logger::init_main_logger;

use clap::Parser;

#[derive(clap::Parser)]
#[clap(
    name = "tooka", 
    version, 
    about = "Tooka CLI", 
    long_about = "tooka is a command-line tool for managing and organizing files based on user-defined rules. 
    It allows you to add, remove, list, and sort files according to various criteria such as file extensions, 
    MIME types, patterns, and metadata.",
)]
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
    Completions(commands::completions::CompletionsArgs)
}

fn main() {
    let cli = Cli::parse();
    let config = core::config::Config::load();
    if let Err(e) = config {
        eprintln!("Error loading configuration: {}", e);
        std::process::exit(1);
    }
    init_main_logger()
        .expect("Failed to initialize main logger");

    log::info!("Tooka CLI started");

    match cli.command {
        Commands::Config(args) => commands::config::run(args),
        Commands::Add(args) => commands::add::run(args),
        Commands::Export(args) => commands::export::run(args),
        Commands::List(args) => commands::list::run(args),
        Commands::Remove(args) => commands::remove::run(args),
        Commands::Sort(args) => commands::sort::run(args),
        Commands::Completions(args) => commands::completions::run(args),
    }
}
