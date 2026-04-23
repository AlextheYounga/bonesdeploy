mod activate_release;
mod doctor;
mod drop_failed_release;
mod init;
mod post_deploy;
mod rollback;
mod stage_release;
mod version;
mod wire_release;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bonesremote", about = "Server-side git deployment tool")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install sudoers drop-in for passwordless bonesremote
    Init,
    /// Check server environment health
    Doctor,
    /// Release lifecycle operations
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
    /// Hook-oriented helper commands
    Hooks {
        #[command(subcommand)]
        command: HookCommand,
    },
    /// Print the version
    Version,
}

#[derive(Subcommand)]
enum ReleaseCommand {
    /// Stage a new release before checkout
    Stage {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Wire shared paths into the staged release
    Wire {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Atomically activate staged release and prune old releases
    Activate {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Drop a failed staged release and clear state
    DropFailed {
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
}

#[derive(Subcommand)]
enum HookCommand {
    /// Harden permissions back to service user after deployment
    PostDeploy {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
}

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init => init::run(),
        Command::Doctor => doctor::run(),
        Command::Release { command } => match command {
            ReleaseCommand::Stage { config } => stage_release::run(config),
            ReleaseCommand::Wire { config } => wire_release::run(config),
            ReleaseCommand::Activate { config } => activate_release::run(config),
            ReleaseCommand::DropFailed { config } => drop_failed_release::run(config),
            ReleaseCommand::Rollback { config } => rollback::run(config),
        },
        Command::Hooks { command } => match command {
            HookCommand::PostDeploy { config } => post_deploy::run(config),
        },
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
