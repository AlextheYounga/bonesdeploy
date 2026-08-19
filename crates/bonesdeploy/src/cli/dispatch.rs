use anyhow::Result;

use crate::cli::args::{Cli, Command, ManifestFormat, ReleasesCommand, RemoteCommand, SecretsCommand};
use crate::commands::{
    deploy, doctor, init, manifest, releases, remote, rollback, secrets, setup, skill, status, update, version,
};
pub async fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init {
            non_interactive,
            project_name,
            branch,
            remote,
            host,
            port,
            template,
            runtime_backend,
            framework_vars,
            services,
        } => {
            init::run(&init::Args {
                non_interactive: *non_interactive,
                project_name: project_name.clone(),
                branch: branch.clone(),
                remote: remote.clone(),
                host: host.clone(),
                port: port.clone(),
                template: template.clone(),
                runtime_backend: runtime_backend.clone(),
                framework_vars: framework_vars.clone(),
                services: services.clone(),
            })?;
            Ok(())
        }
        Command::Setup { yes } => setup::run(*yes).await,
        Command::Doctor { local, verbose } => doctor::run(*local, *verbose).await.map(|_| ()),
        Command::Status => status::run().await,
        Command::Manifest { format } => manifest::run(match format {
            ManifestFormat::Text => "text",
            ManifestFormat::Json => "json",
        }),
        Command::Skill { command } => skill::dispatch(command.as_ref()).await,
        Command::Guide { format } => skill::run_next(*format).await,
        Command::Secrets { command } => dispatch_secrets(command).await,
        Command::Deploy => deploy::run().await,
        Command::Releases { command } => dispatch_releases(command).await,
        Command::Update { skip_local, skip_remote } => {
            update::run(update::Options { skip_local: *skip_local, skip_remote: *skip_remote }).await
        }
        Command::Remote { command } => dispatch_remote(command),
        Command::Rollback => rollback::run().await,
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}

async fn dispatch_secrets(command: &SecretsCommand) -> Result<()> {
    match command {
        SecretsCommand::Init => secrets::init(),
        SecretsCommand::Edit => secrets::edit(),
        SecretsCommand::Push => secrets::push().await,
    }
}

async fn dispatch_releases(command: &Option<ReleasesCommand>) -> Result<()> {
    match command {
        None => releases::list().await,
        Some(ReleasesCommand::Kill { release }) => releases::kill(release).await,
    }
}

fn dispatch_remote(command: &RemoteCommand) -> Result<()> {
    match command {
        RemoteCommand::Bootstrap => remote::bootstrap::run(false, true),
        RemoteCommand::Runtime { yes } => remote::runtime::run(*yes, true),
        RemoteCommand::Ssl { yes, domain, email } => remote::ssl::run(*yes, domain.clone(), email.clone()),
        RemoteCommand::Helpers { yes } => remote::helpers::run(*yes),
        RemoteCommand::Services { yes } => remote::services::run(*yes, true),
    }
}
