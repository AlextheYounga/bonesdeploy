use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bonesdeploy", about = "Remote release deployment tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Set up bonesdeploy in the current repository. Run this once per project.
    Init {
        /// Skip all interactive prompts; required fields must be provided via flags
        #[arg(long)]
        non_interactive: bool,
        /// Run remote setup after init (instead of prompting)
        #[arg(long)]
        setup_remote: bool,
        /// Project name (default: current directory name)
        #[arg(long)]
        project_name: Option<String>,
        /// Git branch to deploy
        #[arg(long)]
        branch: Option<String>,
        /// Deployment remote name (default: production)
        #[arg(short = 'r', long)]
        remote: Option<String>,
        /// Server hostname or IP
        #[arg(short = 'H', long)]
        host: Option<String>,
        /// SSH port (default: 22)
        #[arg(long)]
        port: Option<String>,
    },
    /// Check local and remote environment health
    Doctor {
        /// Skip remote checks
        #[arg(long)]
        local: bool,
    },
    /// Sync .bones/ folder to the remote bare repo
    Push,
    /// Sync .bones/ folder back from the remote bare repo
    Pull,
    /// Manage encrypted local secrets and push them to remote shared/
    Secrets {
        #[command(subcommand)]
        command: SecretsCommand,
    },
    /// Deploy the configured project release to the remote server
    Deploy,
    /// Update bonesdeploy and bonesremote to the latest version
    Update {
        /// Skip local update
        #[arg(long)]
        skip_local: bool,
        /// Skip remote update
        #[arg(long)]
        skip_remote: bool,
    },
    /// Remote operations
    Remote {
        #[command(subcommand)]
        command: RemoteCommand,
    },
    /// Open remote server management TUI
    Manage,
    /// Roll back current release to the previous one
    Rollback,
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
pub enum SecretsCommand {
    /// Create the local secrets config and storage directory
    Init,
    /// Decrypt the .env secret, edit it, then re-encrypt it
    Edit,
    /// Decrypt local secrets and write them into remote shared/
    Push,
}

#[derive(Subcommand)]
pub enum RemoteCommand {
    /// Run remote setup against configured host
    Setup,
    /// Apply the configured runtime against configured host
    Runtime,
    /// Obtain and configure SSL certificates with certbot
    Ssl {
        /// Domain name for the certificate (e.g. app.example.com)
        #[arg(long)]
        domain: Option<String>,
        /// Email used for Let's Encrypt registration and notices
        #[arg(long)]
        email: Option<String>,
    },
}
