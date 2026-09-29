use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use bonesdeploy_core::config::{self, BuildMode, RemoteDeploymentConfig};
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
        let descriptor = control_plane::load(&name).map_err(|error| error.to_string())?;
        sites.push(site_from_descriptor(name, &descriptor, accounts)?);
    }
    Ok(sites)
}

fn site_from_descriptor(
    name: String,
    descriptor: &RemoteDeploymentConfig,
    accounts: &BTreeMap<String, Account>,
) -> Result<Site, String> {
    let runtime_name = config::runtime_user_for(&name);
    let runtime =
        accounts.get(&runtime_name).cloned().ok_or_else(|| format!("runtime user {runtime_name} is absent"))?;
    let build = build_account(&name, descriptor, accounts)?;
    Ok(Site {
        project_root: PathBuf::from(paths::default_project_root_for(&name)),
        name,
        runtime: collect_identity_groups(runtime)?,
        build,
    })
}

fn build_account(
    site: &str,
    descriptor: &RemoteDeploymentConfig,
    accounts: &BTreeMap<String, Account>,
) -> Result<Option<Account>, String> {
    if descriptor.build.mode == BuildMode::Local {
        return Ok(None);
    }
    let build_name = config::build_user_for(site);
    let account = accounts.get(&build_name).cloned().ok_or_else(|| format!("build user {build_name} is absent"))?;
    Ok(Some(collect_identity_groups(account)?))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use bonesdeploy_core::config::{Build, Runtime};

    use super::*;

    fn account(name: &str, uid: u32) -> Account {
        Account { name: name.into(), uid, gid: uid, shell: "/usr/sbin/nologin".into(), groups: BTreeSet::from([uid]) }
    }

    fn descriptor(mode: BuildMode) -> RemoteDeploymentConfig {
        RemoteDeploymentConfig {
            branch: "main".into(),
            releases_keep: 5,
            runtime: Runtime::default(),
            build: Build { mode, ..Build::default() },
        }
    }

    #[test]
    fn local_site_does_not_require_a_build_account() -> Result<(), String> {
        let accounts = BTreeMap::from([("demo".into(), account("demo", 1001))]);
        assert!(build_account("demo", &descriptor(BuildMode::Local), &accounts)?.is_none());
        Ok(())
    }

    #[test]
    fn remote_site_requires_a_build_account() {
        let accounts = BTreeMap::from([("demo".into(), account("demo", 1001))]);
        assert!(site_from_descriptor("demo".into(), &descriptor(BuildMode::Remote), &accounts).is_err());
    }
}
