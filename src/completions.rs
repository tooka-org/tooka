use anyhow::Result;
use clap::Args;
use clap::CommandFactory;
use clap_complete::{generate, shells::Shell};
use std::io;

#[derive(Args)]
#[command(about = "🔧 Generate shell completions")]
pub struct CompletionsArgs {
    /// The `shell` field specifies the target shell and must be provided as a value enum.
    #[arg(value_enum, help = "Target shell for completion generation")]
    pub shell: Shell,
}

pub fn run(args: &CompletionsArgs) -> Result<()> {
    log::info!("Generating completions for shell: {:?}", args.shell);

    let mut cmd = crate::Cli::command();
    generate(args.shell, &mut cmd, "tooka", &mut io::stdout());
    log::info!(
        "Completions generated successfully for shell: {:?}",
        args.shell
    );
    Ok(())
}
