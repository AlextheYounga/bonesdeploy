mod deploy;
mod doctor;
mod init;
mod manage;
mod pull;
mod push;
mod remote_setup;
mod remote_ssl;
mod rollback;
mod update;
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
    /// Set up bonesdeploy in the current repository
    Init,
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
        Command::Init => init::run(),
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
