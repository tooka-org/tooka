mod commands;
mod core;
mod globals;

use core::logger::init_logger;

use clap::Parser;

#[derive(Parser)]
#[clap(
    name = "tooka",
    version,
    about = "Tooka CLI",
    long_about = "tooka is a command-line tool for managing and organizing files based on user-defined rules. 
    It allows you to add, remove, list, and sort files according to various criteria such as file extensions, 
    MIME types, patterns, and metadata."
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    Add(commands::add::AddArgs),
    Completions(commands::completions::CompletionsArgs),
    Config(commands::config::ConfigArgs),
    Export(commands::export::ExportArgs),
    List(commands::list::ListArgs),
    Remove(commands::remove::RemoveArgs),
    Sort(commands::sort::SortArgs),
    Toggle(commands::toggle::ToggleArgs),
}

fn main() {
    let cli = Cli::parse();
    core::config::Config::load();

    init_logger().expect("Failed to initialize logger");

    log::info!("Tooka CLI started");

    match cli.command {
        Commands::Config(args) => commands::config::run(&args),
        Commands::Add(args) => commands::add::run(&args),
        Commands::Export(args) => commands::export::run(args),
        Commands::List(args) => commands::list::run(args),
        Commands::Remove(args) => commands::remove::run(&args),
        Commands::Sort(args) => commands::sort::run(args),
        Commands::Toggle(args) => commands::toggle::run(&args),
        Commands::Completions(args) => commands::completions::run(&args),
    }
}
