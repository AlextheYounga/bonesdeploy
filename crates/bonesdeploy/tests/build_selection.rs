use std::env;
use std::fs;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesdeploy::build;
use bonesdeploy_core::config::Bones;

#[test]
fn native_build_script_requirements_follow_the_selected_framework() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = Command::new(env::current_exe()?)
        .args(["--exact", "build_selection_in_repository", "--nocapture"])
        .current_dir(root.path())
        .env("BONES_BUILD_SELECTION_TEST", "1")
        .env("XDG_CACHE_HOME", root.path().join("cache"))
        .output()?;
    ensure!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn build_selection_in_repository() -> Result<()> {
    if env::var_os("BONES_BUILD_SELECTION_TEST").is_none() {
        return Ok(());
    }
    for args in
        [&["init", "-b", "main"][..], &["config", "user.email", "test@example.com"], &["config", "user.name", "Test"]]
    {
        ensure!(Command::new("git").args(args).status()?.success());
    }
    fs::write("tracked", "committed source")?;
    ensure!(Command::new("git").args(["add", "tracked"]).status()?.success());
    ensure!(Command::new("git").args(["commit", "-m", "fixture"]).status()?.success());
    let mut config = Bones::for_site("selection-test");
    config.branch = "main".into();
    config.runtime.template = "laravel".into();
    let Err(error) = build::package(&config) else {
        anyhow::bail!("Laravel without committed build scripts must fail")
    };
    assert!(error.to_string().contains("Committed deploy branch 'main' has no native build scripts"));
    config.runtime.template = "custom".into();
    let artifact = build::package(&config)?;
    assert!(artifact.path().is_file());
    Ok(())
}
