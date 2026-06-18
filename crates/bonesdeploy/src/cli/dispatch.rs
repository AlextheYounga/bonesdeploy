use anyhow::Result;

use crate::cli::args::{Cli, Command, RemoteCommand};
use crate::app::{remote_runtime, remote_setup, remote_ssl};
use crate::commands::{deploy, doctor, init, manage, pull, push, rollback, update, version};

pub async fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init { non_interactive, setup_remote, project_name, branch, remote, host, port } => {
            let outcome = init::run(&init::InitArgs {
                non_interactive: *non_interactive,
                setup_remote: *setup_remote,
                project_name: project_name.clone(),
                branch: branch.clone(),
                remote: remote.clone(),
                host: host.clone(),
                port: port.clone(),
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
            RemoteCommand::Runtime => remote_runtime::run(),
            RemoteCommand::Ssl { domain, email } => remote_ssl::run(domain.clone(), email.clone()),
        },
        Command::Rollback => rollback::run().await,
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
