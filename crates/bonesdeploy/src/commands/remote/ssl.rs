use std::path::Path;

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::Value;

use shared::config as shared_config;
use shared::paths;

use super::data;
use crate::commands::push_state;
use crate::config;
use crate::ui::output;
use crate::ui::prompts;

pub fn run(yes: bool, domain: Option<String>, email: Option<String>) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let mut cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;

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
        bail!("SSL domain is missing. Pass --domain or set domain in .bones/bones.toml");
    }

    if cfg.email.is_empty() {
        bail!("SSL email is missing. Pass --email or set email in .bones/bones.toml");
    }

    config::save(&cfg, bones_toml)?;

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
    bonesinfra::run_with_stdin(&["ssl", "apply", "--config", paths::LOCAL_BONES_TOML], &json)?;

    cfg.ssl_enabled = true;
    config::save(&cfg, bones_toml)?;
    push_state::sync_bones_directory(&cfg)?;

    println!("{} HTTPS configured.", output::success_marker());
    println!();
    println!("{}", output::next_step("bonesdeploy deploy"));

    Ok(())
}
