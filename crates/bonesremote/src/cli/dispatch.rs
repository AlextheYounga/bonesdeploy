use anyhow::Result;

use crate::cli::args::{Cli, Command, ReleaseCommand, ServiceCommand};
use crate::commands::{
    activate_release, deploy, doctor, drop_failed_release, init, rollback, service, stage_release,
    version, wire_release,
};

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init => init::run(),
        Command::Doctor => doctor::run(),
        Command::Deploy { config, revision } => deploy::run_full(config, revision.as_deref()),
        Command::Release { command } => match command {
            ReleaseCommand::Stage { config } => stage_release::run(config),
            ReleaseCommand::Wire { config } => wire_release::run(config),
            ReleaseCommand::Activate { config } => activate_release::run(config),
            ReleaseCommand::DropFailed { config } => drop_failed_release::run(config),
            ReleaseCommand::Rollback { config } => rollback::run(config),
        },
        Command::Service { command } => match command {
            ServiceCommand::Restart { config } => service::run(config),
        },
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}
