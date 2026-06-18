use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bonesremote", about = "Remote release deployment tool")]
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
    /// Narrow privileged service operations (requires root)
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },
    /// Get a config value from a TOML file
    Config {
        /// Path to TOML config file
        #[arg(long)]
        file: String,
        /// Key to read
        key: String,
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
pub enum ServiceCommand {
    /// Restart the per-site nginx service
    Restart {
        #[arg(long)]
        config: String,
    },
}
