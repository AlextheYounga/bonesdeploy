mod activate_release;
mod cleanup_failed_release;
mod doctor;
mod init;
mod post_deploy;
mod prepare_release;
mod prime_release;
mod rollback;
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
    /// Prepare a new pending release before checkout
    PrepareRelease {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Prime shared paths into the pending release
    PrimeRelease {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Atomically activate pending release and prune old releases
    ActivateRelease {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Remove pending failed release and clear state
    CleanupFailedRelease {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Repoint current to the previous release
    Rollback {
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
        Command::PrepareRelease { config } => prepare_release::run(config),
        Command::PrimeRelease { config } => prime_release::run(config),
        Command::ActivateRelease { config } => activate_release::run(config),
        Command::CleanupFailedRelease { config } => cleanup_failed_release::run(config),
        Command::Rollback { config } => rollback::run(config),
        Command::PostDeploy { config } => post_deploy::run(config),
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
