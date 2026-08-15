use std::fs;
use std::path::Path;

use anyhow::Result;
use console::style;

mod config;
mod framework;
mod scaffold;

pub struct Args {
    pub non_interactive: bool,
    pub project_name: Option<String>,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
    pub template: Option<String>,
    pub runtime_backend: Option<String>,
    pub framework_vars: Vec<String>,
    pub services: Vec<String>,
}

use crate::commands::secrets;
use crate::config as bones_config;
use crate::infra::git;
use crate::ui::output;
use bonesdeploy_core::paths;

#[derive(Debug)]
pub(super) struct FrameworkSelection {
    template: Option<String>,
    config: serde_json::Map<String, serde_json::Value>,
}

pub fn run(args: &Args) -> Result<()> {
    run_with_prefetch(args, || Ok(()))
}

fn run_with_prefetch(args: &Args, prefetch_bonesinfra: impl FnOnce() -> Result<()>) -> Result<()> {
    git::ensure_git_repository()?;

    println!("{} {}", style("Initializing").cyan().bold(), style("bonesdeploy").bold());
    prefetch_bonesinfra()?;

    if fs::symlink_metadata(paths::OLD_BONES_DIR).is_ok() {
        anyhow::bail!("Old .bones layout detected. Run `bonesdeploy migrate` first.");
    }

    let infra_dir = Path::new(paths::LOCAL_INFRA_DIR);
    let is_fresh = !infra_dir.exists();
    if !is_fresh {
        println!("Using existing infra/ configuration.");
    }

    let mut cfg = config::collect_fresh_config(args)?;
    let framework_selection = if is_fresh { Some(framework::collect_framework_config(args)?) } else { None };
    if is_fresh {
        cfg.services.services = framework::collect_database_services(args)?;
        if let Some(framework) = framework_selection {
            scaffold::materialize_project(&mut cfg, framework)?;
        }
    }

    scaffold::update_gitignore()?;
    scaffold::ensure_env_build()?;
    bones_config::save(&cfg, Path::new(paths::DOT_ENV))?;
    secrets::initialize_defaults(&cfg)?;

    if is_fresh {
        println!("{} bonesdeploy initialized.", output::success_marker());
    } else {
        println!("{} bonesdeploy config updated.", output::success_marker());
    }

    scaffold::ensure_local_remote(&cfg)?;
    print_follow_up_hint();
    Ok(())
}

fn print_follow_up_hint() {
    println!();
    println!("{}", output::next_step_with_detail("bonesdeploy setup", "to setup the remote server"));
}
