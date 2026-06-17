use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::Value;

use crate::bootstrap_ssh;
use crate::commands::push;
use crate::config;
use crate::prompts;
use crate::python;
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

    println!(
        "Running {} against {} for {}...",
        style("remote ssl").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(&cfg.ssl.domain).cyan(),
    );

    let ssh_user = bootstrap_ssh::resolve();
    let mut deploy_data = remote_data::ssl(&cfg, &cfg.ssl.domain, &cfg.ssl.email)?;
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from("ssh_user"), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(cfg.data.host.clone()));
        map.insert(String::from("ssh_port"), Value::String(cfg.data.port.clone()));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    python::run_python_with_stdin(
        &["ssl", "apply", "--config", bones_toml.to_str().unwrap_or(".bones/bones.toml")],
        &json,
    )?;

    config::save(&cfg, bones_toml)?;
    push::sync_bones_directory(&cfg)?;

    println!("\n{} SSL setup complete.", style("Done!").green().bold());

    Ok(())
}
