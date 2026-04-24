use std::path::Path;

use anyhow::{Result, bail};
use console::style;

use crate::commands::server_setup;
use crate::config;

pub fn run(domain: Option<String>, email: Option<String>) -> Result<()> {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let mut cfg = config::load(bones_toml)?;

    if let Some(value) = domain {
        cfg.ssl.domain = value;
    }

    if let Some(value) = email {
        cfg.ssl.email = value;
    }

    if cfg.ssl.domain.is_empty() {
        bail!("SSL domain is missing. Pass --domain or set ssl.domain in .bones/bones.toml");
    }

    if cfg.ssl.email.is_empty() {
        bail!("SSL email is missing. Pass --email or set ssl.email in .bones/bones.toml");
    }

    config::save(&cfg, bones_toml)?;

    server_setup::ensure_ansible_playbook_installed()?;

    println!(
        "Running {} against {} for {}...",
        style("server ssl").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(&cfg.ssl.domain).cyan(),
    );

    let extra_args = vec![
        String::from("--tags"),
        String::from("nginx,ssl"),
        String::from("-e"),
        String::from("ssl_enabled=true"),
        String::from("-e"),
        format!("ssl_domain={}", cfg.ssl.domain),
        String::from("-e"),
        format!("ssl_email={}", cfg.ssl.email),
    ];

    let mut cfg_for_run = cfg.clone();
    cfg_for_run.ssl.enabled = false;

    server_setup::run_ansible_playbook(&cfg_for_run, &cfg.permissions.defaults.deploy_user, &extra_args)?;

    cfg.ssl.enabled = true;
    config::save(&cfg, bones_toml)?;

    println!("\n{} SSL setup complete.", style("Done!").green().bold());

    Ok(())
}
