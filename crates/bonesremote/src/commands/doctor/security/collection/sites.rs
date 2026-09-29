use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use bonesdeploy_core::config;
use bonesdeploy_core::paths;

use crate::commands::doctor::security::types::{Account, Site};
use crate::control_plane;

use super::accounts::collect_identity_groups;

pub(crate) fn collect_sites(accounts: &BTreeMap<String, Account>) -> Result<Vec<Site>, String> {
    let root = paths::bonesremote_sites_root();
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("cannot read {}: {error}", root.display())),
    };
    let mut sites = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot enumerate {}: {error}", root.display()))?;
        if !entry.file_type().map_err(|error| error.to_string())?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        control_plane::load(&name).map_err(|error| error.to_string())?;
        sites.push(site_from_descriptor(name, accounts)?);
    }
    Ok(sites)
}

fn site_from_descriptor(name: String, accounts: &BTreeMap<String, Account>) -> Result<Site, String> {
    let runtime_name = config::runtime_user_for(&name);
    let runtime =
        accounts.get(&runtime_name).cloned().ok_or_else(|| format!("runtime user {runtime_name} is absent"))?;
    Ok(Site {
        project_root: PathBuf::from(paths::default_project_root_for(&name)),
        name,
        runtime: collect_identity_groups(runtime)?,
    })
}
