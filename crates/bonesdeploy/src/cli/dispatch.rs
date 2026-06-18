use anyhow::Result;

use crate::cli::args::{Cli, Command, RemoteCommand};
use crate::app::{
    deploy_project, doctor, init_project, manage, pull_state, push_state, remote_runtime, remote_setup, remote_ssl,
    rollback, update, version,
};
use crate::commands::{config, init_config};

pub async fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init { non_interactive, setup_remote, project_name, branch, remote, host, port } => {
            let outcome = init_project::run(&init_config::InitArgs {
                non_interactive: *non_interactive,
                setup_remote: *setup_remote,
                project_name: project_name.clone(),
                branch: branch.clone(),
                remote: remote.clone(),
                host: host.clone(),
                port: port.clone(),
            })?;
            if outcome.remote_setup_ran {
                push_state::run().await?;
            }
            Ok(())
        }
        Command::Doctor { local } => doctor::run(*local).await,
        Command::Push => push_state::run().await,
        Command::Pull => pull_state::run(),
        Command::Deploy => deploy_project::run().await,
        Command::Update { skip_local, skip_remote } => {
            update::run(update::Options { skip_local: *skip_local, skip_remote: *skip_remote }).await
        }
        Command::Manage => manage::run(),
        Command::Remote { command } => match command {
            RemoteCommand::Setup => remote_setup::run(),
            RemoteCommand::Runtime => remote_runtime::run(),
            RemoteCommand::Ssl { domain, email } => remote_ssl::run(domain.clone(), email.clone()),
        },
        Command::Rollback => rollback::run().await,
        Command::Config { file, key } => config::run(file, key),
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
