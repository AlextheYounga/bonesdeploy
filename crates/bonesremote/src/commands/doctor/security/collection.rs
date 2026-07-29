use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use shared::{config, paths};

use super::types::{Account, Site};

pub(super) fn collect_accounts() -> Result<BTreeMap<String, Account>, String> {
    let passwd =
        fs::read_to_string(paths::ETC_PASSWD).map_err(|error| format!("cannot read {}: {error}", paths::ETC_PASSWD))?;
    let groups =
        fs::read_to_string(paths::ETC_GROUP).map_err(|error| format!("cannot read {}: {error}", paths::ETC_GROUP))?;
    let mut accounts = BTreeMap::new();
    for line in passwd.lines().filter(|line| !line.is_empty() && !line.starts_with('#')) {
        let fields: Vec<_> = line.split(':').collect();
        if fields.len() < 7 {
            return Err(format!("malformed passwd entry: {line}"));
        }
        let uid = fields[2].parse().map_err(|_| format!("invalid UID for {}", fields[0]))?;
        let gid = fields[3].parse().map_err(|_| format!("invalid GID for {}", fields[0]))?;
        accounts.insert(
            fields[0].to_string(),
            Account {
                name: fields[0].to_string(),
                uid,
                gid,
                shell: fields[6].to_string(),
                groups: BTreeSet::from([gid]),
            },
        );
    }
    for line in groups.lines().filter(|line| !line.is_empty() && !line.starts_with('#')) {
        let fields: Vec<_> = line.split(':').collect();
        if fields.len() < 4 {
            return Err(format!("malformed group entry: {line}"));
        }
        let gid = fields[2].parse().map_err(|_| format!("invalid group ID for {}", fields[0]))?;
        for member in fields[3].split(',').filter(|member| !member.is_empty()) {
            let account = accounts
                .get_mut(member)
                .ok_or_else(|| format!("group {} references unknown user {member}", fields[0]))?;
            account.groups.insert(gid);
        }
    }
    Ok(accounts)
}

pub(super) fn collect_sites(accounts: &BTreeMap<String, Account>) -> Result<Vec<Site>, String> {
    let root = paths::bonesremote_sites_root();
    let entries = fs::read_dir(&root).map_err(|error| format!("cannot read {}: {error}", root.display()))?;
    let mut sites = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot enumerate {}: {error}", root.display()))?;
        if !entry.file_type().map_err(|error| error.to_string())?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let cfg = config::load(&entry.path().join(paths::BONES_TOML))
            .map_err(|error| format!("cannot load site {name}: {error}"))?;
        if cfg.project_name != name {
            return Err(format!("site directory {name} contains configuration for {}", cfg.project_name));
        }
        let runtime_name = config::runtime_user_for(&name);
        let build_name = config::build_user_for(&name);
        let runtime =
            accounts.get(&runtime_name).cloned().ok_or_else(|| format!("runtime user {runtime_name} is absent"))?;
        let build = accounts.get(&build_name).cloned().ok_or_else(|| format!("build user {build_name} is absent"))?;
        sites.push(Site { name, project_root: PathBuf::from(&cfg.project_root), runtime, build });
    }
    Ok(sites)
}
