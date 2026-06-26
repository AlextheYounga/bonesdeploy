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
        /// Site identifier (must match an imported site directory)
        #[arg(long)]
        site: String,
        /// Exact revision to deploy (defaults to the configured branch)
        #[arg(long)]
        revision: Option<String>,
    },
    /// Thin git-hook entrypoints
    Hook {
        #[command(subcommand)]
        command: HookCommand,
    },
    /// Import or export root-owned remote site state
    Site {
        #[command(subcommand)]
        command: SiteCommand,
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
    /// Print the version
    Version,
}

#[derive(Subcommand)]
pub enum HookCommand {
    /// Resolve a post-receive push and delegate deployment
    PostReceive {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum SiteCommand {
    /// Import a deployment dataset from stdin
    Import {
        #[arg(long)]
        site: String,
    },
    /// Export the current deployment dataset to stdout
    Export {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum ReleaseCommand {
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
    /// Prune old releases according to the registry keep count
    Prune {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum ServiceCommand {
    /// Restart the per-site nginx service
    Restart {
        #[arg(long)]
        site: String,
    },
}
