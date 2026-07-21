use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use std::io::Write as _;

use crate::config;
use crate::infra::ssh;
use crate::ui::output;
use shared::{config::is_numbered_shell_script, paths};

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
    archive_bones_directory_at(Path::new(paths::LOCAL_BONES_DIR))
}

fn archive_bones_directory_at(bones_dir: &Path) -> Result<Vec<u8>> {
    let files = publishable_files(bones_dir)?;
    let output = Command::new("tar")
        .args(["-czf", "-", "--no-recursion", "-C"])
        .arg(bones_dir)
        .arg("--")
        .args(files)
        .output()
        .context("Failed to run tar for .bones")?;

    if output.status.success() {
        return Ok(output.stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Failed to archive .bones\n{stderr}")
}

fn publishable_files(bones_dir: &Path) -> Result<Vec<PathBuf>> {
    require_regular_file(bones_dir, paths::BONES_TOML)?;
    let mut files = vec![PathBuf::from(paths::BONES_TOML)];
    let deployment = bones_dir.join(paths::DEPLOYMENT_DIR);

    add_regular_file(&mut files, &deployment, paths::DEPLOYMENT_FUNCTIONS_FILE);
    for directory in [paths::DEPLOYMENT_BUILD_DIR, paths::DEPLOYMENT_PREPARE_DIR] {
        add_numbered_scripts(&mut files, &deployment.join(directory), directory)?;
    }

    Ok(files)
}

fn require_regular_file(directory: &Path, name: &str) -> Result<()> {
    let path = directory.join(name);
    if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.file_type().is_file()) {
        return Ok(());
    }
    bail!("Required publishable file is missing or not a regular file: {}", path.display())
}

fn add_regular_file(files: &mut Vec<PathBuf>, directory: &Path, name: &str) {
    let path = directory.join(name);
    if path.is_file() && !path.is_symlink() {
        files.push(PathBuf::from(paths::DEPLOYMENT_DIR).join(name));
    }
}

fn add_numbered_scripts(files: &mut Vec<PathBuf>, directory: &Path, name: &str) -> Result<()> {
    if !directory.is_dir() || directory.is_symlink() {
        return Ok(());
    }

    for entry in fs::read_dir(directory).with_context(|| format!("Failed to read {}", directory.display()))? {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else { continue };
        if entry.file_type()?.is_file() && is_numbered_shell_script(file_name) {
            files.push(PathBuf::from(paths::DEPLOYMENT_DIR).join(name).join(file_name));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use anyhow::Result;
    use shared::paths;
    use tempfile::TempDir;

    use super::archive_bones_directory_at;

    #[test]
    fn local_secrets_path_stays_under_bones_dir() {
        let path = Path::new(paths::LOCAL_BONES_SECRETS_DIR);
        assert_eq!(path.parent(), Some(Path::new(paths::LOCAL_BONES_DIR)));
    }

    #[test]
    fn archive_contains_only_allowlisted_regular_files() -> Result<()> {
        let temp = TempDir::new()?;
        let bones = temp.path().join("bones");
        fs::create_dir_all(bones.join("deployment/build/nested"))?;
        fs::create_dir_all(bones.join("deployment/prepare"))?;
        fs::create_dir_all(bones.join("secrets"))?;
        fs::write(bones.join("bones.toml"), "[app]\n")?;
        fs::write(bones.join("custom.py"), "raise RuntimeError\n")?;
        fs::write(bones.join("secrets/.env"), "SECRET=value\n")?;
        fs::write(bones.join("deployment/functions.sh"), "#!/bin/bash\n")?;
        fs::write(bones.join("deployment/build/01_build.sh"), "#!/bin/bash\n")?;
        fs::write(bones.join("deployment/build/README.md"), "not a script\n")?;
        fs::write(bones.join("deployment/build/nested/02_nested.sh"), "#!/bin/bash\n")?;
        fs::write(bones.join("deployment/prepare/02_prepare.sh"), "#!/bin/bash\n")?;
        fs::write(bones.join("deployment/prepare/03_prepare.py"), "print()\n")?;
        std::os::unix::fs::symlink("01_build.sh", bones.join("deployment/build/04_link.sh"))?;

        let archive = temp.path().join("state.tar.gz");
        fs::write(&archive, archive_bones_directory_at(&bones)?)?;
        let output = Command::new("tar").args(["-tzf"]).arg(&archive).output()?;

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout)?.lines().collect::<Vec<_>>(),
            [
                "bones.toml",
                "deployment/functions.sh",
                "deployment/build/01_build.sh",
                "deployment/prepare/02_prepare.sh"
            ]
        );
        Ok(())
    }

    #[test]
    fn archive_rejects_a_symlinked_config() -> Result<()> {
        let temp = TempDir::new()?;
        let bones = temp.path().join("bones");
        fs::create_dir_all(&bones)?;
        fs::write(bones.join("actual.toml"), "[app]\n")?;
        std::os::unix::fs::symlink("actual.toml", bones.join("bones.toml"))?;

        assert!(archive_bones_directory_at(&bones).is_err());
        Ok(())
    }
}
