use crate::context;
use anyhow::{anyhow, Context, Result};
use clap::Args;

#[derive(Args)]
#[command(about = "Manages the Tooka configuration file")]
pub struct ConfigArgs {
    /// Flag to locate the configuration file
    #[arg(long)]
    pub locate: bool,

    /// Flag to reset the configuration file to default values
    #[arg(long)]
    pub reset: bool,

    /// Flag to show the current configuration
    #[arg(long)]
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
        log::warn!("No action specified. Use one of: --locate, --reset, --show");
        return Err(anyhow!(
            "No action specified. Use one of: --locate, --reset, --show"
        ));
    }

    if flag_count > 1 {
        log::warn!("Multiple flags used. Only one flag can be used at a time.");
        return Err(anyhow!(
            "Only one flag can be used at a time. Please choose one of: --locate, --reset, --show"
        ));
    }

    let mut conf = context::get_locked_config()?;

    if args.locate {
        log::info!("Locating config file...");
        let path = conf
            .locate_config_file()
            .context("Failed to locate config file")?;
        println!("Config file found at: {}", path.display());
        log::info!("Config file found at: {}", path.display());
    } else if args.reset {
        log::info!("Resetting config to default...");
        conf.reset_config().context("Failed to reset config to default")?;
        println!("Config reset to default values.");
        log::info!("Config reset complete.");
    } else if args.show {
        log::info!("Showing current config...");
        let config_str = conf.show_config(); // Assuming this can't fail
        println!("{config_str}");
        log::info!("Current config displayed successfully.");
    }

    Ok(())
}
