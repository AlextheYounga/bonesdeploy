mod deploy;
mod doctor;
mod init;
mod push;
mod rollback;
mod server_setup;
mod server_ssl;
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
    /// Run deployment hooks manually without pushing commits
    Deploy,
    /// Server setup operations
    Server {
        #[command(subcommand)]
        command: ServerCommand,
    },
    /// Roll back current release to the previous one
    Rollback,
    /// Print the version
    Version,
}

#[derive(Subcommand)]
enum ServerCommand {
    /// Run server setup playbook against configured host
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
        Command::Init => init::run().await,
        Command::Doctor { local } => doctor::run(*local).await,
        Command::Push => push::run().await,
        Command::Deploy => deploy::run().await,
        Command::Server { command } => match command {
            ServerCommand::Setup => server_setup::run(),
            ServerCommand::Ssl { domain, email } => server_ssl::run(domain.clone(), email.clone()),
        },
        Command::Rollback => rollback::run().await,
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
