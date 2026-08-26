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
        /// Framework template (laravel, django, next, nuxt, rails, sveltekit, vue, or none)
        #[arg(long)]
        template: Option<String>,
        /// Application runtime backend (native or docker; default: native)
        #[arg(long, value_parser = ["native", "docker"])]
        runtime_backend: Option<String>,
        /// Framework variable override, repeated (e.g. `--framework-var php_version=8.5`)
        #[arg(long = "framework-var", value_name = "KEY=VALUE")]
        framework_vars: Vec<String>,
        /// Optional service to provision, repeated (postgres, mariadb, mysql, mongodb, valkey, redis)
        #[arg(long = "service", value_name = "SERVICE")]
        services: Vec<String>,
    },
    /// Run server setup followed by site setup
    Setup {
        /// Skip setup confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Run server and site diagnostics
    Doctor {
        /// Show all successful remote checks
        #[arg(long)]
        verbose: bool,
    },
    /// Server-wide provisioning and diagnostics
    Server {
        #[command(subcommand)]
        command: ServerCommand,
    },
    /// Project-scoped provisioning, diagnostics, and inspection
    Site {
        #[command(subcommand)]
        command: SiteCommand,
    },
    /// Embedded documentation and next-step guidance for AI agents
    Skill {
        /// Optional subcommand: `next`, `list`, or `doc <name>`
        #[command(subcommand)]
        command: Option<SkillCommand>,
    },
    /// Manage encrypted local secrets and push them to remote shared/
    Secrets {
        #[command(subcommand)]
        command: SecretsCommand,
    },
    /// Deploy the configured project release to the remote server
    Deploy,
    /// Update BonesDeploy, BonesRemote, and project infrastructure to the latest version
    Update {
        /// Skip local update
        #[arg(long)]
        skip_local: bool,
        /// Skip remote update
        #[arg(long)]
        skip_remote: bool,
        #[arg(long, hide = true)]
        continue_update: bool,
    },
    /// Roll back current release to the previous one
    Rollback,
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
        /// Release identifier shown by `bonesdeploy site releases`
        release: String,
    },
}

#[derive(Subcommand)]
pub enum ServerCommand {
    /// Provision the shared server baseline
    Setup {
        /// Skip setup confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Check the shared server baseline
    Doctor {
        /// Show all successful remote checks
        #[arg(long)]
        verbose: bool,
    },
    /// Install optional helper tools on the server
    Helpers {
        /// Skip helper installation confirmation prompts
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum SiteCommand {
    /// Provision one project's base, services, runtime, and diagnostics
    Setup {
        /// Skip setup confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Check one project's local and remote health
    Doctor {
        /// Skip remote checks
        #[arg(long)]
        local: bool,
        /// Show all successful remote checks
        #[arg(long)]
        verbose: bool,
    },
    /// Show the current deployment state and next steps
    Status,
    /// Inspect project-owned remote deployment artifacts
    Manifest {
        /// Output format
        #[arg(long, value_enum, default_value_t = ManifestFormat::Text)]
        format: ManifestFormat,
    },
    /// List remote releases and their deployment state
    Releases {
        #[command(subcommand)]
        command: Option<ReleasesCommand>,
    },
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
    /// Provision configured services (bound to localhost only)
    Services {
        /// Skip service setup confirmation prompt
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
pub enum ManifestFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum GuideFormat {
    Text,
    Json,
}
