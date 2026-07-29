//! Read-only security checks for the authority boundaries around a site.
//!
//! Collection deliberately uses only metadata and account databases.  The
//! evaluator is separate so its rules can be exercised without a host.

mod collection;
mod evaluators;
mod fs;
mod types;

use shared::paths;
use std::path::PathBuf;

use collection::{
    collect_accounts, collect_identity_groups, collect_path_tree, collect_release, collect_sites, collect_sudo_policy,
};
use evaluators::{
    evaluate_active_release, evaluate_deploy_sudo, evaluate_identities, evaluate_privileged_path, evaluate_runtime_sudo,
};
use types::{Finding, unverified};

pub(super) struct Report {
    findings: Vec<Finding>,
}

impl Report {
    pub(super) fn render(&self) {
        for finding in &self.findings {
            println!("\n{}  {}", finding.status.label(), finding.rule);
            println!("  Principle: {}", finding.principle);
            println!("  Expected: {}", finding.expected);
            println!("  Observed: {}", finding.observed);
            println!("  Risk: {}", finding.risk);
            println!("  Remediation: {}", finding.remediation);
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

pub(super) fn audit() -> Report {
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
        let evidence = collect_sudo_policy(&site.runtime.name, &[]);
        evaluate_runtime_sudo(&evidence)
    }));

    let mut untrusted: Vec<_> = sites.iter().flat_map(|site| [&site.runtime, &site.build]).collect();
    untrusted.push(&deploy);
    for path in protected_paths() {
        match collect_path_tree(&path, true) {
            Ok(tree) => findings.push(evaluate_privileged_path(&tree, &untrusted)),
            Err(error) => findings.push(unverified("Privileged configuration is root-controlled", error)),
        }
    }
    for site in &sites {
        match collect_release(site) {
            Ok(evidence) => findings.push(evaluate_active_release(&evidence, &site.runtime)),
            Err(error) => findings.push(unverified("Release activation is root-controlled", error)),
        }
    }
    for (command, should_be_allowed) in deploy_sudo_checks() {
        let evidence = collect_sudo_policy(paths::DEPLOY_USER, &command);
        findings.push(evaluate_deploy_sudo(&evidence, should_be_allowed));
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

fn deploy_sudo_checks() -> Vec<(Vec<String>, bool)> {
    let binary = paths::bonesremote_global_link().display().to_string();
    let approved = [
        vec![&binary, "hook", "post-receive", "--site", "nonexistent"],
        vec![&binary, "service", "restart", "--site", "nonexistent"],
        vec![&binary, "release", "rollback", "--site", "nonexistent"],
        vec![&binary, "release", "drop-failed", "--site", "nonexistent"],
        vec![&binary, "release", "prune", "--site", "nonexistent"],
    ];
    let forbidden = [
        vec!["/bin/sh"],
        vec!["/usr/bin/env"],
        vec!["/usr/bin/sudoedit", "/etc/hosts"],
        vec![&binary, "doctor"],
        vec![&binary, "deploy", "--site", "nonexistent"],
        vec![&binary, "version"],
        vec![&binary, "service", "restart", "--site", "nonexistent", "--unexpected"],
    ];
    approved
        .into_iter()
        .map(|command| (command.into_iter().map(str::to_string).collect(), true))
        .chain(forbidden.into_iter().map(|command| (command.into_iter().map(str::to_string).collect(), false)))
        .collect()
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
