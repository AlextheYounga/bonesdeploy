use std::fs;
use std::path::Path;

use anyhow::Result;
use bonesremote::runtime::docker::command::{
    ComposeCommand, ComposeFiles, compose_command, parse_ps_output, project_name,
};
use tempfile::TempDir;

#[test]
fn project_name_is_stable_and_site_scoped() -> Result<()> {
    assert_eq!(project_name("atlas")?, "bonesdeploy-atlas");
    Ok(())
}

#[test]
fn discovery_uses_base_precedence_and_optional_override_in_order() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.file("compose.yml")?;
    fixture.file("compose.override.yml")?;

    let files = ComposeFiles::discover(fixture.root())?;

    assert_eq!(file_names(&files), ["compose.yml", "compose.override.yml"].map(String::from));
    Ok(())
}

#[test]
fn discovery_rejects_multiple_base_files() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.file("compose.yaml")?;
    fixture.file("docker-compose.yml")?;

    let Err(error) = ComposeFiles::discover(fixture.root()) else {
        anyhow::bail!("multiple base files must fail");
    };

    assert!(error.to_string().contains("Multiple Compose base files"));
    Ok(())
}

#[test]
fn discovery_rejects_multiple_override_files() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.file("compose.yaml")?;
    fixture.file("compose.override.yaml")?;
    fixture.file("compose.override.yml")?;

    let Err(error) = ComposeFiles::discover(fixture.root()) else {
        anyhow::bail!("multiple override files must fail");
    };

    assert!(error.to_string().contains("Multiple Compose override files"));
    Ok(())
}

#[test]
fn compose_command_explicitly_controls_release_files_and_environment() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.file("compose.yaml")?;
    fixture.file("compose.override.yaml")?;
    let files = ComposeFiles::discover(fixture.root())?;
    let env_file = fixture.root().join("shared/.env");

    let compose = ComposeCommand { project_directory: fixture.root(), env_file: &env_file, files: &files };
    let command = compose_command("atlas", &compose, ["config", "--quiet"])?;
    let arguments = command.get_args().map(|argument| argument.to_string_lossy()).collect::<Vec<_>>();

    assert_eq!(
        arguments,
        [
            "compose",
            "--project-name",
            "bonesdeploy-atlas",
            "--project-directory",
            fixture.root().to_string_lossy().as_ref(),
            "--env-file",
            env_file.to_string_lossy().as_ref(),
            "--file",
            fixture.root().join("compose.yaml").to_string_lossy().as_ref(),
            "--file",
            fixture.root().join("compose.override.yaml").to_string_lossy().as_ref(),
            "config",
            "--quiet",
        ]
    );
    assert!(matches!(command.get_envs().find(|(name, _)| *name == "COMPOSE_FILE"), Some((_, None))));
    assert!(matches!(command.get_envs().find(|(name, _)| *name == "COMPOSE_PROJECT_NAME"), Some((_, None))));
    assert!(matches!(command.get_envs().find(|(name, _)| *name == "DOCKER_HOST"), Some((_, None))));
    Ok(())
}

#[test]
fn compose_status_distinguishes_health_completion_failure_and_ports() -> Result<()> {
    let statuses = parse_ps_output(
        br#"[
            {"Name":"atlas-web-1","Service":"web","State":"running","Health":"healthy","ExitCode":0,
             "Publishers":[{"URL":"127.0.0.1","TargetPort":3000,"PublishedPort":8080,"Protocol":"tcp"}]},
            {"Name":"atlas-worker-1","Service":"worker","State":"running","Health":"","ExitCode":0,"Publishers":null},
            {"Name":"atlas-migrate-1","Service":"migrate","State":"exited","Health":"","ExitCode":0,"Publishers":[]},
            {"Name":"atlas-db-1","Service":"db","State":"running","Health":"unhealthy","ExitCode":0,"Publishers":[]},
            {"Name":"atlas-job-1","Service":"job","State":"exited","Health":"","ExitCode":7,
             "Publishers":[{"URL":"0.0.0.0","TargetPort":9000,"PublishedPort":9000,"Protocol":"tcp"}]}
        ]"#,
    )?;

    assert_eq!(
        statuses.iter().map(|status| status.condition()).collect::<Vec<_>>(),
        ["healthy", "running", "completed", "unhealthy", "failed",]
    );
    assert!(statuses[0].publishes_loopback_port(8080));
    assert!(statuses[4].publishes_public_port());
    Ok(())
}

#[test]
fn compose_status_accepts_newline_delimited_json() -> Result<()> {
    let statuses = parse_ps_output(
        b"{\"Name\":\"atlas-web-1\",\"Service\":\"web\",\"State\":\"running\"}\n\
          {\"Name\":\"atlas-job-1\",\"Service\":\"job\",\"State\":\"exited\",\"ExitCode\":0}\n",
    )?;

    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses[1].condition(), "completed");
    Ok(())
}

struct Fixture {
    directory: TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        Ok(Self { directory: TempDir::new()? })
    }

    fn root(&self) -> &Path {
        self.directory.path()
    }

    fn file(&self, name: &str) -> Result<()> {
        fs::write(self.root().join(name), "services: {}\n")?;
        Ok(())
    }
}

fn file_names(files: &ComposeFiles) -> Vec<String> {
    files
        .paths()
        .iter()
        .map(|path| path.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned()))
        .collect()
}
