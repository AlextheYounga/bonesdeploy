use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bonesremote", about = "Server-side git deployment tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
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
pub enum ReleaseCommand {
    /// Stage a new release before checkout
    Stage {
        #[arg(long)]
        config: String,
    },
    /// Wire shared paths into the build workspace
    Wire {
        #[arg(long)]
        config: String,
    },
    /// Atomically activate staged release
    Activate {
        #[arg(long)]
        config: String,
    },
    /// Drop a failed staged release and clear state
    DropFailed {
        #[arg(long)]
        config: String,
    },
    /// Repoint current to the previous release
    Rollback {
        #[arg(long)]
        config: String,
    },
}

#[derive(Subcommand)]
pub enum HookCommand {
    /// Run deployment scripts and release activation sequence
    Deploy {
        #[arg(long)]
        config: String,
    },
    /// Run the post-receive checkout sequence
    PostReceive {
        #[arg(long)]
        config: String,
        /// Exact revision to check out into the build workspace
        #[arg(long)]
        revision: Option<String>,
    },
    /// Prune old releases after deployment
    PostDeploy {
        #[arg(long)]
        config: String,
    },
}

#[derive(Subcommand)]
pub enum ServiceCommand {
    /// Restart the per-site nginx service
    Restart {
        #[arg(long)]
        config: String,
    },
}
