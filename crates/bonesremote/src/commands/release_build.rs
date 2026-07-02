use std::fs;
use std::os::unix::fs::chown;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::config::{self, Runtime, build_group_for, build_user_for, load_runtime, runtime_group_for};
use shared::paths;
use shared::paths::default_web_root;

use crate::privileges;
use crate::release::scripts as deploy_output;
use crate::release_state;

pub fn run(site: &str, context: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release build")?;

    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    if !context.is_dir() {
        bail!("Build context does not exist: {}", context.display());
    }

    let build_user = build_user_for(&cfg.project_name);
    let build_group = build_group_for(&cfg.project_name);
    chown_tree_to_user(context, &build_user, &build_group)?;

    let scripts_dir = paths::bonesremote_site_root(site).join(paths::DEPLOYMENT_DIR).join(paths::DEPLOYMENT_BUILD_DIR);
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

        let runtime = load_runtime(&paths::bonesremote_site_root(site)).unwrap_or_else(|_| Runtime {
            web_root: default_web_root(),
            build_image: String::new(),
            runtime_user: String::new(),
            runtime_group: String::new(),
            release_group: String::new(),
            shared: config::Shared::default(),
        });
        if runtime.build_image.is_empty() {
            bail!("Build scripts require build_image in runtime.toml");
        }

        let status = deploy_output::run_podman_build_script(
            &script,
            context,
            &context.join(format!("{script_name}.log")),
            &deploy_output::BuildScriptEnv {
                project_name: &cfg.project_name,
                build_user: &build_user,
                build_uid: user_uid(&build_user)?,
                web_root: &runtime.web_root,
                build_image: &runtime.build_image,
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

    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg.project_root, &release_name);
    let release_group = runtime_group_for(&cfg.project_name);
    harden_release_tree(context, &release_dir, &cfg.project_name, &release_group)
        .with_context(|| format!("Failed to promote release {release_name}"))?;

    println!("Promoted release {release_name} into {}", release_dir.display());
    Ok(release_dir)
}

fn harden_release_tree(source: &Path, destination: &Path, _project_name: &str, release_group: &str) -> Result<()> {
    if !source.is_dir() {
        bail!("Source tree is not a directory: {}", source.display());
    }

    fs::create_dir_all(destination)
        .with_context(|| format!("Failed to create release directory {}", destination.display()))?;
    clear_directory_children(destination)?;

    copy_hardened(source, destination, source)?;
    seal_release(destination, release_group)?;
    Ok(())
}

fn copy_hardened(source: &Path, destination: &Path, tree_root: &Path) -> Result<()> {
    for entry in fs::read_dir(source).with_context(|| format!("Failed to read source tree {}", source.display()))? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)
            .with_context(|| format!("Failed to inspect build artifact {}", source_path.display()))?;
        let file_type = metadata.file_type();

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path)
                .with_context(|| format!("Failed to create release directory {}", dest_path.display()))?;
            copy_hardened(&source_path, &dest_path, tree_root)?;
            continue;
        }

        if file_type.is_file() {
            fs::copy(&source_path, &dest_path).with_context(|| {
                format!("Failed to copy build artifact {} into {}", source_path.display(), dest_path.display())
            })?;
            continue;
        }

        if file_type.is_symlink() {
            let target = fs::read_link(&source_path)
                .with_context(|| format!("Failed to read symlink {}", source_path.display()))?;
            validate_symlink_target(&source_path, &target, tree_root)?;
            symlink(&target, &dest_path)
                .with_context(|| format!("Failed to recreate symlink {}", dest_path.display()))?;
            continue;
        }

        bail!("Unsupported artifact type in promoted release: {}", source_path.display());
    }

    Ok(())
}

fn validate_symlink_target(link_path: &Path, target: &Path, tree_root: &Path) -> Result<()> {
    if target.is_absolute() {
        bail!("Absolute symlink is not allowed in release artifacts: {} -> {}", link_path.display(), target.display());
    }

    let link_parent = link_path.parent().unwrap_or(tree_root);
    let candidate = normalize_relative_path(&link_parent.join(target), tree_root)?;
    if !candidate.starts_with(tree_root) {
        bail!("Symlink escapes release tree: {} -> {}", link_path.display(), target.display());
    }

    Ok(())
}

fn normalize_relative_path(path: &Path, root: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized == root || !normalized.pop() {
                    bail!("Path escapes release tree: {}", path.display());
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Ok(normalized)
}

fn seal_release(destination: &Path, release_group: &str) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    let gid = site_group_gid(release_group)?;
    let uid = root_uid()?;

    let metadata = fs::symlink_metadata(destination)
        .with_context(|| format!("Failed to inspect {} for sealing", destination.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    chown(destination, Some(uid), Some(gid))
        .with_context(|| format!("Failed to chown {} to root:{}", destination.display(), release_group))?;

    let mode = if metadata.file_type().is_dir() {
        0o750
    } else if metadata.mode() & 0o111 != 0 {
        0o750
    } else {
        0o640
    };
    fs::set_permissions(destination, fs::Permissions::from_mode(mode))
        .with_context(|| format!("Failed to set permissions on {}", destination.display()))?;

    if metadata.file_type().is_dir() {
        for entry in fs::read_dir(destination)
            .with_context(|| format!("Failed to read {} for sealing", destination.display()))?
        {
            let entry = entry?;
            seal_release(&entry.path(), release_group)?;
        }
    }

    Ok(())
}

fn root_uid() -> Result<u32> {
    user_uid("root")
}

fn user_uid(user: &str) -> Result<u32> {
    let passwd = fs::read_to_string(paths::ETC_PASSWD)
        .with_context(|| format!("Failed to read {} while resolving uid for {user}", paths::ETC_PASSWD))?;
    parse_user_uid(&passwd, user)
}

fn site_group_gid(group: &str) -> Result<u32> {
    let groupfile = fs::read_to_string(paths::ETC_GROUP)
        .with_context(|| format!("Failed to read {} while sealing release", paths::ETC_GROUP))?;
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

fn parse_user_uid(passwd: &str, user: &str) -> Result<u32> {
    let line = passwd
        .lines()
        .find(|line| line.starts_with(&format!("{user}:")))
        .with_context(|| format!("User '{user}' missing from {}", paths::ETC_PASSWD))?;
    let fields: Vec<&str> = line.split(':').collect();
    fields
        .get(2)
        .with_context(|| format!("User '{user}' missing uid field"))?
        .parse::<u32>()
        .with_context(|| format!("User '{user}' uid is not a valid integer"))
}

fn chown_tree_to_user(path: &Path, user: &str, group: &str) -> Result<()> {
    let uid = user_uid(user)?;
    let gid = site_group_gid(group)?;
    chown_tree(path, uid, gid)
}

fn chown_tree(path: &Path, uid: u32, gid: u32) -> Result<()> {
    chown(path, Some(uid), Some(gid)).with_context(|| format!("Failed to chown {}", path.display()))?;

    if fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect {} for chown", path.display()))?
        .file_type()
        .is_dir()
    {
        for entry in fs::read_dir(path).with_context(|| format!("Failed to read {} for chown", path.display()))? {
            let entry = entry?;
            chown_tree(&entry.path(), uid, gid)?;
        }
    }

    Ok(())
}

fn clear_directory_children(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("Failed to read release directory {}", path.display()))? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        if entry_type.is_dir() {
            fs::remove_dir_all(entry.path())?;
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

#[cfg(test)]
mod tests {
    use super::{clear_directory_children, normalize_relative_path, parse_user_uid, validate_symlink_target};
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process;

    use anyhow::Result;

    #[test]
    fn clear_directory_children_only_removes_entries() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-promote-clear-{}", process::id()));
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

    #[test]
    fn normalize_relative_path_rejects_escape() {
        let root = Path::new("/tmp/release-root");
        let escaped = normalize_relative_path(Path::new("/tmp/release-root/app/../../etc/passwd"), root);
        assert!(escaped.is_err());
    }

    #[test]
    fn validate_symlink_target_rejects_absolute_and_escaping_targets() {
        let root = Path::new("/tmp/release-root");
        assert!(validate_symlink_target(Path::new("/tmp/release-root/x"), Path::new("/etc/passwd"), root).is_err());
        assert!(
            validate_symlink_target(Path::new("/tmp/release-root/public/x"), Path::new("../../evil"), root).is_err()
        );
    }

    #[test]
    fn parse_user_uid_reads_uid_field() {
        let passwd = "root:x:0:0:root:/root:/bin/bash\ndemo-build:x:1234:1234::/nonexistent:/usr/sbin/nologin\n";
        assert_eq!(parse_user_uid(passwd, "demo-build").expect("uid should parse"), 1234);
    }
}
