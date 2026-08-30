use std::fs;
use std::os::unix::fs::lchown;
use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;

pub fn chown_tree_to_user(path: &Path, user: &str, group: &str) -> Result<()> {
    let uid = user_uid(user)?;
    let gid = site_group_gid(group)?;
    chown_tree(path, uid, gid)
}

fn chown_tree(path: &Path, uid: u32, gid: u32) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect {} for chown", path.display()))?
        .file_type();
    // Repository source may contain symlinks. lchown changes the link itself,
    // never a target outside the exported workspace.
    lchown(path, Some(uid), Some(gid)).with_context(|| format!("Failed to chown {}", path.display()))?;

    if metadata.is_dir() {
        for entry in fs::read_dir(path).with_context(|| format!("Failed to read {} for chown", path.display()))? {
            let entry = entry?;
            chown_tree(&entry.path(), uid, gid)?;
        }
    }

    Ok(())
}

pub fn user_uid(user: &str) -> Result<u32> {
    let passwd = fs::read_to_string(paths::ETC_PASSWD)
        .with_context(|| format!("Failed to read {} while resolving uid for {user}", paths::ETC_PASSWD))?;
    parse_user_uid(&passwd, user)
}

pub fn site_group_gid(group: &str) -> Result<u32> {
    let groupfile = fs::read_to_string(paths::ETC_GROUP)
        .with_context(|| format!("Failed to read {} while sealing release", paths::ETC_GROUP))?;
    let line = groupfile
        .lines()
        .find(|line| line.starts_with(&format!("{group}:")))
        .with_context(|| format!("Site group '{group}' is missing from /etc/group"))?;
    let fields: Vec<&str> = line.split(':').collect();
    let gid = fields
        .get(2)
        .with_context(|| format!("Group '{group}' missing gid field"))?
        .parse::<u32>()
        .with_context(|| format!("Group '{group}' gid is not a valid integer"))?;
    Ok(gid)
}

pub fn parse_user_uid(passwd: &str, user: &str) -> Result<u32> {
    let line = passwd
        .lines()
        .find(|line| line.starts_with(&format!("{user}:")))
        .with_context(|| format!("User '{user}' missing from {}", paths::ETC_PASSWD))?;
    let fields: Vec<&str> = line.split(':').collect();
    fields
        .get(2)
        .with_context(|| format!("User '{user}' missing uid field"))?
        .parse::<u32>()
        .with_context(|| format!("User '{user}' uid is not a valid integer"))
}
