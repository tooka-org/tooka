use clap::Args;

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
                println!("📍 Config file is located at: <path/to/config.yaml>");
            } else if args.init {
                println!("🛠️ Initializing config file...");
            } else if args.reset {
                println!("🔄 Resetting config to default...");
            } else if args.show {
                println!("📄 Current config contents:\n---\n<YAML output here>");
            }
        }
        _ => {
            eprintln!("❌ Only one flag can be used at a time.");
        }
    }
}
