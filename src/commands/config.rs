use crate::context;
use clap::Args;

#[derive(Args)]
#[command(about = "Manages the Tooka configuration file")]
pub struct ConfigArgs {
    #[arg(long)]
    pub locate: bool,

    #[arg(long)]
    pub reset: bool,

    #[arg(long)]
    pub show: bool,
}

pub fn run(args: &ConfigArgs) {
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
    let conf = context::get_config();
    let mut conf = match conf.lock() {
        Ok(guard) => guard,
        Err(_) => {
            println!("Failed to lock rules file");
            log::error!("Failed to lock rules file");
            return;
        }
    };

    match flag_count {
        0 => {
            log::error!("No action specified");
            println!("No action specified. Use one of: --locate, --init, --reset, --show");
        }
        1 => {
            if args.locate {
                log::info!("Locating config file...");
                println!("Locating config file...");
                match conf.locate_config_file() {
                    Ok(path) => {
                        log::info!("Config file found at: {}", path.display());
                        println!("Config file found at: {}", path.display());
                    }
                    Err(e) => {
                        log::error!("Error locating config file: {e}");
                        eprintln!("Error locating config file: {e}");
                    }
                }
            } else if args.reset {
                log::info!("Resetting config to default...");
                println!("Resetting config to default...");
                conf.reset_config();
                log::info!("Config reset to default values.");
                println!("Config reset to default values.");
            } else if args.show {
                log::info!("Showing current config...");
                println!("Current config contents:\n---\n");
                let config_str = conf.show_config();
                log::info!("Current config displayed successfully.");
                println!("{config_str}");
            }
        }
        _ => {
            log::warn!("Multiple flags specified; only one allowed.");
            println!("Only one flag can be used at a time.");
        }
    }
}
