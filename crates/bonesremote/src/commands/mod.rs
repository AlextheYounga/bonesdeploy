mod activate_release;
mod deploy;
mod doctor;
mod drop_failed_release;
mod init;
mod landlock_exec;
mod post_deploy;
mod post_receive;
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
    Init {
        /// Deploy user allowed to run privileged bonesremote commands
        #[arg(long, default_value = "git")]
        deploy_user: String,
    },
    /// Check server environment health
    Doctor {
        /// Optional path to bones.yaml for runtime checks
        #[arg(long)]
        config: Option<String>,
    },
    /// Runtime isolation launcher commands
    Landlock {
        #[command(subcommand)]
        command: LandlockCommand,
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
    /// Print the version
    Version,
}

#[derive(Subcommand)]
enum ReleaseCommand {
    /// Stage a new release before checkout
    Stage {
        /// Path to bones.yaml config file
        #[arg(long)]
        config: String,
    },
    /// Wire shared paths into the staged release
    Wire {
        /// Path to bones.yaml config file
        #[arg(long)]
        config: String,
    },
    /// Atomically activate staged release and prune old releases
    Activate {
        /// Path to bones.yaml config file
        #[arg(long)]
        config: String,
    },
    /// Drop a failed staged release and clear state
    DropFailed {
        /// Path to bones.yaml config file
        #[arg(long)]
        config: String,
    },
    /// Repoint current to the previous release
    Rollback {
        /// Path to bones.yaml config file
        #[arg(long)]
        config: String,
    },
}

#[derive(Subcommand)]
enum HookCommand {
    /// Run deployment scripts and release activation sequence
    Deploy {
        /// Path to bones.yaml config file
        #[arg(long)]
        config: String,
    },
    /// Run the post-receive checkout and release wiring sequence
    PostReceive {
        /// Path to bones.yaml config file
        #[arg(long)]
        config: String,
    },
    /// Harden permissions back to service user after deployment
    PostDeploy {
        /// Path to bones.yaml config file
        #[arg(long)]
        config: String,
    },
}

#[derive(Subcommand)]
enum LandlockCommand {
    /// Apply Landlock and exec the runtime command
    Exec {
        /// Path to bones.yaml config file
        #[arg(long)]
        config: String,
    },
}

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init { deploy_user } => init::run(deploy_user),
        Command::Doctor { config } => doctor::run(config.as_deref()),
        Command::Landlock { command } => match command {
            LandlockCommand::Exec { config } => landlock_exec::run(config),
        },
        Command::Release { command } => match command {
            ReleaseCommand::Stage { config } => stage_release::run(config),
            ReleaseCommand::Wire { config } => wire_release::run(config),
            ReleaseCommand::Activate { config } => activate_release::run(config),
            ReleaseCommand::DropFailed { config } => drop_failed_release::run(config),
            ReleaseCommand::Rollback { config } => rollback::run(config),
        },
        Command::Hooks { command } => match command {
            HookCommand::Deploy { config } => deploy::run(config),
            HookCommand::PostReceive { config } => post_receive::run(config),
            HookCommand::PostDeploy { config } => post_deploy::run(config),
        },
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
