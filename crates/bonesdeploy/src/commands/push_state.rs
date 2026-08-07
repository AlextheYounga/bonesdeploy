use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::ui::output;
use bonesdeploy_core::paths;

const AUTOCOMMIT_MESSAGE: &str = "automated commit";

pub fn run(show_next: bool) -> Result<()> {
    println!("Publishing .bones...");
    sync_bones_directory().context("Failed to publish .bones.")?;

    println!(".bones published.");
    if show_next {
        println!();
        println!("{}", output::next_step("bonesdeploy doctor"));
    }

    Ok(())
}

pub(crate) fn sync_bones_directory() -> Result<()> {
    let bones_dir = Path::new(paths::local_bones_dir());
    if !bones_dir.exists() {
        bail!(".bones directory is missing; run 'bonesdeploy init' first")
    }
    stage_all_at(bones_dir)?;
    commit_if_needed_at(bones_dir)?;
    push_to_remote(bones_dir)?;
    Ok(())
}

fn stage_all_at(bones_dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(bones_dir)
        .args(["add", "-A"])
        .output()
        .context("Failed to stage .bones files")?;
    if !output.status.success() {
        bail!("Failed to stage .bones files: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn commit_if_needed_at(bones_dir: &Path) -> Result<()> {
    let diff = Command::new("git")
        .args(["-C"])
        .arg(bones_dir)
        .args(["diff", "--cached", "--quiet"])
        .status()
        .context("Failed to check .bones staged changes")?;
    if diff.success() {
        return Ok(());
    }
    let output = Command::new("git")
        .arg("-c")
        .arg("user.name=BonesDeploy")
        .arg("-c")
        .arg("user.email=bonesdeploy@local")
        .args(["-C"])
        .arg(bones_dir)
        .args(["commit", "-m", AUTOCOMMIT_MESSAGE])
        .output()
        .context("Failed to commit .bones changes")?;
    if !output.status.success() {
        bail!("Failed to commit .bones: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn push_to_remote(bones_dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(bones_dir)
        .args(["push", "origin", "master"])
        .output()
        .context("Failed to push .bones to remote config repo")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to push .bones config\n{stderr}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    use anyhow::Result;
    use tempfile::TempDir;

    use super::{AUTOCOMMIT_MESSAGE, commit_if_needed_at, stage_all_at};

    #[test]
    fn pushes_to_bones_config_repo() -> Result<()> {
        let temp = TempDir::new()?;
        let bones_dir = temp.path().join(".bones");
        fs::create_dir_all(bones_dir.join("confs"))?;
        fs::write(bones_dir.join("bones.toml"), "[app]\nproject_name = \"test\"\nhost = \"example.com\"\n")?;

        let remote = temp.path().join("remote");
        fs::create_dir_all(&remote)?;
        Command::new("git").args(["--git-dir"]).arg(&remote).arg("init").arg("--bare").status()?;

        Command::new("git").args(["-C"]).arg(&bones_dir).args(["init"]).output()?;

        let remote_url = remote.to_string_lossy();
        Command::new("git").args(["-C"]).arg(&bones_dir).args(["remote", "add", "origin", &remote_url]).status()?;

        stage_all_at(&bones_dir)?;
        commit_if_needed_at(&bones_dir)?;

        let push_output =
            Command::new("git").args(["-C"]).arg(&bones_dir).args(["push", "origin", "master"]).output()?;
        if !push_output.status.success() {
            eprintln!("push stderr: {}", String::from_utf8_lossy(&push_output.stderr));
        }

        let log = Command::new("git").args(["--git-dir"]).arg(&remote).args(["log", "--oneline", "master"]).output()?;
        assert!(log.status.success(), "log failed: {}", String::from_utf8_lossy(&log.stderr));
        let log_str = String::from_utf8(log.stdout)?;
        assert!(log_str.contains(AUTOCOMMIT_MESSAGE), "expected commit message in: {log_str}");
        Ok(())
    }
}
