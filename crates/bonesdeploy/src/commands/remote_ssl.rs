use std::path::Path;

use anyhow::{Result, bail};
use console::style;
use serde_json::json;
use shared::paths::{ssl_certificate_key_path, ssl_certificate_path};

use crate::commands::push;
use crate::commands::remote_setup;
use crate::config;

const SSL_PLAYBOOK_TAGS: [&str; 2] = ["--tags", "ssl"];

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

    let extra_vars = ssl_extra_vars(&cfg.ssl.domain, &cfg.ssl.email);

    let mut cfg_for_run = cfg.clone();
    cfg_for_run.ssl.enabled = false;

    let ssh_user = remote_setup::resolve_bootstrap_ssh_user();
    remote_setup::run_ansible_playbook(&cfg_for_run, &ssh_user, extra_vars, &ssl_playbook_args())?;

    cfg.ssl.enabled = true;
    config::save(&cfg, bones_yaml)?;
    push::sync_bones_directory(&cfg)?;

    println!("\n{} SSL setup complete.", style("Done!").green().bold());

    Ok(())
}

fn ssl_extra_vars(domain: &str, email: &str) -> serde_json::Value {
    json!({
        "ssl_enabled": true,
        "ssl_domain": domain,
        "ssl_email": email,
        "nginx_ssl_certificate_path": ssl_certificate_path(domain),
        "nginx_ssl_certificate_key_path": ssl_certificate_key_path(domain),
    })
}

fn ssl_playbook_args() -> Vec<String> {
    SSL_PLAYBOOK_TAGS.iter().map(|value| String::from(*value)).collect()
}

#[cfg(test)]
mod tests {
    use super::{ssl_extra_vars, ssl_playbook_args};

    /// Passes the SSL enabled flag as a typed JSON boolean in extra vars.
    #[test]
    fn ssl_extra_vars_pass_enabled_as_typed_json_boolean() {
        let vars = ssl_extra_vars("app.example.com", "ops@example.com");

        assert_eq!(vars.get("ssl_enabled"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(vars.get("ssl_domain"), Some(&serde_json::Value::String(String::from("app.example.com"))));
        assert_eq!(vars.get("ssl_email"), Some(&serde_json::Value::String(String::from("ops@example.com"))));
    }

    /// Runs `remote ssl` through the SSL-only Ansible tag path instead of replaying the nginx setup tag path.
    #[test]
    fn remote_ssl_uses_ssl_only_ansible_tags() {
        assert_eq!(ssl_playbook_args(), vec![String::from("--tags"), String::from("ssl")]);
    }
}
