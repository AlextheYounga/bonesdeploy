use std::fs;
use std::os::unix::fs::chown;
use std::path::Path;

use anyhow::{Context, Result};
use shared::paths;

pub(super) fn chown_tree_to_user(path: &Path, user: &str, group: &str) -> Result<()> {
    let uid = user_uid(user)?;
    let gid = site_group_gid(group)?;
    chown_tree(path, uid, gid)
}

fn chown_tree(path: &Path, uid: u32, gid: u32) -> Result<()> {
    chown(path, Some(uid), Some(gid)).with_context(|| format!("Failed to chown {}", path.display()))?;

    if fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect {} for chown", path.display()))?
        .file_type()
        .is_dir()
    {
        for entry in fs::read_dir(path).with_context(|| format!("Failed to read {} for chown", path.display()))? {
            let entry = entry?;
            chown_tree(&entry.path(), uid, gid)?;
        }
    }

    Ok(())
}

pub(super) fn user_uid(user: &str) -> Result<u32> {
    let passwd = fs::read_to_string(paths::ETC_PASSWD)
        .with_context(|| format!("Failed to read {} while resolving uid for {user}", paths::ETC_PASSWD))?;
    parse_user_uid(&passwd, user)
}

pub(super) fn site_group_gid(group: &str) -> Result<u32> {
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

pub(super) fn parse_user_uid(passwd: &str, user: &str) -> Result<u32> {
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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::parse_user_uid;

    #[test]
    fn parse_user_uid_reads_uid_field() -> Result<()> {
        let passwd = "root:x:0:0:root:/root:/bin/bash\ndemo-build:x:1234:1234::/nonexistent:/usr/sbin/nologin\n";
        assert_eq!(parse_user_uid(passwd, "demo-build")?, 1234);
        Ok(())
    }
}
