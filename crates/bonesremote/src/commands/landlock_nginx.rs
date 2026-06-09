use std::collections::BTreeSet;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths::DeploymentPaths;

use crate::config;
use crate::landlock;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let active_release_root =
        fs::canonicalize(release_state::current_release_dir(&cfg)?).context("Failed to resolve current release")?;
    let active_web_root = fs::canonicalize(active_release_root.join(&cfg.data.web_root))
        .with_context(|| format!("Failed to resolve web_root: {}", cfg.data.web_root))?;

    let paths =
        DeploymentPaths::new(&cfg.data.project_name, &cfg.data.repo_path, &cfg.data.project_root, &cfg.data.web_root);
    let socket_dir = PathBuf::from(paths.runtime_socket_dir);
    let nginx_conf = PathBuf::from(paths.repo_nginx_config);
    let policy = build_policy(&active_web_root, &socket_dir, &nginx_conf);

    landlock::restrict_self(&policy)?;

    let mut command = Command::new("nginx");
    command.args(["-c", &nginx_conf.display().to_string(), "-g", "daemon off;"]);

    let exec_error = command.exec();
    bail!("Failed to exec nginx: {exec_error}")
}

fn build_policy(web_root: &Path, socket_dir: &Path, nginx_conf: &Path) -> landlock::Policy {
    let mut read_only_paths = BTreeSet::new();
    read_only_paths.insert(web_root.to_path_buf());
    read_only_paths.insert(nginx_conf.to_path_buf());

    for system_path in landlock::default_system_read_paths() {
        read_only_paths.insert(system_path);
    }

    let mut writable_paths = BTreeSet::new();
    writable_paths.insert(socket_dir.to_path_buf());

    landlock::Policy {
        read_only_paths: read_only_paths.into_iter().collect(),
        writable_paths: writable_paths.into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use shared::paths::{self, DeploymentPaths};

    use super::build_policy;

    /// Includes the nginx configuration path in the Landlock read-only policy set.
    #[test]
    fn build_policy_includes_nginx_conf_in_read_only_paths() {
        let paths = DeploymentPaths::new(
            "acme",
            &paths::default_repo_path_for("acme"),
            &paths::default_project_root_for("acme"),
            paths::DEFAULT_WEB_ROOT,
        );
        let web_root = PathBuf::from(paths.current_web_root);
        let socket_dir = PathBuf::from(paths.runtime_socket_dir);
        let nginx_conf = PathBuf::from(paths.repo_nginx_config);

        let policy = build_policy(&web_root, &socket_dir, &nginx_conf);

        assert!(policy.read_only_paths.iter().any(|path| path == &web_root));
        assert!(policy.read_only_paths.iter().any(|path| path == &nginx_conf));
    }
}
