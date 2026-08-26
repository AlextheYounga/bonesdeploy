use anyhow::Result;

use crate::cli::args::{Cli, Command, ManifestFormat, ReleasesCommand, SecretsCommand, ServerCommand, SiteCommand};
use crate::commands::{deploy, init, rollback, secrets, server, setup, site, skill, update, version};

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
        Command::Doctor { verbose } => run_doctor(*verbose).await,
        Command::Server { command } => dispatch_server(command).await,
        Command::Site { command } => dispatch_site(command).await,
        Command::Skill { command } => skill::dispatch(command.as_ref()).await,
        Command::Secrets { command } => dispatch_secrets(command).await,
        Command::Deploy => deploy::run().await,
        Command::Update { skip_local, skip_remote, continue_update } => {
            update::run(update::Options {
                skip_local: *skip_local,
                skip_remote: *skip_remote,
                continue_update: *continue_update,
            })
            .await
        }
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

async fn dispatch_server(command: &ServerCommand) -> Result<()> {
    match command {
        ServerCommand::Setup { yes } => server::setup(*yes).await,
        ServerCommand::Doctor { verbose } => server::doctor(*verbose).await,
        ServerCommand::Helpers { yes } => server::helpers(*yes),
    }
}

async fn dispatch_site(command: &SiteCommand) -> Result<()> {
    match command {
        SiteCommand::Setup { yes } => site::setup(*yes).await,
        SiteCommand::Doctor { local, verbose } => site::doctor(*local, *verbose).await,
        SiteCommand::Status => site::status().await,
        SiteCommand::Manifest { format } => site::manifest(match format {
            ManifestFormat::Text => "text",
            ManifestFormat::Json => "json",
        }),
        SiteCommand::Releases { command } => dispatch_releases(command.as_ref()).await,
        SiteCommand::Runtime { yes } => site::runtime(*yes),
        SiteCommand::Ssl { yes, domain, email } => site::ssl(*yes, domain.clone(), email.clone()),
        SiteCommand::Services { yes } => site::services(*yes),
    }
}

async fn dispatch_releases(command: Option<&ReleasesCommand>) -> Result<()> {
    site::releases(command).await
}

async fn run_doctor(verbose: bool) -> Result<()> {
    let server_result = server::doctor(verbose).await;
    let site_result = site::doctor(false, verbose).await;
    aggregate_doctor_results(server_result, site_result)
}

fn aggregate_doctor_results(server_result: Result<()>, site_result: Result<()>) -> Result<()> {
    match (server_result, site_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(server_error), Ok(())) => Err(server_error),
        (Ok(()), Err(site_error)) => Err(site_error),
        (Err(server_error), Err(site_error)) => {
            Err(anyhow::anyhow!("Server doctor failed: {server_error:#}\nSite doctor failed: {site_error:#}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::aggregate_doctor_results;

    #[test]
    fn root_doctor_reports_both_failures() {
        let result =
            aggregate_doctor_results(Err(anyhow::anyhow!("server failed")), Err(anyhow::anyhow!("site failed")));
        assert!(result.is_err(), "both doctor failures must fail the composed command");
        let message = match result {
            Err(error) => error.to_string(),
            Ok(()) => String::new(),
        };

        assert!(message.contains("Server doctor failed: server failed"));
        assert!(message.contains("Site doctor failed: site failed"));
    }
}
