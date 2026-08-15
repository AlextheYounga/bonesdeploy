use anyhow::Result;

use crate::cli::args::{Cli, Command, ReleaseCommand, RuntimeCommand, ServiceCommand};
use crate::commands::{deploy, doctor, drop_failed_release, release, service, status, version};
use crate::release::SiteMutation;
use crate::runtime::docker;

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Doctor { site, exhaustive } => doctor::run(site.as_deref(), *exhaustive),
        Command::Deploy { site, revision } => deploy::run_full(site, revision.as_deref()),
        Command::Status { site } => status::run(site),
        Command::Release { command } => match command {
            ReleaseCommand::List { site: site_name } => release::list::run(site_name),
            ReleaseCommand::Kill { site: site_name, release } => release::kill::run(site_name, release),
            ReleaseCommand::Rollback { site: site_name } => deploy::rollback(site_name),
            ReleaseCommand::DropFailed { site: site_name } => drop_failed_release::run(site_name),
            ReleaseCommand::Prune { site: site_name } => release::prune::run(site_name),
            ReleaseCommand::Recover { site: site_name } => release::recover::run(site_name),
        },
        Command::Service { command } => match command {
            ServiceCommand::Restart { site: site_name } => {
                let mutation = SiteMutation::acquire(site_name)?;
                service::run(&mutation)
            }
        },
        Command::Runtime { command } => match command {
            RuntimeCommand::Start { site } => docker::service::start(site),
            RuntimeCommand::Stop { site } => docker::service::stop(site),
        },
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
