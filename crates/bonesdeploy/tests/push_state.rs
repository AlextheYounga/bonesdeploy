mod common;

use std::fs;
use std::process::Command;

use anyhow::Result;
use common::TestEnv;

const AUTOCOMMIT_MESSAGE: &str = "automated commit";

#[test]
fn pushes_to_bones_config_repo() -> Result<()> {
    let env = TestEnv::new()?;
    let repo = env.repo();
    let bones_dir = repo.join(".bones");

    fs::create_dir_all(bones_dir.join("confs"))?;
    fs::write(bones_dir.join("bones.toml"), "[app]\nproject_name = \"test\"\nhost = \"example.com\"\n")?;

    let remote = repo.join("remote");
    fs::create_dir_all(&remote)?;
    Command::new("git").args(["--git-dir"]).arg(&remote).arg("init").arg("--bare").status()?;

    Command::new("git").args(["-C"]).arg(&bones_dir).args(["init", "--initial-branch", "master"]).output()?;

    let remote_url = remote.to_string_lossy().into_owned();
    Command::new("git").args(["-C"]).arg(&bones_dir).args(["remote", "add", "origin", &remote_url]).status()?;

    let output = env.run(&["push"])?;
    assert!(output.status.success(), "push failed: {}", String::from_utf8_lossy(&output.stderr));

    let log = Command::new("git").args(["--git-dir"]).arg(&remote).args(["log", "--oneline", "master"]).output()?;
    assert!(log.status.success(), "log failed: {}", String::from_utf8_lossy(&log.stderr));
    let log_str = String::from_utf8_lossy(&log.stdout);
    assert!(log_str.contains(AUTOCOMMIT_MESSAGE), "expected commit message in: {log_str}");
    Ok(())
}
