//! Read-only security checks for the authority boundaries around a site.
//!
//! Collection deliberately uses only metadata and account databases.  The
//! evaluator is separate so its rules can be exercised without a host.

mod collection;
mod evaluators;
mod fs;
mod types;

use bonesdeploy_core::paths;
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::ui;

use collection::{
    collect_accounts, collect_identity_groups, collect_path_tree, collect_release, collect_sites, collect_sudo_policy,
};
use evaluators::{evaluate_active_release, evaluate_identities, evaluate_privileged_path, evaluate_runtime_sudo};
use types::{Finding, unverified};

pub(super) struct Report {
    findings: Vec<Finding>,
}

impl Report {
    pub(super) fn render(&self) {
        let mut rendered_rules = BTreeSet::new();
        for finding in &self.findings {
            if !rendered_rules.insert(finding.rule) {
                continue;
            }

            let failures: Vec<_> = self
                .findings
                .iter()
                .filter(|candidate| candidate.rule == finding.rule && candidate.status.requires_failure())
                .collect();
            if failures.is_empty() {
                println!("{} {}", ui::success_marker(), finding.rule);
                continue;
            }

            for failure in failures {
                println!("{} {}", ui::failure_marker(), failure.rule);
                println!("  Observed: {}", failure.observed);
                println!("  Next: {}", failure.remediation);
            }
        }
    }

    pub(super) fn required_failures(&self) -> Vec<String> {
        self.findings
            .iter()
            .filter(|finding| finding.status.requires_failure())
            .map(|finding| format!("security rule {}: {}", finding.rule, finding.observed))
            .collect()
    }
}

pub(super) fn audit(site_name: Option<&str>, exhaustive: bool) -> Report {
    let accounts = match collect_accounts() {
        Ok(accounts) => accounts,
        Err(error) => return Report { findings: vec![unverified("Site identity isolation", error)] },
    };
    let sites = match collect_sites(&accounts) {
        Ok(sites) => sites,
        Err(error) => return Report { findings: vec![unverified("Site identity isolation", error)] },
    };
    let Some(deploy) = accounts.get(paths::DEPLOY_USER).cloned() else {
        return Report {
            findings: vec![unverified(
                "Site identity isolation",
                format!("deploy user {} is absent", paths::DEPLOY_USER),
            )],
        };
    };
    let deploy = match collect_identity_groups(deploy) {
        Ok(deploy) => deploy,
        Err(error) => return Report { findings: vec![unverified("Site identity isolation", error)] },
    };

    let mut findings = vec![evaluate_identities(&sites, &deploy)];
    findings.extend(sites.iter().map(|site| {
        let evidence = collect_sudo_policy(&site.runtime.name);
        evaluate_runtime_sudo(&evidence)
    }));

    let mut untrusted: Vec<_> = sites.iter().flat_map(|site| [&site.runtime, &site.build]).collect();
    untrusted.push(&deploy);
    for path in protected_paths() {
        match collect_path_tree(&path, false) {
            Ok(tree) => findings.push(evaluate_privileged_path(&tree, &untrusted)),
            Err(error) => findings.push(unverified("Privileged configuration is root-controlled", error)),
        }
    }
    for site in sites.iter().filter(|site| site_name.is_none_or(|name| site.name == name)) {
        match collect_release(site, exhaustive) {
            Ok(evidence) => findings.push(evaluate_active_release(&evidence, &site.runtime)),
            Err(error) => findings.push(unverified("Release activation is root-controlled", error)),
        }
    }
    Report { findings }
}

fn protected_paths() -> Vec<PathBuf> {
    vec![
        paths::bonesremote_config_root(),
        paths::ETC_SYSTEMD_SYSTEM.into(),
        paths::ETC_SUDOERS_D.into(),
        paths::ETC_NGINX_SITES_AVAILABLE.into(),
        paths::ETC_NGINX_SITES_ENABLED.into(),
        paths::ETC_APPARMOR_D.into(),
        paths::SUDOERS_PATH.into(),
        paths::bonesremote_global_link(),
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::evaluators::evaluate_identities;
    use super::fs::has_login_shell;
    use super::types::{Account, Site, Status};

    fn account(name: &str, uid: u32, gid: u32) -> Account {
        Account {
            name: name.to_string(),
            uid,
            gid,
            shell: "/usr/sbin/nologin".to_string(),
            groups: BTreeSet::from([gid]),
        }
    }

    #[test]
    fn nologin_shells_are_not_interactive() {
        assert!(!has_login_shell("/usr/sbin/nologin"));
        assert!(!has_login_shell("/bin/false"));
        assert!(has_login_shell("/bin/bash"));
    }

    #[test]
    fn duplicate_runtime_identity_fails_isolation() {
        let deploy = account("git", 1000, 1000);
        let sites = vec![
            Site {
                name: "atlas".to_string(),
                project_root: PathBuf::from("/srv/sites/atlas"),
                runtime: account("atlas", 1001, 1001),
                build: account("atlas-build", 1002, 1002),
            },
            Site {
                name: "beacon".to_string(),
                project_root: PathBuf::from("/srv/sites/beacon"),
                runtime: account("beacon", 1001, 1001),
                build: account("beacon-build", 1004, 1004),
            },
        ];

        assert_eq!(evaluate_identities(&sites, &deploy).status, Status::Fail);
    }
}
