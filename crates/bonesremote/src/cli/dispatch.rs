use anyhow::Result;

use crate::cli::args::{Cli, Command, HookCommand, ReleaseCommand, ServiceCommand, SiteCommand};
use crate::commands::{
    deploy, doctor, drop_failed_release, hook, release_kill, release_list, release_prune, service, site, status,
    version,
};

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
            ReleaseCommand::List { site: site_name } => release_list::run(site_name),
            ReleaseCommand::Kill { site: site_name, release } => release_kill::run(site_name, release),
            ReleaseCommand::Rollback { site: site_name } => deploy::rollback(site_name),
            ReleaseCommand::DropFailed { site: site_name } => drop_failed_release::run(site_name),
            ReleaseCommand::Prune { site: site_name } => release_prune::run(site_name),
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
