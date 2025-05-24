use clap::Args;
use crate::core::config;

#[derive(Args)]
#[command(about = "Manages the Tooka configuration file")]
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

    match flag_count {
        0 => {
            eprintln!("❌ No action specified. Use one of: --locate, --init, --reset, --show");
        }
        1 => {
            if args.locate {
                println!("🔍 Locating config file...");
                match config::locate_config_file() {
                    Ok(path) => println!("Config file found at: {}", path.display()),
                    Err(e) => eprintln!("❌ Error locating config file: {}", e),
                }
            } else if args.init {
                println!("🛠️ Initializing config file...");
                match config::Config::load() {
                    Ok(_) => println!("✅ Config file initialized successfully!"),
                    Err(e) => eprintln!("❌ Error initializing config file: {}", e),
                }
            } else if args.reset {
                println!("🔄 Resetting config to default...");
                match config::reset_config() {
                    Ok(_) => println!("✅ Config reset to default successfully!"),
                    Err(e) => eprintln!("❌ Error resetting config: {}", e),
                }
            } else if args.show {
                println!("📄 Current config contents:\n---\n<YAML output here>");
                match config::show_config() {
                    Ok(contents) => println!("{}", contents),
                    Err(e) => eprintln!("❌ Error showing config: {}", e),
                }
            }
        }
        _ => {
            eprintln!("❌ Only one flag can be used at a time.");
        }
    }
}
