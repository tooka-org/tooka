mod cli;
mod commands;
mod common;
mod completions;
mod core;
mod file;
mod rules;
mod utils;

use crate::cli::display;
use crate::common::logger::init_logger;
use crate::core::context::{init_config, init_rules_file};
use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[clap(
    name = "tooka",
    version,
    about = "🚀 A fast, rule-based CLI tool for organizing your files",
    long_about = "tooka is a powerful command-line tool for automatically managing and organizing files based on user-defined rules."
)]
#[command(disable_version_flag = true)]
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
    // Check if no arguments are provided
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        display::show_banner();
        return Ok(());
    }
    if args.len() == 2 && args[1] == "--version" {
        display::show_version();
        return Ok(());
    }

    // Top-level error handling
    if let Err(e) = run() {
        display::error(&format!("Error: {e}"));
        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    init_config()?;
    init_logger()?;
    init_rules_file()?;

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
