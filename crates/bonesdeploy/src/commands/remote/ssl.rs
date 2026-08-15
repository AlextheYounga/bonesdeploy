use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::Value;

use bonesdeploy_core::config as shared_config;
use bonesdeploy_core::paths;

use super::data;
use crate::config;
use crate::ui::output;
use crate::ui::prompts;

pub fn run(yes: bool, domain: Option<String>, email: Option<String>) -> Result<()> {
    let env_file = Path::new(paths::DOT_ENV);
    let mut cfg = config::load(env_file)?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_INFRA_DIR))?;

    if let Some(value) = domain {
        cfg.domain = value.trim().to_string();
    } else if cfg.domain.is_empty() && !yes {
        cfg.domain = prompts::prompt_ssl_domain(Some(&cfg))?;
    }

    if let Some(value) = email {
        cfg.email = value.trim().to_string();
    } else if cfg.email.is_empty() && !yes {
        cfg.email = prompts::prompt_ssl_email(Some(&cfg))?;
    }

    if cfg.domain.is_empty() {
        bail!("SSL domain is missing. Pass --domain or set DOMAIN in root .env");
    }

    if cfg.email.is_empty() {
        bail!("SSL email is missing. Pass --email or set EMAIL in root .env");
    }

    config::save(&cfg, env_file)?;

    if !yes && !prompts::confirm_remote_ssl()? {
        println!("Skipped HTTPS setup.");
        println!();
        println!("{}", output::next_step_with_detail("bonesdeploy remote ssl", "when DNS is ready"));
        return Ok(());
    }

    println!("{} {}", style("Configuring HTTPS for").cyan().bold(), style(&cfg.domain).bold());

    let ssh_user = config::bootstrap_ssh_user(&cfg);
    let mut deploy_data = data::ssl(&cfg, &runtime.web_root, &cfg.domain, &cfg.email);
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from(shared_config::bonesinfra_input::SSH_USER), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(cfg.host.clone()));
        map.insert(String::from(shared_config::bonesinfra_input::SSH_PORT), Value::String(cfg.port.clone()));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    bonesinfra::run_with_stdin(&["ssl", "apply", "--env-file", paths::DOT_ENV], &json)?;

    cfg.ssl_enabled = true;
    config::save(&cfg, env_file)?;
    println!("{} HTTPS configured.", output::success_marker());
    println!();
    println!("{}", output::next_step("bonesdeploy deploy"));

    Ok(())
}
