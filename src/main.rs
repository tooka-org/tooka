mod commands;
mod common;
mod completions;
mod core;
mod file;
mod rules;
mod utils;

use crate::common::logger::init_logger;
use crate::core::context::{init_config, init_rules_file};
use anyhow::Result;
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
    Completions(completions::CompletionsArgs),
    Config(commands::config::ConfigArgs),
    Export(commands::export::ExportArgs),
    List(commands::list::ListArgs),
    Remove(commands::remove::RemoveArgs),
    Sort(commands::sort::SortArgs),
    Toggle(commands::toggle::ToggleArgs),
    Template(commands::template::TemplateArgs),
    Validate(commands::validate::ValidateArgs),
}

fn main() -> Result<()> {
    // Top-level error handling
    if let Err(e) = run() {
        eprintln!("❌ Error: {e}");
        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Only initialize what's needed based on the command
    match &cli.command {
        Commands::Sort(_) => {
            // For sort command, we'll handle config/rules loading directly
            // for better performance
            init_logger()?;
        }
        _ => {
            // For other commands, use the traditional global context approach
            init_config()?;
            init_logger()?;
            init_rules_file()?;
        }
    }

    log::info!("Tooka CLI started");

    match cli.command {
        Commands::Config(args) => commands::config::run(&args)?,
        Commands::Add(args) => commands::add::run(&args)?,
        Commands::Export(args) => commands::export::run(args)?,
        Commands::List(args) => commands::list::run(args)?,
        Commands::Remove(args) => commands::remove::run(&args)?,
        Commands::Sort(args) => commands::sort::run(args)?,
        Commands::Toggle(args) => commands::toggle::run(&args)?,
        Commands::Completions(args) => completions::run(&args)?,
        Commands::Template(args) => commands::template::run(args)?,
        Commands::Validate(args) => commands::validate::run(&args)?,
    }

    Ok(())
}
