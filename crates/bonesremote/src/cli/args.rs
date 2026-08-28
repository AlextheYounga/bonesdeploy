use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bonesremote", about = "Remote release deployment tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Check server environment health
    Doctor {
        /// Also validate the imported site state and runtime boundary for one site
        #[arg(long)]
        site: Option<String>,
        /// Recursively inspect every file in the active release for permission drift
        #[arg(long, requires = "site")]
        exhaustive: bool,
    },
    /// Run the full remote deployment lifecycle
    Deploy {
        /// Site identifier (must match a provisioned site directory)
        #[arg(long)]
        site: String,
        /// Exact revision to deploy (defaults to the configured branch)
        #[arg(long)]
        revision: Option<String>,
        /// Read the deployment config descriptor as JSON from stdin
        #[arg(long)]
        config_stdin: bool,
    },
    /// Synchronize the sanitized site configuration snapshot
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Print remote deployment status as JSON
    Status {
        #[arg(long)]
        site: String,
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
    /// Manage a BonesDeploy-generated application runtime (requires root)
    Runtime {
        #[command(subcommand)]
        command: RuntimeCommand,
    },
    /// Scheduled shared-data backup operations (requires root)
    Backup {
        #[command(subcommand)]
        command: BackupCommand,
    },
    /// Print the version
    Version,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// Install a site configuration snapshot from stdin
    Sync {
        #[arg(long)]
        site: String,
        #[arg(long, required = true)]
        config_stdin: bool,
    },
}

#[derive(Subcommand)]
pub enum RuntimeCommand {
    /// Start the configured Docker application runtime
    Start {
        #[arg(long)]
        site: String,
    },
    /// Stop the configured Docker application runtime
    Stop {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum ReleaseCommand {
    /// Print releases and active deployment state as JSON
    List {
        #[arg(long)]
        site: String,
    },
    /// Cancel a building or interrupted release and clean its temporary state
    Kill {
        #[arg(long)]
        site: String,
        #[arg(long)]
        release: String,
    },
    /// Repoint current to the previous release
    Rollback {
        #[arg(long)]
        site: String,
    },
    /// Drop the staged release and clean state
    DropFailed {
        #[arg(long)]
        site: String,
    },
    /// Prune old releases, keeping the most recent `keep` count
    Prune {
        #[arg(long)]
        site: String,
        #[arg(long, default_value_t = 5)]
        keep: usize,
    },
    /// Quarantine malformed deployment state after verifying no deployment is running
    Recover {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum ServiceCommand {
    /// Restart all services registered with the per-site lifecycle target
    Restart {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum BackupCommand {
    /// Create a shared-data archive and prune archives outside the retention window
    Run {
        #[arg(long)]
        site: String,
        /// Age-based retention window in days
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u16).range(1..))]
        keep_days: u16,
    },
}
