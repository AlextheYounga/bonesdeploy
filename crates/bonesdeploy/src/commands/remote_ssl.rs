use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::json;
use shared::paths::{ssl_certificate_key_path, ssl_certificate_path};

use crate::commands::push;
use crate::commands::remote_setup;
use crate::config;
use crate::embedded;
use crate::prompts;

pub fn run(domain: Option<String>, email: Option<String>) -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let mut cfg = config::load(bones_yaml)?;

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
        bail!("SSL domain is missing. Pass --domain or set ssl.domain in .bones/bones.yaml");
    }

    if cfg.ssl.email.is_empty() {
        bail!("SSL email is missing. Pass --email or set ssl.email in .bones/bones.yaml");
    }

    config::save(&cfg, bones_yaml)?;

    ensure_runtime_assets_exist()?;

    remote_setup::ensure_ansible_playbook_installed()?;

    println!(
        "Running {} against {} for {}...",
        style("remote ssl").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(&cfg.ssl.domain).cyan(),
    );

    let extra_vars = ssl_extra_vars(&cfg.ssl.domain, &cfg.ssl.email);

    let ssh_user = remote_setup::resolve_bootstrap_ssh_user();
    remote_setup::run_ansible_playbook(
        &cfg,
        &ssh_user,
        extra_vars,
        &remote_setup::AnsiblePlaybook {
            extra_args: &[],
            playbook: Path::new(config::Constants::BONES_REMOTE_SSL_PLAYBOOK),
            roles_dirs: &[],
        },
    )?;

    config::save(&cfg, bones_yaml)?;
    push::sync_bones_directory(&cfg)?;

    println!("\n{} SSL setup complete.", style("Done!").green().bold());

    Ok(())
}

fn ensure_runtime_assets_exist() -> Result<()> {
    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if !bones_dir.exists() {
        bail!(".bones/ does not exist. Run `bonesdeploy init` first.");
    }

    let runtime_playbook = bones_dir.join("runtime/playbooks/runtime.yml");
    if !runtime_playbook.is_file() {
        embedded::scaffold_runtime_base(bones_dir)?;
    }

    let runtime_yaml = Path::new(config::Constants::BONES_RUNTIME_YAML);
    if !runtime_yaml.is_file() {
        if let Some(parent) = runtime_yaml.parent() {
            fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        config::save_runtime(&serde_json::Map::new(), runtime_yaml)?;
    }

    Ok(())
}

fn ssl_extra_vars(domain: &str, email: &str) -> serde_json::Value {
    json!({
        "ssl_domain": domain,
        "ssl_email": email,
        "nginx_ssl_certificate_path": ssl_certificate_path(domain),
        "nginx_ssl_certificate_key_path": ssl_certificate_key_path(domain),
    })
}

#[cfg(test)]
mod tests {
    use super::ssl_extra_vars;

    #[test]
    fn ssl_extra_vars_includes_domain_and_email() {
        let vars = ssl_extra_vars("app.example.com", "ops@example.com");

        assert_eq!(vars.get("ssl_domain"), Some(&serde_json::Value::String(String::from("app.example.com"))));
        assert_eq!(vars.get("ssl_email"), Some(&serde_json::Value::String(String::from("ops@example.com"))));
    }
}
