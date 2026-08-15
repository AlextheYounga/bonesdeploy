use anyhow::Result;

use crate::cli::args::{Cli, Command, ManifestFormat, ReleasesCommand, RemoteCommand, SecretsCommand};
use crate::commands::{
    deploy, doctor, init, manifest, migrate, releases, remote, rollback, secrets, setup, skill, status, update, version,
};
// ponytail: direct command dispatch keeps CLI routing visible; split only if commands need shared dispatch state.
#[expect(clippy::cognitive_complexity)]
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
        Command::Migrate => migrate::run(),
        Command::Skill { command } => skill::dispatch(command.as_ref()).await,
        Command::Guide { format } => skill::run_next(*format).await,
        Command::Secrets { command } => match command {
            SecretsCommand::Init => secrets::init(),
            SecretsCommand::Edit => secrets::edit(),
            SecretsCommand::Push => secrets::push().await,
        },
        Command::Deploy => deploy::run().await,
        Command::Releases { command } => match command {
            None => releases::list().await,
            Some(ReleasesCommand::Kill { release }) => releases::kill(release).await,
        },
        Command::Update { skip_local, skip_remote } => {
            update::run(update::Options { skip_local: *skip_local, skip_remote: *skip_remote }).await
        }
        Command::Remote { command } => match command {
            RemoteCommand::Bootstrap => remote::bootstrap::run(false, true),
            RemoteCommand::Runtime { yes } => remote::runtime::run(*yes, true),
            RemoteCommand::Ssl { yes, domain, email } => remote::ssl::run(*yes, domain.clone(), email.clone()),
            RemoteCommand::Helpers { yes } => remote::helpers::run(*yes),
            RemoteCommand::Services { yes } => remote::services::run(*yes, true),
        },
        Command::Rollback => rollback::run().await,
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
