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
    /// Deploy a Docker Compose source revision or receive a native artifact
    Deploy {
        /// Site identifier (must match a provisioned site directory)
        #[arg(long)]
        site: String,
        /// Receive a framed local-build artifact from standard input for native sites
        #[arg(long)]
        artifact_stdin: bool,
    },
    /// Synchronize the sanitized site configuration snapshot
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Persist and finalize remote site decommissioning state
    Decommission {
        #[command(subcommand)]
        command: DecommissionCommand,
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
    /// Manage a Docker Compose runtime (requires root)
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
    /// Install a site configuration snapshot read from stdin
    Sync {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum DecommissionCommand {
    /// Persist a deletion plan read as JSON from stdin and print the stored plan
    Begin {
        #[arg(long)]
        site: String,
    },
    /// Replace mutable deployment state with an unverified deletion tombstone
    Complete {
        #[arg(long)]
        site: String,
    },
    /// Mark a completed deletion tombstone verified
    Verify {
        #[arg(long)]
        site: String,
    },
    /// Clear a verified deletion tombstone after successful site setup
    Reactivate {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum RuntimeCommand {
    /// Start the configured Docker Compose stack
    Start {
        #[arg(long)]
        site: String,
    },
    /// Stop the configured Docker Compose stack without removing volumes
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
