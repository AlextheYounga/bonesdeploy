use std::collections::BTreeSet;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::landlock;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let active_runtime_root = fs::canonicalize(&cfg.data.live_root)
        .with_context(|| format!("Failed to resolve live_root: {}", cfg.data.live_root))?;

    let socket_dir = PathBuf::from("/run").join(&cfg.data.project_name);
    let policy = build_policy(&active_runtime_root, &socket_dir);

    landlock::restrict_self(&policy)?;

    let nginx_conf = format!("{}/bones/nginx.conf", cfg.data.git_dir);
    let mut command = Command::new("nginx");
    command.args(["-c", &nginx_conf, "-g", "daemon off;"]);

    let exec_error = command.exec();
    bail!("Failed to exec nginx: {exec_error}")
}

fn build_policy(runtime_root: &Path, socket_dir: &Path) -> landlock::Policy {
    let mut read_only_paths = BTreeSet::new();
    read_only_paths.insert(runtime_root.to_path_buf());

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
