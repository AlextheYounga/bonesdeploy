use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::{fs, io};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::config::BonesConfig;

pub fn chown_to_deploy_user(cfg: &BonesConfig) -> Result<()> {
    let user = &cfg.permissions.defaults.deploy;
    let worktree = &cfg.data.worktree;

    if !Path::new(worktree).exists() {
        fs::create_dir_all(worktree)
            .with_context(|| format!("Failed to create worktree directory: {worktree}"))?;
        println!("Created worktree directory: {worktree}");
    }

    run_chown(&format!("{user}:{user}"), worktree, true)?;
    println!("Changed ownership of {worktree} to {user}");
    Ok(())
}

pub fn chown_paths(user: &str, paths: &[&str]) -> Result<()> {
    let ownership = format!("{user}:{user}");
    for path in paths {
        if Path::new(path).exists() {
            run_chown(&ownership, path, true)?;
            println!("Changed ownership of {path} to {user}");
        }
    }
    Ok(())
}

pub fn harden(cfg: &BonesConfig) -> Result<()> {
    harden_path(cfg, &cfg.data.worktree)
}

pub fn harden_release(cfg: &BonesConfig) -> Result<()> {
    let worktree = &cfg.data.worktree;
    let current = Path::new(worktree).join("current");

    if !current.is_symlink() {
        bail!("No current release symlink found at {}", current.display());
    }

    let release_dir = fs::read_link(&current)
        .with_context(|| format!("Failed to read symlink {}", current.display()))?;
    let release_path = if release_dir.is_relative() {
        current.parent().unwrap().join(&release_dir)
    } else {
        release_dir
    };

    // Harden the active release directory
    harden_path(cfg, &release_path.to_string_lossy())?;

    // Harden the shared directory
    let shared_dir = Path::new(worktree).join("shared");
    if shared_dir.exists() {
        harden_path(cfg, &shared_dir.to_string_lossy())?;
    }

    Ok(())
}

fn harden_path(cfg: &BonesConfig, root: &str) -> Result<()> {
    let defaults = &cfg.permissions.defaults;

    // Apply default ownership
    let ownership = format!("{}:{}", defaults.owner, defaults.group);
    run_chown(&ownership, root, true)?;
    println!("Set ownership of {root} to {ownership}");

    // Apply default dir_mode and file_mode
    let dir_mode = parse_mode(&defaults.dir_mode)?;
    let file_mode = parse_mode(&defaults.file_mode)?;
    apply_default_modes(root, dir_mode, file_mode)?;
    println!(
        "Applied default modes: dirs={}, files={}",
        defaults.dir_mode, defaults.file_mode
    );

    // Apply path overrides
    for override_entry in &cfg.permissions.paths {
        let target = Path::new(root).join(&override_entry.path);
        if !target.exists() {
            println!(
                "Warning: override path '{}' does not exist, skipping",
                target.display()
            );
            continue;
        }

        let mode = parse_mode(&override_entry.mode)?;

        if override_entry.recursive {
            apply_recursive_mode(&target, mode)?;
        } else if let Some(ref path_type) = override_entry.path_type {
            match path_type.as_str() {
                "dir" | "file" => apply_single_mode(&target, mode)?,
                other => bail!("Unknown path type: {other}"),
            }
        } else {
            apply_single_mode(&target, mode)?;
        }

        println!(
            "Applied mode {} to {}{}",
            override_entry.mode,
            override_entry.path,
            if override_entry.recursive {
                " (recursive)"
            } else {
                ""
            }
        );
    }

    Ok(())
}

fn run_chown(ownership: &str, path: &str, recursive: bool) -> Result<()> {
    let mut cmd = Command::new("chown");
    if recursive {
        cmd.arg("-R");
    }
    cmd.arg(ownership).arg(path);

    let status = cmd
        .status()
        .with_context(|| format!("Failed to chown {path}"))?;

    if !status.success() {
        bail!("chown {ownership} {path} failed");
    }
    Ok(())
}


fn parse_mode(mode_str: &str) -> Result<u32> {
    u32::from_str_radix(mode_str, 8).with_context(|| format!("Invalid mode: {mode_str}"))
}

fn apply_default_modes(worktree: &str, dir_mode: u32, file_mode: u32) -> Result<()> {
    for entry in WalkDir::new(worktree) {
        let entry = entry.with_context(|| format!("Failed to walk {worktree}"))?;
        // Follow symlinks so a symlink to a directory gets dir_mode, not file_mode
        let metadata = fs::metadata(entry.path())
            .with_context(|| format!("Failed to read metadata for {}", entry.path().display()))?;

        let mode = if metadata.is_dir() {
            dir_mode
        } else {
            file_mode
        };
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
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("chmod {:o} {}: {e}", mode, path.display()),
        )
    })?;
    Ok(())
}
