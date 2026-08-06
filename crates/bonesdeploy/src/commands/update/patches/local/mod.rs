use std::path::Path;

use anyhow::{Result, bail};
use bonesdeploy_core::paths;

use crate::config;

mod config_repo;
mod root_config_repo;

pub(super) struct Context<'a> {
    pub(super) cfg: &'a config::Bones,
    pub(super) bones_dir: &'a Path,
}

pub(super) fn config_repo_url(cfg: &config::Bones) -> String {
    let repository = paths::default_bones_repo_path_for(&cfg.project_name);
    if cfg.port == "22" {
        format!("root@{}:{repository}", cfg.host)
    } else {
        format!("ssh://root@{}:{}{repository}", cfg.host, cfg.port)
    }
}

pub(super) fn apply(id: &str, context: &Context<'_>) -> Result<()> {
    match id {
        config_repo::ID => config_repo::apply(context),
        root_config_repo::ID => root_config_repo::apply(context),
        _ => bail!("Unknown local patch {id}"),
    }
}

pub(super) const CONFIG_REPO_ID: &str = config_repo::ID;
pub(super) const ROOT_CONFIG_REPO_ID: &str = root_config_repo::ID;
