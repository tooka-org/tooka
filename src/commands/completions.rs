use anyhow::Result;
use clap::Args;
use clap::CommandFactory;
use clap_complete::{generate, shells::Shell};
use std::io;

#[derive(Args)]
#[command(about = "Generate shell completions")]
pub struct CompletionsArgs {
    #[arg(value_enum)]
    /// The `shell` field specifies the target shell and must be provided as a value enum.
    pub shell: Shell,
}

pub fn run(args: &CompletionsArgs) -> Result<()> {
    log::info!("Generating completions for shell: {:?}", args.shell);

    // Check if the shell is supported by clap_complete::Shell
    let supported_shells = [
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Zsh,
    ];

    if !supported_shells.contains(&args.shell) {
        log::warn!(
            "Unsupported shell: {:?}. Supported shells are: {:?}",
            args.shell,
            supported_shells
        );
        return Err(anyhow::anyhow!(
            "Unsupported shell: {:?}. Supported shells are: {:?}",
            args.shell,
            supported_shells
        ));
    }

    let mut cmd = crate::Cli::command();
    generate(args.shell, &mut cmd, "tooka", &mut io::stdout());
    log::info!(
        "Completions generated successfully for shell: {:?}",
        args.shell
    );
    Ok(())
}
