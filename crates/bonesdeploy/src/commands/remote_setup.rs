use std::env;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;

use crate::bootstrap_ssh;
use crate::config;
use crate::pyinfra;
use crate::pyinfra::PyinfraDeploy;
use crate::remote_data;

pub fn run() -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let deploy_file = Path::new(config::Constants::BONES_REMOTE_SETUP_DEPLOY);
    if !deploy_file.is_file() {
        bail!("Missing remote setup deploy file: {}", deploy_file.display());
    }

    pyinfra::ensure_pyinfra_installed()?;

    let ssh_user = bootstrap_ssh::resolve();
    let deploy_authorized_key = resolve_deploy_authorized_key()?;

    let deploy_data = remote_data::setup(&cfg, &deploy_authorized_key)?;
    pyinfra::run_pyinfra_deploy(&cfg, &ssh_user, &deploy_data, &PyinfraDeploy { extra_args: &[], deploy_file })?;

    println!("{} Remote setup complete.", style("Done!").green().bold());

    Ok(())
}

fn resolve_deploy_authorized_key() -> Result<String> {
    if let Some(path) = env::var("BONES_DEPLOY_PUBLIC_KEY_PATH").ok().filter(|value| !value.trim().is_empty()) {
        return read_public_key(Path::new(path.trim()));
    }

    let home = env::var("HOME").context("HOME is not set; cannot discover SSH public key")?;
    let ssh_dir = Path::new(&home).join(".ssh");
    let candidates = ["id_ed25519.pub", "id_ecdsa.pub", "id_rsa.pub"];

    for candidate in candidates {
        let path = ssh_dir.join(candidate);
        if path.is_file() {
            return read_public_key(&path);
        }
    }

    bail!(
        "No SSH public key found for deploy user setup. Set BONES_DEPLOY_PUBLIC_KEY_PATH or create one of: ~/.ssh/id_ed25519.pub, ~/.ssh/id_ecdsa.pub, ~/.ssh/id_rsa.pub"
    )
}

fn read_public_key(path: &Path) -> Result<String> {
    let key = fs::read_to_string(path).with_context(|| format!("Failed to read SSH public key: {}", path.display()))?;
    let key = key.trim().to_string();
    if key.is_empty() {
        bail!("SSH public key file is empty: {}", path.display());
    }
    Ok(key)
}
