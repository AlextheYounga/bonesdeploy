use std::path::Path;

use anyhow::{Result, bail};
use bonesdeploy_core::paths;
use console::style;

use crate::config;
use crate::ui::output;
use crate::ui::prompts;

pub fn run(yes: bool, domain: Option<String>, email: Option<String>) -> Result<()> {
    let env_file = Path::new(paths::DOT_ENV);
    let mut cfg = config::load(env_file)?;
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
    if !yes && !prompts::confirm_site_ssl()? {
        println!("Skipped HTTPS setup.");
        println!();
        println!("{}", output::next_step_with_detail("bonesdeploy site ssl", "when DNS is ready"));
        return Ok(());
    }
    println!("{} {}", style("Configuring HTTPS for").cyan().bold(), style(&cfg.domain).bold());
    bonesinfra::run(&["ssl", "apply", "--env-file", paths::DOT_ENV])?;
    cfg.ssl_enabled = true;
    config::save(&cfg, env_file)?;
    println!("{} HTTPS configured.", output::success_marker());
    println!();
    println!("{}", output::next_step("bonesdeploy deploy"));
    Ok(())
}
