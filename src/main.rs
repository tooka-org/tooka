mod commands;
mod context;
mod core;

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
    Template(commands::template::TemplateArgs),
}

fn main() {
    let cli = Cli::parse();
    // Init everything
    if let Err(e) = context::init_config() {
        eprintln!("Failed to initialize config: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = context::init_rules_file() {
        eprintln!("Failed to initialize rules file: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }

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
        Commands::Template(args) => commands::template::run(args),
    }
}
