use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Local;

use crate::config;
use crate::permissions;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let worktree = &cfg.data.worktree;

    // Derive git_dir from config path: {git_dir}/bones/bones.toml
    let config_abs = fs::canonicalize(config_path)
        .with_context(|| format!("Failed to resolve config path: {config_path}"))?;
    let git_dir = config_abs
        .parent() // bones/
        .and_then(|p| p.parent()) // {git_dir}/
        .with_context(|| "Could not derive git_dir from config path")?;

    let releases_dir = Path::new(worktree).join("releases");
    let shared_dir = Path::new(worktree).join("shared");

    // Create releases/ and shared/ if missing
    if !releases_dir.exists() {
        fs::create_dir_all(&releases_dir)
            .with_context(|| format!("Failed to create {}", releases_dir.display()))?;
        println!("Created {}", releases_dir.display());
    }
    if !shared_dir.exists() {
        fs::create_dir_all(&shared_dir)
            .with_context(|| format!("Failed to create {}", shared_dir.display()))?;
        println!("Created {}", shared_dir.display());
    }

    // Generate timestamp release name
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let release_dir = releases_dir.join(&timestamp);

    fs::create_dir_all(&release_dir)
        .with_context(|| format!("Failed to create {}", release_dir.display()))?;
    println!("Created release directory: {}", release_dir.display());

    // Chown new release dir + shared dir to deploy user
    let deploy_user = &cfg.permissions.defaults.deploy;
    permissions::chown_paths(
        deploy_user,
        &[
            &release_dir.to_string_lossy(),
            &shared_dir.to_string_lossy(),
        ],
    )?;

    // Write release name to state file
    let state_file = git_dir.join("bones").join(".current_release");
    fs::write(&state_file, &timestamp)
        .with_context(|| format!("Failed to write {}", state_file.display()))?;
    println!("Wrote current release to {}", state_file.display());

    Ok(())
}
