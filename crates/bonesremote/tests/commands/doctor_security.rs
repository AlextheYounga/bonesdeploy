use std::collections::BTreeSet;
use std::path::PathBuf;

use bonesremote::commands::doctor::security::evaluators::evaluate_identities;
use bonesremote::commands::doctor::security::fs::has_login_shell;
use bonesremote::commands::doctor::security::types::{Account, Site, Status};

fn account(name: &str, uid: u32, gid: u32) -> Account {
    Account { name: name.to_string(), uid, gid, shell: "/usr/sbin/nologin".to_string(), groups: BTreeSet::from([gid]) }
}

#[test]
fn nologin_shells_are_not_interactive() {
    assert!(!has_login_shell("/usr/sbin/nologin"));
    assert!(!has_login_shell("/bin/false"));
    assert!(has_login_shell("/bin/bash"));
}

#[test]
fn duplicate_runtime_identity_fails_isolation() {
    let deploy = account("deploy", 1000, 1000);
    let sites = vec![
        Site {
            name: "atlas".to_string(),
            project_root: PathBuf::from("/srv/sites/atlas"),
            runtime: account("atlas", 1001, 1001),
        },
        Site {
            name: "beacon".to_string(),
            project_root: PathBuf::from("/srv/sites/beacon"),
            runtime: account("beacon", 1001, 1001),
        },
    ];

    assert_eq!(evaluate_identities(&sites, &deploy).status, Status::Fail);
}

#[test]
fn site_without_a_build_identity_remains_isolated() {
    let deploy = account("deploy", 1000, 1000);
    let sites = vec![Site {
        name: "atlas".to_string(),
        project_root: PathBuf::from("/srv/sites/atlas"),
        runtime: account("atlas", 1001, 1001),
    }];

    assert_eq!(evaluate_identities(&sites, &deploy).status, Status::Pass);
}
