use std::env;
use std::io::{self, BufRead};
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::{paths, registry};

use crate::commands::deploy;
use crate::config;
use crate::privileges;

pub fn post_receive(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote hook post-receive")?;
    registry::validate_site_name(site)?;

    let config_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&config_path)
        .with_context(|| format!("Failed to load remote site state from {}", config_path.display()))?;

    if !cfg.deploy_on_push {
        println!("[bonesdeploy] deploy_on_push=false, skipping deployment on push.");
        return Ok(());
    }

    let Some(revision) = resolve_revision(&cfg)? else {
        return Ok(());
    };

    deploy::run_full(config_path.to_string_lossy().as_ref(), Some(&revision))
}

fn resolve_revision(cfg: &config::Bones) -> Result<Option<String>> {
    let target_ref = format!("refs/heads/{}", cfg.branch);

    if force_deploy_requested() {
        let revision = current_revision(&cfg.repo_path, &target_ref)?;
        return Ok(Some(revision));
    }

    for line in io::stdin().lock().lines() {
        let line = line.context("Failed to read post-receive input")?;
        let mut parts = line.split_whitespace();
        let oldrev = parts.next();
        let newrev = parts.next();
        let refname = parts.next();

        if refname != Some(target_ref.as_str()) {
            continue;
        }

        let Some(newrev) = newrev else {
            bail!("Malformed post-receive input for {target_ref}");
        };

        if is_zero_oid(newrev) {
            println!("[bonesdeploy] Push deleted {target_ref}; skipping deployment.");
            return Ok(None);
        }

        let _ = oldrev;
        return Ok(Some(newrev.to_string()));
    }

    println!("[bonesdeploy] Push did not update {target_ref}; skipping deployment.");
    Ok(None)
}

fn force_deploy_requested() -> bool {
    force_deploy_requested_from(env::var("BONES_FORCE_DEPLOY").ok().as_deref())
}

fn force_deploy_requested_from(value: Option<&str>) -> bool {
    value == Some("1")
}

fn current_revision(repo_path: &str, target_ref: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["--git-dir", repo_path, "rev-parse", target_ref])
        .output()
        .with_context(|| format!("Failed to resolve deployment ref {target_ref} in {repo_path}"))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Configured deployment ref not found: {target_ref}\n{stderr}")
}

fn is_zero_oid(oid: &str) -> bool {
    !oid.is_empty() && oid.bytes().all(|byte| byte == b'0')
}

#[cfg(test)]
mod tests {
    use super::{force_deploy_requested_from, is_zero_oid};

    #[test]
    fn zero_oid_detection_accepts_all_zero_hex() {
        assert!(is_zero_oid("0000000000000000000000000000000000000000"));
        assert!(!is_zero_oid("0000000000000000000000000000000000000001"));
        assert!(!is_zero_oid(""));
    }

    #[test]
    fn force_deploy_defaults_off() {
        assert!(!force_deploy_requested_from(None));
        assert!(force_deploy_requested_from(Some("1")));
        assert!(!force_deploy_requested_from(Some("0")));
    }
}
