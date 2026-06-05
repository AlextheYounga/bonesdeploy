use std::path::Path;

use anyhow::{Result, bail};
use console::style;

use crate::commands::push;
use crate::commands::remote_setup;
use crate::config;

pub fn run(domain: Option<String>, email: Option<String>) -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let mut cfg = config::load(bones_yaml)?;

    if let Some(value) = domain {
        cfg.ssl.domain = value;
    }

    if let Some(value) = email {
        cfg.ssl.email = value;
    }

    if cfg.ssl.domain.is_empty() {
        bail!("SSL domain is missing. Pass --domain or set ssl.domain in .bones/bones.yaml");
    }

    if cfg.ssl.email.is_empty() {
        bail!("SSL email is missing. Pass --email or set ssl.email in .bones/bones.yaml");
    }

    config::save(&cfg, bones_yaml)?;

    remote_setup::ensure_ansible_playbook_installed()?;

    println!(
        "Running {} against {} for {}...",
        style("remote ssl").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(&cfg.ssl.domain).cyan(),
    );

    let extra_args = ssl_extra_args(&cfg.ssl.domain, &cfg.ssl.email);

    let mut cfg_for_run = cfg.clone();
    cfg_for_run.ssl.enabled = false;

    let ssh_user = remote_setup::resolve_bootstrap_ssh_user();
    remote_setup::run_ansible_playbook(&cfg_for_run, &ssh_user, &extra_args)?;

    cfg.ssl.enabled = true;
    config::save(&cfg, bones_yaml)?;
    push::sync_bones_directory(&cfg)?;

    println!("\n{} SSL setup complete.", style("Done!").green().bold());

    Ok(())
}

fn ssl_extra_args(domain: &str, email: &str) -> Vec<String> {
    vec![
        String::from("--tags"),
        String::from("nginx,ssl"),
        String::from("-e"),
        remote_setup::build_extra_var_json_bool("ssl_enabled", true),
        String::from("-e"),
        remote_setup::build_extra_var_json_string("ssl_domain", domain),
        String::from("-e"),
        remote_setup::build_extra_var_json_string("ssl_email", email),
    ]
}

#[cfg(test)]
mod tests {
    use super::ssl_extra_args;

    #[test]
    fn ssl_extra_args_pass_enabled_as_typed_json_boolean() {
        let args = ssl_extra_args("app.example.com", "ops@example.com");

        assert!(args.contains(&String::from("{\"ssl_enabled\":true}")));
        assert!(!args.contains(&String::from("ssl_enabled=true")));
    }
}
