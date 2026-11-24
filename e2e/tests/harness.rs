use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use e2e::{project::SampleProject, session::Session};

#[test]
fn session_commands_use_a_short_isolated_state_home() -> Result<()> {
    let session = Session::create()?;
    let command = session.command("true");
    let state_home = command
        .get_envs()
        .find_map(|(name, value)| (name == "XDG_STATE_HOME").then_some(value).flatten())
        .context("XDG_STATE_HOME was not set")?;
    let state_home = Path::new(state_home);

    assert_eq!(state_home.parent(), Some(Path::new("/tmp")));
    assert!(state_home.file_name().is_some_and(|name| name.to_string_lossy().starts_with("bones-e2e-state-")));
    assert!(state_home.as_os_str().len() < 64);
    Ok(())
}

fn project_with_build_env(content: &str) -> Result<SampleProject> {
    let session = Session::create()?;
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/laravel.md");
    let project = SampleProject::from_fixture(&session, &fixture)?;
    fs::write(project.dir().join(".env.build"), content)?;
    Ok(project)
}

#[test]
fn pinning_node_version_preserves_other_build_environment_content() -> Result<()> {
    let project = project_with_build_env("BUILD_FLAG=true\nNODE_VERSION=\nOTHER=value\n")?;

    project.pin_node_version("24.19.0")?;

    assert_eq!(
        fs::read_to_string(project.dir().join(".env.build"))?,
        "BUILD_FLAG=true\nNODE_VERSION=24.19.0\nOTHER=value\n"
    );
    Ok(())
}

#[test]
fn pinning_node_version_replaces_an_existing_version() -> Result<()> {
    let project = project_with_build_env("# generated\nNODE_VERSION=25.9.0\n")?;

    project.pin_node_version("24.19.0")?;

    assert_eq!(fs::read_to_string(project.dir().join(".env.build"))?, "# generated\nNODE_VERSION=24.19.0\n");
    Ok(())
}

#[test]
fn pinning_node_version_without_an_assignment_leaves_the_file_unchanged() -> Result<()> {
    let original = "# generated\nOTHER=value\n";
    let project = project_with_build_env(original)?;

    assert!(project.pin_node_version("24.19.0").is_err());
    assert_eq!(fs::read_to_string(project.dir().join(".env.build"))?, original);
    Ok(())
}
