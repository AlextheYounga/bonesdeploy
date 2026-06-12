use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::Value;
use shared::paths::{self, DeploymentPaths, ssl_certificate_key_path, ssl_certificate_path};

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

    remote_setup::ensure_pyinfra_installed()?;

    println!(
        "Running {} against {} for {}...",
        style("remote ssl").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(&cfg.ssl.domain).cyan(),
    );

    let data_vars = build_ssl_data_vars(&cfg, &cfg.ssl.domain, &cfg.ssl.email);

    let ssh_user = remote_setup::resolve_bootstrap_ssh_user();
    let deploy_file = Path::new(config::Constants::BONES_REMOTE_SSL_DEPLOY);
    remote_setup::run_pyinfra_deploy(
        &cfg,
        &ssh_user,
        &data_vars,
        &remote_setup::PyinfraDeploy { extra_args: &[], deploy_file },
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

    let runtime_deploy = bones_dir.join("infra/runtime.py");
    if !runtime_deploy.is_file() {
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

fn build_ssl_data_vars(cfg: &config::BonesConfig, domain: &str, email: &str) -> Value {
    let paths =
        DeploymentPaths::new(&cfg.data.project_name, &cfg.data.repo_path, &cfg.data.project_root, &cfg.data.web_root);
    let mut vars = serde_json::Map::new();

    vars.insert(String::from("ssl_domain"), Value::String(domain.to_string()));
    vars.insert(String::from("ssl_email"), Value::String(email.to_string()));
    vars.insert(String::from("nginx_ssl_certificate_path"), Value::String(ssl_certificate_path(domain)));
    vars.insert(String::from("nginx_ssl_certificate_key_path"), Value::String(ssl_certificate_key_path(domain)));
    vars.insert(String::from("project_name"), Value::String(cfg.data.project_name.clone()));
    vars.insert(String::from("service_user"), Value::String(config::service_user(&cfg.data.project_name)));
    vars.insert(String::from("group"), Value::String(String::from(paths::DEFAULT_GROUP)));
    vars.insert(String::from("paths"), serde_json::to_value(paths).unwrap_or_default());

    Value::Object(vars)
}

#[cfg(test)]
mod tests {
    use crate::config::{BonesConfig, Data, PermissionDefaults, Permissions, Shared};

    use super::build_ssl_data_vars;

    fn test_cfg() -> BonesConfig {
        BonesConfig {
            data: Data {
                project_name: String::from("test"),
                repo_path: String::from("/home/git/test.git"),
                project_root: String::from("/srv/test"),
                web_root: String::from("public"),
                host: String::from("example.com"),
                port: String::from("22"),
                branch: String::from("master"),
                remote_name: String::from("production"),
                deploy_on_push: true,
            },
            permissions: Permissions {
                defaults: PermissionDefaults { dir_mode: String::from("750"), file_mode: String::from("640") },
                paths: vec![],
            },
            releases: Default::default(),
            shared: Shared::default(),
            ssl: Default::default(),
        }
    }

    /// Passes the SSL domain and email into the data vars sent to the pyinfra SSL deploy.
    #[test]
    fn ssl_data_vars_includes_domain_and_email() {
        let cfg = test_cfg();
        let vars = build_ssl_data_vars(&cfg, "app.example.com", "ops@example.com");

        assert_eq!(vars.get("ssl_domain"), Some(&serde_json::Value::String(String::from("app.example.com"))));
        assert_eq!(vars.get("ssl_email"), Some(&serde_json::Value::String(String::from("ops@example.com"))));
    }
}
