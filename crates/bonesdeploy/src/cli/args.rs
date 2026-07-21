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
        /// Runtime template (laravel, django, next, nuxt, rails, sveltekit, vue, or none)
        #[arg(long)]
        template: Option<String>,
        /// Runtime variable override, repeated (e.g. `--runtime-var php_version=8.5`)
        #[arg(long = "runtime-var", value_name = "KEY=VALUE")]
        runtime_vars: Vec<String>,
        /// Database service to provision, repeated (postgres, mariadb, mysql, mongodb, valkey, redis)
        #[arg(long = "db", value_name = "SERVICE")]
        dbs: Vec<String>,
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
    /// Embedded documentation and next-step guidance for AI agents
    Skill {
        /// Optional subcommand: `next`, `list`, or `doc <name>`
        #[command(subcommand)]
        command: Option<SkillCommand>,
    },
    #[command(hide = true)]
    Guide {
        #[arg(long, value_enum, default_value_t = GuideFormat::Text)]
        format: GuideFormat,
    },
    /// Publish .bones/ into bonesremote's remote control-plane state
    Push,
    /// Manage encrypted local secrets and push them to remote shared/
    Secrets {
        #[command(subcommand)]
        command: SecretsCommand,
    },
    /// Deploy the configured project release to the remote server
    Deploy,
    /// List remote releases and their deployment state
    Releases {
        #[command(subcommand)]
        command: Option<ReleasesCommand>,
    },
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
    /// Roll back current release to the previous one
    Rollback,
    /// Read a value from .bones/bones.toml, or dump the whole file when no key is given
    Config {
        /// Path to TOML config file (default: .bones/bones.toml)
        #[arg(long)]
        file: Option<String>,
        /// Key to read; omit to dump the whole file
        key: Option<String>,
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
pub enum ReleasesCommand {
    /// Cancel a building or interrupted release and clean its temporary state
    Kill {
        /// Release identifier shown by `bonesdeploy releases`
        release: String,
    },
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
    /// Install helper tools on the remote host (starship, neovim, aptui, etc.)
    Helpers {
        /// Skip helper installation confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Provision configured database services (bound to localhost only)
    Dbs {
        /// Skip database setup confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum SkillCommand {
    /// Suggest the next prompt-free command to run, based on actual state
    Next {
        /// Output format
        #[arg(long, value_enum, default_value_t = GuideFormat::Text)]
        format: GuideFormat,
    },
    /// List every embedded skill doc by name
    List,
    /// Print a specific embedded skill doc
    Doc {
        /// Doc name (see `bonesdeploy skill list`)
        name: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum GuideFormat {
    Text,
    Json,
}
