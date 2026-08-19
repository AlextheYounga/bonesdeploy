pub(crate) fn account_exists(passwd: &str, account: &str) -> bool {
    account_entry(passwd, account).is_some()
}

pub(crate) fn account_home<'a>(passwd: &'a str, account: &str) -> Option<&'a str> {
    account_field(passwd, account, 5)
}

pub(crate) fn account_identity(passwd: &str, account: &str) -> Option<(u32, u32)> {
    let uid = account_field(passwd, account, 2)?.parse().ok()?;
    let gid = account_field(passwd, account, 3)?.parse().ok()?;
    Some((uid, gid))
}

pub(crate) fn group_members(groupfile: &str, group: &str) -> Option<Vec<String>> {
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

#[cfg(test)]
mod tests {
    use super::{account_exists, account_home, account_identity, group_members};

    #[test]
    fn parses_passwd_accounts_without_prefix_collisions() {
        let passwd = "demo:x:1000:1000::/srv:/usr/sbin/nologin\ndemolition:x:1001:1001::/tmp:/bin/sh\n";

        assert!(account_exists(passwd, "demo"));
        assert!(!account_exists(passwd, "git"));
        assert_eq!(account_home(passwd, "demo"), Some("/srv"));
        assert_eq!(account_identity(passwd, "demo"), Some((1000, 1000)));
    }

    #[test]
    fn parses_group_members_and_missing_groups() {
        assert_eq!(
            group_members("demo:x:1000:git,www-data\n", "demo"),
            Some(vec!["git".to_string(), "www-data".to_string()])
        );
        assert_eq!(group_members("demo:x:1000:\n", "demo"), Some(Vec::new()));
        assert_eq!(group_members("demo:x:1000:\n", "nope"), None);
    }
}
