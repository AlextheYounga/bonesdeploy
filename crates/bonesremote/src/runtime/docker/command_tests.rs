use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, bail};
use tempfile::TempDir;

use super::{active_start_with, active_stop_with, prepare_candidate_with};

#[test]
fn candidate_runs_quiet_validation_pull_and_build_in_order() -> Result<()> {
    let fixture = Fixture::new()?;
    let mut calls = Vec::new();

    prepare_candidate_with("atlas", fixture.root(), fixture.candidate(), &mut |command, action| {
        calls.push(call(command, action));
        Ok(())
    })?;

    assert_eq!(actions(&calls), ["validate Compose configuration", "pull Compose images", "build Compose images"]);
    assert_eq!(operation(&calls[0])?, ["config", "--quiet"]);
    assert_eq!(operation(&calls[1])?, ["pull"]);
    assert_eq!(operation(&calls[2])?, ["build"]);
    assert!(calls.iter().all(|call| !call.arguments.iter().any(|argument| argument.contains("docker.sock"))));
    Ok(())
}

#[test]
fn candidate_stops_at_the_first_failed_operation() -> Result<()> {
    let fixture = Fixture::new()?;
    let mut calls = Vec::new();

    let result = prepare_candidate_with("atlas", fixture.root(), fixture.candidate(), &mut |command, action| {
        calls.push(call(command, action));
        if action == "pull Compose images" {
            bail!("simulated pull failure");
        }
        Ok(())
    });
    let Err(error) = result else {
        bail!("pull failure must abort candidate preparation");
    };

    assert_eq!(actions(&calls), ["validate Compose configuration", "pull Compose images"]);
    assert!(error.to_string().contains("simulated pull failure"));
    Ok(())
}

#[test]
fn active_start_reconciles_current_with_wait_timeout() -> Result<()> {
    let fixture = Fixture::new()?;
    let mut calls = Vec::new();

    active_start_with("atlas", fixture.root(), 45, &mut |command, action| {
        calls.push(call(command, action));
        Ok(())
    })?;

    assert_eq!(actions(&calls), ["start Compose stack"]);
    assert_eq!(
        operation(&calls[0])?,
        ["up", "--detach", "--build", "--remove-orphans", "--wait", "--wait-timeout", "45"]
    );
    assert_eq!(calls[0].directory, fixture.root().join("current"));
    Ok(())
}

#[test]
fn active_stop_never_removes_containers_or_volumes() -> Result<()> {
    let fixture = Fixture::new()?;
    let mut calls = Vec::new();

    active_stop_with("atlas", fixture.root(), &mut |command, action| {
        calls.push(call(command, action));
        Ok(())
    })?;

    assert_eq!(actions(&calls), ["stop Compose stack"]);
    assert_eq!(operation(&calls[0])?, ["stop"]);
    assert!(!calls[0].arguments.iter().any(|argument| matches!(argument.as_str(), "down" | "rm" | "--volumes" | "-v")));
    Ok(())
}

struct Fixture {
    directory: TempDir,
    candidate: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self> {
        let directory = TempDir::new()?;
        let candidate = directory.path().join("candidate");
        fs::create_dir_all(directory.path().join("shared"))?;
        fs::create_dir(&candidate)?;
        fs::write(candidate.join("compose.yaml"), "services: {}\n")?;
        symlink(&candidate, directory.path().join("current"))?;
        Ok(Self { directory, candidate })
    }

    fn root(&self) -> &Path {
        self.directory.path()
    }

    fn candidate(&self) -> &Path {
        &self.candidate
    }
}

struct RecordedCall {
    action: String,
    arguments: Vec<String>,
    directory: PathBuf,
}

fn call(command: &Command, action: &str) -> RecordedCall {
    RecordedCall {
        action: action.to_owned(),
        arguments: command.get_args().map(|argument| argument.to_string_lossy().into_owned()).collect(),
        directory: command.get_current_dir().unwrap_or_else(|| Path::new("")).to_owned(),
    }
}

fn actions(calls: &[RecordedCall]) -> Vec<&str> {
    calls.iter().map(|call| call.action.as_str()).collect()
}

fn operation(call: &RecordedCall) -> Result<Vec<&str>> {
    let Some(file) = call.arguments.iter().rposition(|argument| argument == "--file") else {
        bail!("Compose command must identify an explicit file");
    };
    Ok(call.arguments[file + 2..].iter().map(String::as_str).collect())
}
