use crate::core::context;
use crate::{cli::display, common::config::Config};
use anyhow::{Context, Result, anyhow};
use clap::Args;

#[derive(Args)]
#[command(about = "⚙️ Manage the Tooka configuration file")]
pub struct ConfigArgs {
    /// Flag to locate the configuration file
    #[arg(long, help = "Show the path to the configuration file")]
    pub locate: bool,

    /// Flag to reset the configuration file to default values
    #[arg(long, help = "Reset configuration to default values")]
    pub reset: bool,

    /// Flag to show the current configuration
    #[arg(long, help = "Display the current configuration")]
    pub show: bool,
}

pub fn run(args: &ConfigArgs) -> Result<()> {
    let flag_count = [args.locate, args.reset, args.show]
        .iter()
        .filter(|&&x| x)
        .count();

    log::info!(
        "Running config command with flags: locate={}, reset={}, show={}",
        args.locate,
        args.reset,
        args.show
    );

    if flag_count == 0 {
        display::warning("No action specified. Use one of: --locate, --reset, --show");
        log::warn!("No action specified. Use one of: --locate, --reset, --show");
        return Err(anyhow!(
            "No action specified. Use one of: --locate, --reset, --show"
        ));
    }

    if flag_count > 1 {
        display::error("Only one flag can be used at a time.");
        log::warn!("Multiple flags used. Only one flag can be used at a time.");
        return Err(anyhow!(
            "Only one flag can be used at a time. Please choose one of: --locate, --reset, --show"
        ));
    }

    let mut conf = context::get_locked_config()?;

    if args.locate {
        display::info("📍 Locating config file...");
        log::info!("Locating config file...");
        let path = Config::locate_config_file().context("Failed to locate config file")?;
        display::success(&format!("Config file found at: {}", path.display()));
        log::info!("Config file found at: {}", path.display());
    } else if args.reset {
        display::warning("🔄 Resetting config to default...");
        log::info!("Resetting config to default...");
        conf.reset_config()
            .context("Failed to reset config to default")?;
        display::success("Config reset to default values.");
        log::info!("Config reset complete.");
    } else if args.show {
        display::header("📋 Current Configuration");
        log::info!("Showing current config...");
        let config_str = conf.show_config(); // Assuming this can't fail
        println!("{config_str}");
        log::info!("Current config displayed successfully.");
    }

    Ok(())
}
