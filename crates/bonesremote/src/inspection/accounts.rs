pub fn account_exists(passwd: &str, account: &str) -> bool {
    account_entry(passwd, account).is_some()
}

pub fn account_home<'a>(passwd: &'a str, account: &str) -> Option<&'a str> {
    account_field(passwd, account, 5)
}

pub fn account_identity(passwd: &str, account: &str) -> Option<(u32, u32)> {
    let uid = account_field(passwd, account, 2)?.parse().ok()?;
    let gid = account_field(passwd, account, 3)?.parse().ok()?;
    Some((uid, gid))
}

pub fn group_members(groupfile: &str, group: &str) -> Option<Vec<String>> {
    let line = groupfile.lines().find(|line| entry_name(line) == group)?;
    let members = line.split(':').nth(3).unwrap_or_default();
    Some(if members.is_empty() { Vec::new() } else { members.split(',').map(str::to_owned).collect() })
}

fn account_field<'a>(passwd: &'a str, account: &str, index: usize) -> Option<&'a str> {
    account_entry(passwd, account)?.split(':').nth(index)
}

fn account_entry<'a>(passwd: &'a str, account: &str) -> Option<&'a str> {
    passwd.lines().find(|line| entry_name(line) == account)
}

fn entry_name(entry: &str) -> &str {
    entry.split(':').next().unwrap_or_default()
}
