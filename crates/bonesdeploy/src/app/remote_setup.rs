use std::env;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::Value;

use shared::config as shared_config;

use crate::bootstrap_ssh;
use crate::config;
use crate::python;
use super::remote_data;

pub fn run() -> Result<()> {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime_config(Path::new(config::Constants::BONES_DIR))?;

    let ssh_user = bootstrap_ssh::resolve();
    let deploy_authorized_key = resolve_deploy_authorized_key()?;

    let mut deploy_data = remote_data::setup(&cfg, &runtime.web_root, &deploy_authorized_key)?;
    let host = cfg.host;
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from("ssh_user"), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(host));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    python::run_python_with_stdin(
        &["setup", "apply", "--config", bones_toml.to_str().unwrap_or(".bones/bones.toml")],
        &json,
    )?;

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
