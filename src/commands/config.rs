use clap::Args;
use crate::core::config;

#[derive(Args)]
#[command(about = "Manages the Tooka configuration file")]
/// Command-line arguments for configuration-related commands.
///
/// This struct defines the available flags for configuration operations:
/// - `locate`: Locate the configuration file.
/// - `init`: Initialize a new configuration.
/// - `reset`: Reset the configuration to default values.
/// - `show`: Display the current configuration.
pub struct ConfigArgs {
    #[arg(long)]
    pub locate: bool,

    #[arg(long)]
    pub init: bool,

    #[arg(long)]
    pub reset: bool,

    #[arg(long)]
    pub show: bool,
}

pub fn run(args: ConfigArgs) {
    let flag_count = [args.locate, args.init, args.reset, args.show]
        .iter()
        .filter(|&&x| x)
        .count();

    log::info!("Running config command with flags: locate={}, init={}, reset={}, show={}",
               args.locate, args.init, args.reset, args.show);

    match flag_count {
        0 => {
            log::error!("No action specified");
            println!("❌ No action specified. Use one of: --locate, --init, --reset, --show");
        }
        1 => {
            if args.locate {
                log::info!("Locating config file...");
                println!("🔍 Locating config file...");
                match config::locate_config_file() {
                    Ok(path) => {
                        log::info!("Config file found at: {}", path.display());
                        println!("Config file found at: {}", path.display());
                    }
                    Err(e) => {
                        log::error!("Error locating config file: {}", e);
                        eprintln!("❌ Error locating config file: {}", e);
                    }
                }
            } else if args.init {
                log::info!("Initializing config file...");
                println!("🛠️ Initializing config file...");
                match config::Config::load() {
                    Ok(_) => {
                        log::info!("Config file initialized successfully!");
                        println!("✅ Config file initialized successfully!");
                    }
                    Err(e) => {
                        log::error!("Error initializing config file: {}", e);
                        eprintln!("❌ Error initializing config file: {}", e);
                    }
                }
            } else if args.reset {
                log::info!("Resetting config to default...");
                println!("🔄 Resetting config to default...");
                match config::reset_config() {
                    Ok(_) => {
                        log::info!("Config reset to default successfully!");
                        println!("✅ Config reset to default successfully!");
                    }
                    Err(e) => {
                        log::error!("Error resetting config: {}", e);
                        eprintln!("❌ Error resetting config: {}", e);
                    }
                }
            } else if args.show {
                log::info!("Showing current config...");
                println!("📄 Current config contents:\n---\n<YAML output here>");
                match config::show_config() {
                    Ok(contents) => {
                        log::info!("Config contents displayed.");
                        println!("{}", contents);
                    }
                    Err(e) => {
                        log::error!("Error showing config: {}", e);
                        eprintln!("❌ Error showing config: {}", e);
                    }
                }
            }
        }
        _ => {
            log::warn!("Multiple flags specified; only one allowed.");
            println!("❌ Only one flag can be used at a time.");
        }
    }
}
