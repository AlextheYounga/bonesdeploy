use anyhow::Result;

use crate::cli::args::{Cli, Command, ReleasesCommand, RemoteCommand, SecretsCommand};
use crate::commands::{
    config, deploy_project, doctor, init, push_state, releases, remote_bootstrap, remote_helpers, remote_runtime,
    remote_ssl, rollback, secrets, setup, skill, status, update, version,
};
pub async fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init { non_interactive, project_name, branch, remote, host, port, template, runtime_vars } => {
            init::run(&init::Args {
                non_interactive: *non_interactive,
                project_name: project_name.clone(),
                branch: branch.clone(),
                remote: remote.clone(),
                host: host.clone(),
                port: port.clone(),
                template: template.clone(),
                runtime_vars: runtime_vars.clone(),
            })?;
            Ok(())
        }
        Command::Setup { yes } => setup::run(*yes).await,
        Command::Doctor { local } => doctor::run(*local).await.map(|_| ()),
        Command::Status => status::run().await,
        Command::Skill { command } => skill::dispatch(command.as_ref()).await,
        Command::Push => push_state::run(true),
        Command::Secrets { command } => match command {
            SecretsCommand::Init => secrets::init(),
            SecretsCommand::Edit => secrets::edit(),
            SecretsCommand::Push => secrets::push().await,
        },
        Command::Deploy => deploy_project::run().await,
        Command::Releases { command } => match command {
            None => releases::list().await,
            Some(ReleasesCommand::Kill { release }) => releases::kill(release).await,
        },
        Command::Update { skip_local, skip_remote } => {
            update::run(update::Options { skip_local: *skip_local, skip_remote: *skip_remote }).await
        }
        Command::Remote { command } => match command {
            RemoteCommand::Bootstrap => remote_bootstrap::run(false),
            RemoteCommand::Runtime { yes } => remote_runtime::run(*yes),
            RemoteCommand::Ssl { yes, domain, email } => remote_ssl::run(*yes, domain.clone(), email.clone()),
            RemoteCommand::Helpers { yes } => remote_helpers::run(*yes),
        },
        Command::Rollback => rollback::run().await,
        Command::Config { file, key } => config::run(file.as_deref(), key.as_deref()),
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
