use std::fs;
use std::os::unix::fs::chown;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::paths;

use crate::config;
use crate::privileges;
use crate::release_state;

pub fn run(site: &str, context: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release build")?;

    let config_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&config_path)
        .with_context(|| format!("Failed to load remote site state from {}", config_path.display()))?;

    if !context.is_dir() {
        bail!("Build context does not exist: {}", context.display());
    }

    let scripts_dir = paths::bonesremote_site_root(site).join(paths::DEPLOYMENT_DIR);
    if !scripts_dir.is_dir() {
        println!(
            "No deployment scripts at {}; running build steps directly on the exported source tree.",
            scripts_dir.display()
        );
        return Ok(());
    }

    let scripts = list_scripts(&scripts_dir)?;
    if scripts.is_empty() {
        println!("No deployment scripts found at {}; skipping build.", scripts_dir.display());
        return Ok(());
    }

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running build script {script_name}...");

        let runtime = shared::config::load_runtime(&paths::bonesremote_site_root(site)).unwrap_or_else(|_| {
            shared::config::Runtime {
                web_root: shared::paths::default_web_root(),
                runtime_user: String::new(),
                runtime_group: String::new(),
                release_group: String::new(),
            }
        });
        let status = deploy_output::run_deployment_script(
            &script,
            context,
            &context.join(format!("{script_name}.log")),
            &deploy_output::ScriptEnv {
                project_name: &cfg.project_name,
                project_root: &cfg.project_root,
                repo_path: &cfg.repo_path,
                web_root: &runtime.web_root,
            },
        )
        .with_context(|| format!("Failed to execute build script {}", script.display()))?;

        if !status.success() {
            bail!("Build script {script_name} exited with status {status}");
        }
    }

    Ok(())
}

pub fn promote(site: &str, context: &Path) -> Result<PathBuf> {
    privileges::ensure_root("bonesremote release promote")?;

    let config_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&config_path)
        .with_context(|| format!("Failed to load remote site state from {}", config_path.display()))?;

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    harden_release_tree(context, &release_dir).with_context(|| format!("Failed to promote release {release_name}"))?;

    println!("Promoted release {release_name} into {}", release_dir.display());
    Ok(release_dir)
}

fn harden_release_tree(source: &Path, destination: &Path) -> Result<()> {
    if !source.is_dir() {
        bail!("Source tree is not a directory: {}", source.display());
    }

    fs::create_dir_all(destination)
        .with_context(|| format!("Failed to create release directory {}", destination.display()))?;
    clear_directory_children(destination)?;

    copy_hardened(source, destination)?;
    seal_release(destination)?;
    Ok(())
}

fn copy_hardened(source: &Path, destination: &Path) -> Result<()> {
    let status = std::process::Command::new("cp")
        .arg("-a")
        .arg(source.join("."))
        .arg(destination)
        .status()
        .with_context(|| format!("Failed to copy source {} into {}", source.display(), destination.display()))?;

    if !status.success() {
        bail!("Failed to promote source {} into {}: status {status}", source.display(), destination.display());
    }

    Ok(())
}

fn seal_release(destination: &Path) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    let gid = site_group_gid()?;
    let uid = root_uid()?;

    let metadata = fs::symlink_metadata(destination)
        .with_context(|| format!("Failed to inspect {} for sealing", destination.display()))?;
    chown(destination, Some(uid), Some(gid))
        .with_context(|| format!("Failed to chown {} to root:<site>", destination.display()))?;

    if metadata.file_type().is_dir() {
        for entry in fs::read_dir(destination)
            .with_context(|| format!("Failed to read {} for sealing", destination.display()))?
        {
            let entry = entry?;
            let entry_type = entry.file_type()?;
            if entry_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            let sub_metadata = fs::symlink_metadata(&path)
                .with_context(|| format!("Failed to inspect {} for sealing", path.display()))?;
            chown(&path, Some(uid), Some(gid))
                .with_context(|| format!("Failed to chown {} to root:<site>", path.display()))?;
            if sub_metadata.file_type().is_dir() {
                let _ = sub_metadata;
                seal_release(&path)?;
            }
        }
    }

    Ok(())
}

fn root_uid() -> Result<u32> {
    let passwd = std::fs::read_to_string("/etc/passwd").context("Failed to read /etc/passwd while sealing release")?;
    let line = passwd.lines().find(|line| line.starts_with("root:")).context("root entry missing from /etc/passwd")?;
    let fields: Vec<&str> = line.split(':').collect();
    let uid = fields
        .get(2)
        .context("root passwd line missing uid field")?
        .parse::<u32>()
        .context("root uid is not a valid integer")?;
    Ok(uid)
}

fn site_group_gid() -> Result<u32> {
    let cfg = load_active_cfg_for_seal()?;
    let group = shared::config::runtime_group_for(&cfg.project_name);
    let groupfile = std::fs::read_to_string("/etc/group").context("Failed to read /etc/group while sealing release")?;
    let line = groupfile
        .lines()
        .find(|line| line.starts_with(&format!("{group}:")))
        .with_context(|| format!("Site group '{group}' is missing from /etc/group"))?;
    let fields: Vec<&str> = line.split(':').collect();
    let gid = fields
        .get(2)
        .with_context(|| format!("Group '{group}' missing gid field"))?
        .parse::<u32>()
        .with_context(|| format!("Group '{group}' gid is not a valid integer"))?;
    Ok(gid)
}

fn load_active_cfg_for_seal() -> Result<config::Bones> {
    let sites = paths::bonesremote_sites_root();
    if !sites.exists() {
        bail!("bonesremote site root missing while sealing release");
    }

    for entry in fs::read_dir(&sites)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let toml = entry.path().join(paths::BONES_TOML);
        if toml.is_file() {
            return config::load(&toml);
        }
    }

    bail!("No site bones.toml found under {}", sites.display());
}

fn clear_directory_children(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("Failed to read release directory {}", path.display()))? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        if entry_type.is_dir() {
            fs::remove_dir_all(entry.path())?;
        } else if entry_type.is_symlink() {
            fs::remove_file(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in
        fs::read_dir(scripts_dir).with_context(|| format!("Failed to read scripts dir: {}", scripts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}

#[path = "../release/scripts.rs"]
mod deploy_output;

#[cfg(test)]
mod tests {
    use super::clear_directory_children;
    use std::fs;

    use anyhow::Result;

    #[test]
    fn clear_directory_children_only_removes_entries() -> Result<()> {
        let root = std::env::temp_dir().join(format!("bonesremote-promote-clear-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join("file.txt"), "x")?;
        fs::create_dir_all(root.join("nested"))?;

        clear_directory_children(&root)?;

        assert!(root.exists(), "clear must not remove the directory itself");
        assert!(fs::read_dir(&root)?.next().is_none());

        fs::remove_dir_all(&root).ok();
        Ok(())
    }
}
