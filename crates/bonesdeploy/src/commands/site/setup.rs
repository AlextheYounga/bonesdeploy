use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::{config::RuntimeBackend, paths};
use bonesinfra::provisioning::apply_site_setup;

use crate::build::native;
use crate::commands::server;
use crate::config;
use crate::ui::output;
use crate::ui::prompts;

pub async fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_site_setup()? {
        println!("Skipped.");
        return Ok(());
    }

    server::doctor(false).await.context("Server baseline is not ready.\n\nNext: bonesdeploy server setup --yes")?;
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    if cfg.runtime.backend == RuntimeBackend::Native {
        println!("Checking local builder...");
        native::ensure_builder_image().context("Failed to prepare local builder")?;
    }
    println!("Provisioning site base...");
    apply_site_setup(&cfg, Path::new(".")).await?;
    super::runtime::apply().await?;

    super::doctor::run_with_pending(false, false).await.context("Site setup failed while checking site")?;
    println!();
    println!("{} Site setup complete.", output::success_marker());
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    print_next_step(&cfg).await;
    Ok(())
}

async fn print_next_step(cfg: &config::Bones) {
    if cfg.domain.is_empty() {
        println!("{}", output::next_step("bonesdeploy deploy"));
        println!("Optional public preview: run `bonesdeploy site tunnel start` to create an ephemeral Cloudflare URL.");
    } else {
        println!("{}", output::next_step_with_detail("bonesdeploy site ssl", "to configure HTTPS"));
    }
}
