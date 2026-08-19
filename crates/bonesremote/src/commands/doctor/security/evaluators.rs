use std::collections::BTreeSet;
use std::path::PathBuf;

use super::fs::{Authority, account_can_modify, has_login_shell};
use super::types::{
    Account, CurrentState, FileKind, Finding, PathTree, PolicyDecision, ReleaseEvidence, Rule, Site, Status,
    SudoEvidence, finding, unverified,
};

const RUNTIME_SUDO_RULE: Rule = Rule {
    name: "Runtime sudo authority is absent",
    remediation: "Remove the runtime identity from sudoers and all sudo-capable groups.",
};

const PRIVILEGED_PATH_RULE: Rule = Rule {
    name: "Privileged configuration is root-controlled",
    remediation: "Restore root ownership and remove effective write access from the reported path.",
};

const RELEASE_WRITE_RULE: Rule = Rule {
    name: "Release activation is root-controlled",
    remediation: "Seal releases as root and restore root control of activation directories.",
};

const IDENTITY_RULE: Rule = Rule {
    name: "Site identity isolation",
    remediation: "Assign distinct UIDs and primary GIDs; remove cross-site and deploy-group membership.",
};

pub fn evaluate_identities(sites: &[Site], deploy: &Account) -> Finding {
    let mut runtime_user_ids = BTreeSet::new();
    let mut runtime_primary_groups = BTreeSet::new();
    for site in sites {
        if site.runtime.uid == site.build.uid || site.runtime.gid == site.build.gid {
            return finding(
                Status::Fail,
                IDENTITY_RULE,
                format!("{} runtime and build accounts share an identity", site.name),
            );
        }
        if !runtime_user_ids.insert(site.runtime.uid) || !runtime_primary_groups.insert(site.runtime.gid) {
            return finding(
                Status::Fail,
                IDENTITY_RULE,
                format!("{} reuses another site's runtime UID or primary GID", site.name),
            );
        }
        if has_login_shell(&site.runtime.shell) {
            return finding(
                Status::Fail,
                IDENTITY_RULE,
                format!("runtime user {} has login shell {}", site.runtime.name, site.runtime.shell),
            );
        }
        if deploy.groups.contains(&site.runtime.gid) {
            return finding(
                Status::Fail,
                IDENTITY_RULE,
                format!("deploy user {} belongs to runtime group for {}", deploy.name, site.name),
            );
        }
        for other in sites {
            if site.name != other.name
                && (site.runtime.groups.contains(&other.runtime.gid)
                    || site.runtime.groups.contains(&other.build.gid)
                    || site.build.groups.contains(&other.runtime.gid)
                    || site.build.groups.contains(&other.build.gid))
            {
                return finding(
                    Status::Fail,
                    IDENTITY_RULE,
                    format!("{} account belongs to an identity group for {}", site.name, other.name),
                );
            }
        }
    }
    finding(
        Status::Pass,
        IDENTITY_RULE,
        format!("{} imported site(s) have distinct runtime/build identities", sites.len()),
    )
}

pub fn evaluate_runtime_sudo(evidence: &SudoEvidence) -> Finding {
    match &evidence.decision {
        PolicyDecision::Denied => {
            finding(Status::Pass, RUNTIME_SUDO_RULE, format!("sudo policy denied all authority for {}", evidence.user))
        }
        PolicyDecision::Allowed(output) => {
            finding(Status::Fail, RUNTIME_SUDO_RULE, format!("sudo policy for {}:\n{}", evidence.user, output))
        }
        PolicyDecision::Unverified(error) => unverified(
            RUNTIME_SUDO_RULE.name,
            format!("could not determine sudo authority for {}: {error}", evidence.user),
        ),
    }
}

pub fn evaluate_privileged_path(tree: &PathTree, untrusted: &[&Account]) -> Finding {
    let mut uncertainty = None;
    for node in tree.nodes.values() {
        if node.kind == FileKind::Symlink {
            continue;
        }
        for account in untrusted {
            match account_can_modify(&node.path, account, tree) {
                Authority::Granted => {
                    return finding(
                        Status::Fail,
                        PRIVILEGED_PATH_RULE,
                        format!(
                            "{} can modify {} in the protected tree rooted at {}",
                            account.name,
                            node.path.display(),
                            tree.requested.display()
                        ),
                    );
                }
                Authority::Unverified(error) => {
                    uncertainty.get_or_insert(error);
                }
                Authority::Denied => {}
            }
        }
    }
    if let Some(error) = uncertainty {
        return unverified(PRIVILEGED_PATH_RULE.name, error);
    }
    finding(
        Status::Pass,
        PRIVILEGED_PATH_RULE,
        format!(
            "{} filesystem objects under {} were denied to all untrusted identities",
            tree.nodes.len(),
            tree.requested.display()
        ),
    )
}

pub fn evaluate_active_release(evidence: &ReleaseEvidence, runtime: &Account) -> Finding {
    let Some(target) = active_release_target(evidence) else {
        return release_current_finding(evidence);
    };
    if !target.starts_with(&evidence.releases_root) || target == &evidence.releases_root {
        return finding(
            Status::Fail,
            Rule {
                name: "Release activation is root-controlled",
                remediation: "Reactivate a release contained by the site's releases directory.",
            },
            format!("current resolves to {} outside {}", target.display(), evidence.releases_root.display()),
        );
    }

    let mut uncertainty = None;
    for node in evidence.filesystem.nodes.values() {
        if node.kind == FileKind::Symlink {
            continue;
        }
        match account_can_modify(&node.path, runtime, &evidence.filesystem) {
            Authority::Granted => {
                return finding(
                    Status::Fail,
                    RELEASE_WRITE_RULE,
                    format!("runtime user {} can modify {}", runtime.name, node.path.display()),
                );
            }
            Authority::Unverified(error) => {
                uncertainty.get_or_insert(error);
            }
            Authority::Denied => {}
        }
    }
    if let Some(error) = uncertainty {
        return unverified(RELEASE_WRITE_RULE.name, error);
    }
    finding(
        Status::Pass,
        RELEASE_WRITE_RULE,
        format!("active release {} and activation parents are immutable to {}", target.display(), runtime.name),
    )
}

fn active_release_target(evidence: &ReleaseEvidence) -> Option<&PathBuf> {
    match &evidence.current {
        CurrentState::Active(target) => Some(target),
        CurrentState::Missing | CurrentState::Broken | CurrentState::NotSymlink => None,
    }
}

fn release_current_finding(evidence: &ReleaseEvidence) -> Finding {
    match &evidence.current {
        CurrentState::Missing => finding(
            Status::NotApplicable,
            Rule { name: "Release activation is root-controlled", remediation: "Deploy once, then rerun doctor." },
            format!("site {} has no current entry before its first activation", evidence.site),
        ),
        CurrentState::Broken => finding(
            Status::Fail,
            Rule {
                name: "Release activation is root-controlled",
                remediation: "Activate a valid sealed release through BonesRemote.",
            },
            format!("{} is a broken symlink", evidence.filesystem.requested.display()),
        ),
        CurrentState::NotSymlink => finding(
            Status::Fail,
            Rule {
                name: "Release activation is root-controlled",
                remediation: "Restore current through a BonesRemote activation.",
            },
            format!("{} exists but is not a symlink", evidence.filesystem.requested.display()),
        ),
        CurrentState::Active(_) => unreachable!("active release was handled before evaluating its state"),
    }
}
