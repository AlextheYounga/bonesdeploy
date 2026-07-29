use std::collections::BTreeSet;
use std::path::PathBuf;

use super::fs::{Authority, account_can_modify, has_login_shell};
use super::types::{
    Account, CurrentState, FileKind, Finding, PathTree, PolicyDecision, ReleaseEvidence, Rule, Site, Status,
    SudoEvidence, finding, unverified,
};

const RUNTIME_SUDO_RULE: Rule = Rule {
    name: "Runtime sudo authority is absent",
    principle: "Sudo policy is an independent privilege path.",
    expected: "The runtime identity has no sudo policy.",
    risk: "A compromised application may be able to elevate through sudo.",
    remediation: "Remove the runtime identity from sudoers and all sudo-capable groups.",
};

const PRIVILEGED_PATH_RULE: Rule = Rule {
    name: "Privileged configuration is root-controlled",
    principle: "Anything an untrusted identity can change before root interprets it is indirect root authority.",
    expected: "Protected files, directories, symlink targets, and their parents are not modifiable by runtime, build, or deploy identities.",
    risk: "An attacker could replace configuration or code later interpreted with root authority.",
    remediation: "Restore root ownership and remove effective write access from the reported path.",
};

const RELEASE_WRITE_RULE: Rule = Rule {
    name: "Release activation is root-controlled",
    principle: "Ownership or effective write access lets a compromised runtime alter executable application state.",
    expected: "The runtime identity cannot own or modify the project root, releases root, active release, or their parent chain.",
    risk: "A compromised application could alter or activate attacker-controlled code.",
    remediation: "Seal releases as root and restore root control of activation directories.",
};

const IDENTITY_RULE: Rule = Rule {
    name: "Site identity isolation",
    principle: "Two processes with the same Linux identity have the same discretionary authority.",
    expected: "Every site has unique runtime/build identities that are not shared with other sites or the deploy user.",
    risk: "A compromised application could act as another site's runtime or the deploy user.",
    remediation: "Assign distinct UIDs and primary GIDs; remove cross-site and deploy-group membership.",
};

pub(super) fn evaluate_identities(sites: &[Site], deploy: &Account) -> Finding {
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

pub(super) fn evaluate_runtime_sudo(evidence: &SudoEvidence) -> Finding {
    match &evidence.decision {
        PolicyDecision::Denied => {
            finding(Status::Pass, RUNTIME_SUDO_RULE, format!("sudo policy denied all authority for {}", evidence.user))
        }
        PolicyDecision::Allowed => {
            finding(Status::Fail, RUNTIME_SUDO_RULE, format!("sudo listed permitted commands for {}", evidence.user))
        }
        PolicyDecision::Unverified(error) => unverified(
            RUNTIME_SUDO_RULE.name,
            format!("could not determine sudo authority for {}: {error}", evidence.user),
        ),
    }
}

pub(super) fn evaluate_deploy_sudo(evidence: &SudoEvidence, should_be_allowed: bool) -> Finding {
    let rule = Rule {
        name: "Deploy sudo authority is exact",
        principle: "A privileged mediator grants exactly the authority its sudo policy permits.",
        expected: if should_be_allowed {
            "The approved command is allowed."
        } else {
            "The representative forbidden command is denied."
        },
        risk: "This sampled policy boundary matches the explicit allowlist.",
        remediation: "None.",
    };
    let command = evidence.command.join(" ");
    match (&evidence.decision, should_be_allowed) {
        (PolicyDecision::Allowed, true) | (PolicyDecision::Denied, false) => {
            finding(Status::Pass, rule, format!("sudo policy produced the expected decision for {command}"))
        }
        (PolicyDecision::Denied, true) => finding(
            Status::Fail,
            Rule {
                expected: "Every approved BonesRemote command is allowed.",
                risk: "Deployment operations will fail or bypass the intended mediator.",
                remediation: "Reinstall the exact BonesDeploy sudoers policy.",
                ..rule
            },
            format!("sudo denied approved command {command}"),
        ),
        (PolicyDecision::Allowed, false) => finding(
            Status::Fail,
            Rule {
                expected: "Shells, arbitrary binaries, sudoedit, and unapproved BonesRemote arguments are denied.",
                risk: "Compromise of the deploy identity could become root compromise.",
                remediation: "Replace broad or wildcard sudoers entries with the exact generated allowlist.",
                ..rule
            },
            format!("sudo allowed forbidden command {command}"),
        ),
        (PolicyDecision::Unverified(error), _) => {
            unverified(rule.name, format!("could not evaluate sudo policy for {command}: {error}"))
        }
    }
}

pub(super) fn evaluate_privileged_path(tree: &PathTree, untrusted: &[&Account]) -> Finding {
    let mut uncertainty = None;
    for node in &tree.nodes {
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

pub(super) fn evaluate_active_release(evidence: &ReleaseEvidence, runtime: &Account) -> Finding {
    let Some(target) = active_release_target(evidence) else {
        return release_current_finding(evidence);
    };
    if !target.starts_with(&evidence.releases_root) || target == &evidence.releases_root {
        return finding(
            Status::Fail,
            Rule {
                name: "Release activation is root-controlled",
                principle: "Activation must select only a sealed release from the site's release store.",
                expected: "current resolves to a child of the configured releases directory.",
                risk: "Root may execute or serve state outside the release boundary.",
                remediation: "Reactivate a release contained by the site's releases directory.",
            },
            format!("current resolves to {} outside {}", target.display(), evidence.releases_root.display()),
        );
    }

    let mut uncertainty = None;
    for node in &evidence.filesystem.nodes {
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
            Rule {
                name: "Release activation is root-controlled",
                principle: "Write access to a directory permits replacement of its entries.",
                expected: "The runtime identity cannot modify current or an active release.",
                risk: "No active release is exposed.",
                remediation: "Deploy once, then rerun doctor.",
            },
            format!("site {} has no current entry before its first activation", evidence.site),
        ),
        CurrentState::Broken => finding(
            Status::Fail,
            Rule {
                name: "Release activation is root-controlled",
                principle: "Activation must resolve to a sealed release.",
                expected: "current is a valid symlink to a release inside the site's releases directory.",
                risk: "Service behavior and release integrity cannot be established.",
                remediation: "Activate a valid sealed release through BonesRemote.",
            },
            format!("{} is a broken symlink", evidence.filesystem.requested.display()),
        ),
        CurrentState::NotSymlink => finding(
            Status::Fail,
            Rule {
                name: "Release activation is root-controlled",
                principle: "Activation is controlled by atomically replacing a root-owned symlink.",
                expected: "current is a symlink managed by BonesRemote.",
                risk: "The normal atomic activation boundary has been bypassed.",
                remediation: "Restore current through a BonesRemote activation.",
            },
            format!("{} exists but is not a symlink", evidence.filesystem.requested.display()),
        ),
        CurrentState::Active(_) => unreachable!("active release was handled before evaluating its state"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::{evaluate_active_release, evaluate_deploy_sudo, evaluate_privileged_path};
    use crate::commands::doctor::security::types::{
        Account, CurrentState, FileKind, FileNode, PathTree, PolicyDecision, ReleaseEvidence, Status, SudoEvidence,
    };

    fn account() -> Account {
        Account {
            name: "atlas".to_string(),
            uid: 1001,
            gid: 1001,
            shell: "/usr/sbin/nologin".to_string(),
            groups: BTreeSet::from([1001]),
        }
    }

    fn node(path: &str, kind: FileKind, uid: u32, mode: u32) -> FileNode {
        FileNode { path: PathBuf::from(path), kind, uid, gid: uid, mode, has_acl: false }
    }

    #[test]
    fn writable_child_of_protected_directory_fails() {
        let tree = PathTree {
            requested: PathBuf::from("/etc/systemd/system"),
            nodes: vec![
                node("/", FileKind::Directory, 0, 0o755),
                node("/etc", FileKind::Directory, 0, 0o755),
                node("/etc/systemd", FileKind::Directory, 0, 0o755),
                node("/etc/systemd/system", FileKind::Directory, 0, 0o755),
                node("/etc/systemd/system/atlas.service", FileKind::File, 1001, 0o600),
            ],
        };
        let runtime = account();

        assert_eq!(evaluate_privileged_path(&tree, &[&runtime]).status, Status::Fail);
    }

    #[test]
    fn sudo_collection_error_is_unverified() {
        let evidence = SudoEvidence {
            user: "git".to_string(),
            command: vec!["/bin/sh".to_string()],
            decision: PolicyDecision::Unverified("sudoers parse error".to_string()),
        };

        assert_eq!(evaluate_deploy_sudo(&evidence, false).status, Status::Unverified);
    }

    #[test]
    fn broken_and_out_of_tree_current_targets_fail() {
        let runtime = account();
        let filesystem = PathTree { requested: PathBuf::from("/srv/sites/atlas/current"), nodes: Vec::new() };
        let broken = ReleaseEvidence {
            site: "atlas".to_string(),
            releases_root: PathBuf::from("/srv/sites/atlas/releases"),
            current: CurrentState::Broken,
            filesystem: filesystem.clone(),
        };
        let outside = ReleaseEvidence {
            site: "atlas".to_string(),
            releases_root: PathBuf::from("/srv/sites/atlas/releases"),
            current: CurrentState::Active(PathBuf::from("/tmp/attacker-release")),
            filesystem,
        };

        assert_eq!(evaluate_active_release(&broken, &runtime).status, Status::Fail);
        assert_eq!(evaluate_active_release(&outside, &runtime).status, Status::Fail);
    }
}
