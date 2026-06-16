use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;

use crate::bootstrap_ssh;
use crate::commands::push;
use crate::config;
use crate::embedded;
use crate::prompts;
use crate::pyinfra;
use crate::pyinfra::PyinfraDeploy;
use crate::remote_data;

pub fn run(domain: Option<String>, email: Option<String>) -> Result<()> {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let mut cfg = config::load(bones_toml)?;

    if let Some(value) = domain {
        cfg.ssl.domain = value.trim().to_string();
    } else if cfg.ssl.domain.is_empty() {
        cfg.ssl.domain = prompts::prompt_ssl_domain(Some(&cfg))?;
    }

    if let Some(value) = email {
        cfg.ssl.email = value.trim().to_string();
    } else if cfg.ssl.email.is_empty() {
        cfg.ssl.email = prompts::prompt_ssl_email(Some(&cfg))?;
    }

    if cfg.ssl.domain.is_empty() {
        bail!("SSL domain is missing. Pass --domain or set ssl.domain in .bones/bones.toml");
    }

    if cfg.ssl.email.is_empty() {
        bail!("SSL email is missing. Pass --email or set ssl.email in .bones/bones.toml");
    }

    config::save(&cfg, bones_toml)?;

    if !prompts::confirm_remote_ssl()? {
        println!("Skipped remote SSL setup.");
        return Ok(());
    }

    ensure_runtime_assets_exist()?;

    pyinfra::ensure_pyinfra_installed()?;

    println!(
        "Running {} against {} for {}...",
        style("remote ssl").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(&cfg.ssl.domain).cyan(),
    );

    let deploy_data = remote_data::ssl(&cfg, &cfg.ssl.domain, &cfg.ssl.email)?;

    let ssh_user = bootstrap_ssh::resolve();
    let deploy_file = Path::new(config::Constants::BONES_REMOTE_SSL_DEPLOY);
    pyinfra::run_pyinfra_deploy(&cfg, &ssh_user, &deploy_data, &PyinfraDeploy { extra_args: &[], deploy_file })?;

    config::save(&cfg, bones_toml)?;
    push::sync_bones_directory(&cfg)?;

    println!("\n{} SSL setup complete.", style("Done!").green().bold());

    Ok(())
}

fn ensure_runtime_assets_exist() -> Result<()> {
    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if !bones_dir.exists() {
        bail!(".bones/ does not exist. Run `bonesdeploy init` first.");
    }

    embedded::scaffold_runtime_base(bones_dir)?;

    let runtime_toml = Path::new(config::Constants::BONES_RUNTIME_TOML);
    if !runtime_toml.is_file() {
        if let Some(parent) = runtime_toml.parent() {
            fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        config::save_runtime(&serde_json::Map::new(), runtime_toml)?;
    }

    Ok(())
}
