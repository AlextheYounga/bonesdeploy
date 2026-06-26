use clap::{Parser, Subcommand, ValueEnum};

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
    /// Run the full first-time deployment setup
    Setup {
        /// Skip runtime confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Check local and remote environment health
    Doctor {
        /// Skip remote checks
        #[arg(long)]
        local: bool,
    },
    /// Show the current deployment state and next steps
    Status,
    /// Suggest the next prompt-free command to run
    Guide {
        /// Output format
        #[arg(long, value_enum, default_value_t = GuideFormat::Text)]
        format: GuideFormat,
    },
    /// Publish .bones/ into bonesremote's remote control-plane state
    Push,
    /// Recover .bones/ from bonesremote's remote control-plane state
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
    /// Run remote bootstrap against configured host
    #[command(alias = "setup")]
    Bootstrap,
    /// Apply the configured runtime against configured host
    Runtime {
        /// Skip runtime confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Obtain and configure SSL certificates with certbot
    Ssl {
        /// Skip SSL confirmation prompts
        #[arg(long)]
        yes: bool,
        /// Domain name for the certificate (e.g. app.example.com)
        #[arg(long)]
        domain: Option<String>,
        /// Email used for Let's Encrypt registration and notices
        #[arg(long)]
        email: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum GuideFormat {
    Text,
    Json,
}
