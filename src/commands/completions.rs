use clap::Args;
use clap::CommandFactory;
use clap_complete::{generate, shells::Shell};
use std::io;

#[derive(Args)]
#[command(about = "Generate shell completions")]
pub struct CompletionsArgs {
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn run(args: CompletionsArgs) {
    let mut cmd = crate::Cli::command();
    generate(args.shell, &mut cmd, "tooka", &mut io::stdout());
}
