use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::process::Command;

use bonesdeploy_core::paths;

use crate::commands::doctor::security::types::Account;

pub(crate) fn collect_accounts() -> Result<BTreeMap<String, Account>, String> {
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

pub(crate) fn collect_identity_groups(mut account: Account) -> Result<Account, String> {
    let output = Command::new("id")
        .args(["-G", "--", &account.name])
        .output()
        .map_err(|error| format!("cannot resolve supplementary groups for {}: {error}", account.name))?;
    if !output.status.success() {
        return Err(format!(
            "cannot resolve supplementary groups for {}: {}",
            account.name,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let output = String::from_utf8(output.stdout)
        .map_err(|error| format!("group IDs for {} are not valid UTF-8: {error}", account.name))?;
    let groups = output
        .split_whitespace()
        .map(|group| group.parse().map_err(|_| format!("invalid group ID for {}: {group}", account.name)))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if groups.is_empty() || !groups.contains(&account.gid) {
        return Err(format!("resolved groups for {} do not contain primary GID {}", account.name, account.gid));
    }
    account.groups = groups;
    Ok(account)
}
