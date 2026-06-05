mod deploy;
mod doctor;
mod init;
mod init_config;
mod manage;
mod pull;
mod push;
mod remote_setup;
mod remote_setup_output;
mod remote_ssl;
mod rollback;
mod update;
mod update_release;
mod version;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bonesdeploy", about = "Git deployment scaffolding tool")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
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
        /// Template name (e.g. laravel, django)
        #[arg(long)]
        template: Option<String>,
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
    /// Run deployment hooks manually without pushing commits
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
    /// Print the version
    Version,
}

#[derive(Subcommand)]
enum RemoteCommand {
    /// Run remote setup playbook against configured host
    Setup,
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

pub async fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init { non_interactive, setup_remote, project_name, branch, remote, host, port, template } => {
            let outcome = init::run(&init::InitArgs {
                non_interactive: *non_interactive,
                setup_remote: *setup_remote,
                project_name: project_name.clone(),
                branch: branch.clone(),
                remote: remote.clone(),
                host: host.clone(),
                port: port.clone(),
                template: template.clone(),
            })?;
            if outcome.remote_setup_ran {
                push::run().await?;
            }
            Ok(())
        }
        Command::Doctor { local } => doctor::run(*local).await,
        Command::Push => push::run().await,
        Command::Pull => pull::run(),
        Command::Deploy => deploy::run().await,
        Command::Update { skip_local, skip_remote } => {
            update::run(update::UpdateOptions { skip_local: *skip_local, skip_remote: *skip_remote }).await
        }
        Command::Manage => manage::run(),
        Command::Remote { command } => match command {
            RemoteCommand::Setup => remote_setup::run(),
            RemoteCommand::Ssl { domain, email } => remote_ssl::run(domain.clone(), email.clone()),
        },
        Command::Rollback => rollback::run().await,
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
