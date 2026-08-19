use anyhow::Result;
use bonesdeploy_core::paths;
use std::env;

pub use bonesdeploy_core::config::{Bones, load, save};

/// Resolves the SSH user for provisioning commands: `BONES_BOOTSTRAP_SSH_USER`
/// overrides the configured `ssh_user`; blank values fall back to `root`.
pub fn bootstrap_ssh_user(config: &Bones) -> String {
    if let Ok(env_user) = env::var("BONES_BOOTSTRAP_SSH_USER") {
        let trimmed = env_user.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let trimmed = config.ssh_user.trim();
    if trimmed.is_empty() { String::from("root") } else { trimmed.to_string() }
}

pub fn default_project_root_for(project_name: &str) -> String {
    paths::default_project_root_for(project_name)
}

pub fn repo_directory_name() -> Result<String> {
    let cwd = env::current_dir()?;
    Ok(cwd.file_name().map_or_else(|| String::from("project"), |n| n.to_string_lossy().to_string()))
}
