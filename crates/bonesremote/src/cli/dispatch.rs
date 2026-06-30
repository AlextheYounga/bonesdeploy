use anyhow::Result;

use crate::cli::args::{Cli, Command, HookCommand, ReleaseCommand, ServiceCommand, SiteCommand};
use crate::commands::{deploy, doctor, drop_failed_release, hook, post_deploy, service, site, status, version};

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Doctor { site } => doctor::run(site.as_deref()),
        Command::Deploy { site, revision } => deploy::run_full(site, revision.as_deref()),
        Command::Status { site } => status::run(site),
        Command::Hook { command } => match command {
            HookCommand::PostReceive { site: site_name } => hook::post_receive(site_name),
        },
        Command::Site { command } => match command {
            SiteCommand::Import { site: site_name } => site::import(site_name),
        },
        Command::Release { command } => match command {
            ReleaseCommand::Rollback { site: site_name } => deploy::rollback(site_name),
            ReleaseCommand::DropFailed { site: site_name } => drop_failed_release::run(site_name),
            ReleaseCommand::Prune { site: site_name } => post_deploy::run(site_name),
        },
        Command::Service { command } => match command {
            ServiceCommand::Restart { site: site_name } => service::run(site_name),
        },
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
