mod common;

use anyhow::Result;

#[test]
fn version_flag_prints_the_package_version() -> Result<()> {
    let env = common::TestEnv::new()?;
    let output = env.run(&["--version"])?;

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), concat!("bonesdeploy ", env!("CARGO_PKG_VERSION")));
    Ok(())
}

#[test]
fn setup_help_distinguishes_fresh_host_and_project_scopes() -> Result<()> {
    let env = common::TestEnv::new()?;

    let root = env.run(&["--help"])?;
    assert!(root.status.success());
    assert!(String::from_utf8_lossy(&root.stdout).contains("Set up a fresh host and this project"));

    let combined = env.run(&["setup", "--help"])?;
    assert!(combined.status.success());
    assert!(String::from_utf8_lossy(&combined.stdout).contains("additional project on a prepared host"));

    let server = env.run(&["server", "setup", "--help"])?;
    assert!(server.status.success());
    assert!(String::from_utf8_lossy(&server.stdout).contains("once per host"));

    let site = env.run(&["site", "setup", "--help"])?;
    assert!(site.status.success());
    assert!(String::from_utf8_lossy(&site.stdout).contains("once per project"));

    Ok(())
}

#[test]
fn doctor_accepts_verbose_flag() -> Result<()> {
    let env = common::TestEnv::new()?;
    let output = env.run(&["site", "doctor", "--local", "--verbose"])?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Checking deployment"), "doctor should run with --verbose: {stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Doctor found 1 issue."), "missing config should fail doctor: {stderr}");
    Ok(())
}

#[test]
fn site_manifest_accepts_json_format_and_reports_missing_config() -> Result<()> {
    let env = common::TestEnv::new()?;
    let output = env.run(&["site", "manifest", "--format", "json"])?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("root .env and infra/ are required"), "unexpected stderr: {stderr}");
    Ok(())
}

#[test]
fn init_rejects_removed_service_selection() -> Result<()> {
    let env = common::TestEnv::new()?;
    let output = env.run(&["init", "--service", "postgres"])?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument '--service'"));
    Ok(())
}

#[test]
fn init_rejects_invalid_config_before_writing_project_files() -> Result<()> {
    let env = common::TestEnv::new()?;
    let output = env.run(&[
        "init",
        "--non-interactive",
        "--project-name",
        "shop_admin",
        "--host",
        "deploy.example.com",
        "--template",
        "none",
    ])?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid project name"));
    assert!(!env.repo().join(".env").exists());
    assert!(!env.repo().join("infra").exists());
    Ok(())
}
