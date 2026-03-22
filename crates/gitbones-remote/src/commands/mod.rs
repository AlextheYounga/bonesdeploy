mod doctor;
mod init;
mod post_deploy;
mod pre_deploy;
mod version;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gitbones-remote", about = "Server-side git deployment tool")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install sudoers drop-in for passwordless gitbones-remote
    Init,
    /// Check server environment health
    Doctor,
    /// Change worktree ownership to deploy user before deployment
    PreDeploy {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Harden permissions back to service user after deployment
    PostDeploy {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Print the version
    Version,
}

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init => init::run(),
        Command::Doctor => doctor::run(),
        Command::PreDeploy { config } => pre_deploy::run(config),
        Command::PostDeploy { config } => post_deploy::run(config),
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
