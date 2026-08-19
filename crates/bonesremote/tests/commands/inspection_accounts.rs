use bonesremote::inspection::accounts::{account_exists, account_home, account_identity, group_members};

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
