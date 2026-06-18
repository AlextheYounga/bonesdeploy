use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::Value;

use shared::config as shared_config;

use crate::bootstrap_ssh;
use super::push_state;
use crate::config;
use crate::prompts;
use crate::python;
use super::remote_data;

pub fn run(domain: Option<String>, email: Option<String>) -> Result<()> {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let mut cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime_config(Path::new(config::Constants::BONES_DIR))?;

    if let Some(value) = domain {
        cfg.domain = value.trim().to_string();
    } else if cfg.domain.is_empty() {
        cfg.domain = prompts::prompt_ssl_domain(Some(&cfg))?;
    }

    if let Some(value) = email {
        cfg.email = value.trim().to_string();
    } else if cfg.email.is_empty() {
        cfg.email = prompts::prompt_ssl_email(Some(&cfg))?;
    }

    if cfg.domain.is_empty() {
        bail!("SSL domain is missing. Pass --domain or set domain in .bones/bones.toml");
    }

    if cfg.email.is_empty() {
        bail!("SSL email is missing. Pass --email or set email in .bones/bones.toml");
    }

    config::save(&cfg, bones_toml)?;

    if !prompts::confirm_remote_ssl()? {
        println!("Skipped remote SSL setup.");
        return Ok(());
    }

    println!(
        "Running {} against {} for {}...",
        style("remote ssl").cyan().bold(),
        style(&cfg.host).cyan(),
        style(&cfg.domain).cyan(),
    );

    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));
    let mut deploy_data = remote_data::ssl(&cfg, &runtime.web_root, &cfg.domain, &cfg.email)?;
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from("ssh_user"), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(cfg.host.clone()));
        map.insert(String::from("ssh_port"), Value::String(cfg.port.clone()));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    python::run_python_with_stdin(
        &["ssl", "apply", "--config", bones_toml.to_str().unwrap_or(".bones/bones.toml")],
        &json,
    )?;

    config::save(&cfg, bones_toml)?;
    push_state::sync_bones_directory(&cfg)?;

    println!("\n{} SSL setup complete.", style("Done!").green().bold());

    Ok(())
}
