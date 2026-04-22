use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::{fs, io};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::config::BonesConfig;
use crate::release_state;

pub fn chown_paths_to_deploy_user(cfg: &BonesConfig, paths: &[&Path]) -> Result<()> {
    let deploy_user = &cfg.permissions.defaults.deploy;
    let service_group = &cfg.permissions.defaults.group;
    let ownership = format!("{deploy_user}:{service_group}");

    for path in paths {
        if !path.exists() {
            continue;
        }

        let path_str = path.to_string_lossy();
        run_chown(&ownership, &path_str, true)?;
        println!("Changed ownership of {} to {ownership}", path.display());
    }

    Ok(())
}

pub fn harden_paths(cfg: &BonesConfig, paths: &[&Path]) -> Result<()> {
    let defaults = &cfg.permissions.defaults;
    let ownership = format!("{}:{}", defaults.owner, defaults.group);
    let dir_mode = parse_mode(&defaults.dir_mode)?;
    let file_mode = parse_mode(&defaults.file_mode)?;

    for path in paths {
        if !path.exists() {
            continue;
        }

        let path_str = path.to_string_lossy();
        run_chown(&ownership, &path_str, true)?;
        apply_default_modes(path, dir_mode, file_mode)?;
        println!(
            "Hardened {} (owner {}:{}, dirs {}, files {})",
            path.display(),
            defaults.owner,
            defaults.group,
            defaults.dir_mode,
            defaults.file_mode
        );
    }

    Ok(())
}

pub fn harden_active_release(cfg: &BonesConfig) -> Result<()> {
    let current_link = release_state::current_link(cfg);
    let active_target =
        fs::read_link(&current_link).with_context(|| format!("Failed to read {}", current_link.display()))?;

    let active_release = if active_target.is_absolute() {
        active_target
    } else {
        current_link.parent().unwrap_or_else(|| Path::new("/")).join(active_target)
    };

    let shared = release_state::shared_dir(cfg);
    harden_paths(cfg, &[active_release.as_path(), shared.as_path()])?;
    apply_path_overrides(cfg, &active_release, &shared)
}

fn apply_path_overrides(cfg: &BonesConfig, active_release: &Path, shared_root: &Path) -> Result<()> {
    for override_entry in &cfg.permissions.paths {
        let Some(target) = select_override_target(active_release, shared_root, &override_entry.path)? else {
            let logical_path = active_release.join(&override_entry.path);
            println!("Warning: override path '{}' does not exist, skipping", logical_path.display());
            continue;
        };

        let mode = parse_mode(&override_entry.mode)?;

        if override_entry.recursive {
            apply_recursive_mode(&target, mode)?;
        } else if let Some(ref path_type) = override_entry.path_type {
            let metadata = fs::metadata(&target)
                .with_context(|| format!("Failed to read metadata for override target {}", target.display()))?;

            match path_type.as_str() {
                "dir" if metadata.is_dir() => apply_single_mode(&target, mode)?,
                "file" if metadata.is_file() => apply_single_mode(&target, mode)?,
                "dir" => {
                    bail!("Override '{}' expected a directory, got {}", override_entry.path, target.display())
                }
                "file" => {
                    bail!("Override '{}' expected a file, got {}", override_entry.path, target.display())
                }
                other => bail!("Unknown path type: {other}"),
            }
        } else {
            apply_single_mode(&target, mode)?;
        }

        println!(
            "Applied mode {} to {} (target: {}){}",
            override_entry.mode,
            override_entry.path,
            target.display(),
            if override_entry.recursive { " (recursive)" } else { "" }
        );
    }

    Ok(())
}

fn select_override_target(active_release: &Path, shared_root: &Path, override_path: &str) -> Result<Option<PathBuf>> {
    let logical = active_release.join(override_path);

    if fs::symlink_metadata(&logical).is_err() {
        return Ok(None);
    }

    if has_symlink_in_override_path(active_release, override_path)? {
        let shared_target = shared_root.join(override_path);
        if fs::symlink_metadata(&shared_target).is_err() {
            bail!(
                "Override '{}' is symlinked in active release but missing in shared root at {}",
                override_path,
                shared_target.display()
            );
        }
        return Ok(Some(shared_target));
    }

    Ok(Some(logical))
}

fn has_symlink_in_override_path(active_release: &Path, override_path: &str) -> Result<bool> {
    let mut current = PathBuf::from(active_release);

    for component in Path::new(override_path).components() {
        match component {
            Component::Normal(segment) => current.push(segment),
            Component::CurDir => continue,
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("Override path must be relative and not traverse parents: {override_path}");
            }
        }

        let metadata = fs::symlink_metadata(&current)
            .with_context(|| format!("Failed to inspect override path component {}", current.display()))?;
        if metadata.file_type().is_symlink() {
            return Ok(true);
        }
    }

    Ok(false)
}

fn run_chown(ownership: &str, path: &str, recursive: bool) -> Result<()> {
    let mut cmd = Command::new("chown");
    if recursive {
        cmd.arg("-R");
    }
    cmd.arg(ownership).arg(path);

    let status = cmd.status().with_context(|| format!("Failed to chown {path}"))?;

    if !status.success() {
        bail!("chown {ownership} {path} failed");
    }
    Ok(())
}

fn parse_mode(mode_str: &str) -> Result<u32> {
    u32::from_str_radix(mode_str, 8).with_context(|| format!("Invalid mode: {mode_str}"))
}

fn apply_default_modes(root: &Path, dir_mode: u32, file_mode: u32) -> Result<()> {
    for entry in WalkDir::new(root) {
        let entry = entry.with_context(|| format!("Failed to walk {}", root.display()))?;
        // Follow symlinks so a symlink to a directory gets dir_mode, not file_mode
        let metadata = fs::metadata(entry.path())
            .with_context(|| format!("Failed to read metadata for {}", entry.path().display()))?;

        let mode = if metadata.is_dir() { dir_mode } else { file_mode };
        set_permissions(entry.path(), mode)?;
    }
    Ok(())
}

fn apply_recursive_mode(path: &Path, mode: u32) -> Result<()> {
    for entry in WalkDir::new(path) {
        let entry = entry.with_context(|| format!("Failed to walk {}", path.display()))?;
        set_permissions(entry.path(), mode)?;
    }
    Ok(())
}

fn apply_single_mode(path: &Path, mode: u32) -> Result<()> {
    set_permissions(path, mode)
}

fn set_permissions(path: &Path, mode: u32) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|e| io::Error::new(e.kind(), format!("chmod {:o} {}: {e}", mode, path.display())))?;
    Ok(())
}
