mod activate_release;
mod deploy;
mod doctor;
mod drop_failed_release;
mod init;
mod post_deploy;
mod post_receive;
mod rollback;
mod service;
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
    /// Run the full remote deployment lifecycle
    Deploy {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
        /// Exact revision to check out into the build workspace
        #[arg(long)]
        revision: Option<String>,
    },
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
    /// Narrow privileged service operations (requires root)
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
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
    /// Wire shared paths into the build workspace
    Wire {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Atomically activate staged release
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
    /// Run deployment scripts and release activation sequence
    Deploy {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
    /// Run the post-receive checkout sequence
    PostReceive {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
        /// Exact revision to check out into the build workspace
        #[arg(long)]
        revision: Option<String>,
    },
    /// Prune old releases after deployment
    PostDeploy {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
}

#[derive(Subcommand)]
enum ServiceCommand {
    /// Restart the per-site nginx service
    Restart {
        /// Path to bones.toml config file
        #[arg(long)]
        config: String,
    },
}

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init => init::run(),
        Command::Doctor => doctor::run(),
        Command::Deploy { config, revision } => deploy::run_full(config, revision.as_deref()),
        Command::Release { command } => match command {
            ReleaseCommand::Stage { config } => stage_release::run(config),
            ReleaseCommand::Wire { config } => wire_release::run(config),
            ReleaseCommand::Activate { config } => activate_release::run(config),
            ReleaseCommand::DropFailed { config } => drop_failed_release::run(config),
            ReleaseCommand::Rollback { config } => rollback::run(config),
        },
        Command::Hooks { command } => match command {
            HookCommand::Deploy { config } => deploy::run(config),
            HookCommand::PostReceive { config, revision } => post_receive::run(config, revision.as_deref()),
            HookCommand::PostDeploy { config } => post_deploy::run(config),
        },
        Command::Service { command } => match command {
            ServiceCommand::Restart { config } => service::run(config),
        },
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
