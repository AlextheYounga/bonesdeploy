use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use std::io::Write as _;

use crate::config;
use crate::infra::ssh;
use crate::ui::output;
use shared::paths;

pub fn run(show_next: bool) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    println!("Publishing .bones...");
    sync_bones_directory(&cfg).context("Failed to publish .bones.")?;

    println!(".bones published.");
    if show_next {
        println!();
        println!("{}", output::next_step("bonesdeploy doctor"));
    }

    Ok(())
}

pub(crate) fn sync_bones_directory(cfg: &config::Bones) -> Result<()> {
    let archive = archive_bones_directory()?;
    let mut child = ssh::external_command(&cfg.ssh_user, &cfg.host, &cfg.port)
        .arg(remote_import_command(&cfg.project_name))
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start remote site import")?;

    let mut stdin = child.stdin.take().context("ssh stdin was not piped")?;
    stdin.write_all(&archive).context("Failed to stream .bones archive to remote host")?;
    drop(stdin);

    let output = child.wait_with_output().context("Failed to finish remote site import")?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Failed to import remote site state\n{stderr}")
}

pub(crate) fn remote_import_command(site: &str) -> String {
    format!("bonesremote site import --site '{site}'")
}

fn archive_bones_directory() -> Result<Vec<u8>> {
    let output = Command::new("tar")
        .args(["-czf", "-", "--exclude", "./secrets", "-C", paths::LOCAL_BONES_DIR, "."])
        .output()
        .context("Failed to run tar for .bones")?;

    if output.status.success() {
        return Ok(output.stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Failed to archive .bones\n{stderr}")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::remote_import_command;
    use shared::paths;

    #[test]
    fn local_secrets_path_stays_under_bones_dir() {
        let path = Path::new(paths::LOCAL_BONES_SECRETS_DIR);
        assert_eq!(path.parent(), Some(Path::new(paths::LOCAL_BONES_DIR)));
    }

    #[test]
    fn remote_import_command_targets_control_plane_import() {
        assert_eq!(remote_import_command("acme"), "bonesremote site import --site 'acme'");
    }
}
