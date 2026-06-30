use anyhow::Result;

use crate::cli::args::{Cli, Command, RemoteCommand, SecretsCommand};
use crate::commands::{
    config, deploy_project, doctor, guide, init_config, init_project, push_state, remote_runtime, remote_setup,
    remote_ssl, rollback, secrets, setup, status, update, version,
};

pub async fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init { non_interactive, project_name, branch, remote, host, port } => {
            let remote_setup_ran = init_project::run(&init_config::InitArgs {
                non_interactive: *non_interactive,
                project_name: project_name.clone(),
                branch: branch.clone(),
                remote: remote.clone(),
                host: host.clone(),
                port: port.clone(),
            })?;
            if remote_setup_ran {
                push_state::run(false)?;
            }
            Ok(())
        }
        Command::Setup { yes } => setup::run(*yes).await,
        Command::Doctor { local } => doctor::run(*local).await,
        Command::Status => status::run().await,
        Command::Guide { format } => guide::run(*format).await,
        Command::Push => push_state::run(true),
        Command::Secrets { command } => match command {
            SecretsCommand::Init => secrets::init(),
            SecretsCommand::Edit => secrets::edit(),
            SecretsCommand::Push => secrets::push().await,
        },
        Command::Deploy => deploy_project::run().await,
        Command::Update { skip_local, skip_remote } => {
            update::run(update::Options { skip_local: *skip_local, skip_remote: *skip_remote }).await
        }
        Command::Remote { command } => match command {
            RemoteCommand::Bootstrap => remote_setup::run(false),
            RemoteCommand::Runtime { yes } => remote_runtime::run(*yes),
            RemoteCommand::Ssl { yes, domain, email } => remote_ssl::run(*yes, domain.clone(), email.clone()),
        },
        Command::Rollback => rollback::run().await,
        Command::Config { file, key } => config::run(file, key),
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
