//! Read-only security checks for the authority boundaries around a site.
//!
//! Collection deliberately uses only metadata and account databases.  The
//! evaluator is separate so its rules can be exercised without a host.

mod collection;
mod evaluators;
mod fs;
mod types;

use shared::paths;

use collection::{collect_accounts, collect_sites};
use evaluators::{evaluate_active_release, evaluate_identities, evaluate_privileged_paths, evaluate_runtime_sudo};
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
    let Some(deploy) = accounts.get(paths::DEPLOY_USER) else {
        return Report {
            findings: vec![unverified(
                "Site identity isolation",
                format!("deploy user {} is absent", paths::DEPLOY_USER),
            )],
        };
    };

    let mut findings = vec![evaluate_identities(&sites, deploy)];
    findings.extend(evaluate_runtime_sudo(&sites));
    findings.extend(evaluate_privileged_paths(&sites, deploy));
    findings.extend(sites.iter().map(evaluate_active_release));
    Report { findings }
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
