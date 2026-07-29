use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;

use shared::paths;

use super::fs::{account_can_write, find_runtime_writable, has_login_shell, try_canonicalize, writable_in_path_chain};
use super::types::{Account, Finding, Site, Status, fail, pass, unverified};

pub(super) fn evaluate_identities(sites: &[Site], deploy: &Account) -> Finding {
    let mut runtime_user_ids = BTreeSet::new();
    let mut runtime_primary_groups = BTreeSet::new();
    for site in sites {
        if site.runtime.uid == site.build.uid || site.runtime.gid == site.build.gid {
            return fail(
                "Site identity isolation",
                format!("{} runtime and build accounts share an identity", site.name),
            );
        }
        if !runtime_user_ids.insert(site.runtime.uid) || !runtime_primary_groups.insert(site.runtime.gid) {
            return fail(
                "Site identity isolation",
                format!("{} reuses another site's runtime UID or primary GID", site.name),
            );
        }
        if has_login_shell(&site.runtime.shell) {
            return fail(
                "Site identity isolation",
                format!("runtime user {} has login shell {}", site.runtime.name, site.runtime.shell),
            );
        }
        if deploy.groups.contains(&site.runtime.gid) {
            return fail(
                "Site identity isolation",
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
                return fail(
                    "Site identity isolation",
                    format!("{} account belongs to runtime group for {}", site.name, other.name),
                );
            }
        }
    }
    pass("Site identity isolation", format!("{} imported site(s) have distinct runtime/build identities", sites.len()))
}

pub(super) fn evaluate_runtime_sudo(sites: &[Site]) -> Vec<Finding> {
    sites
        .iter()
        .map(|site| match Command::new("sudo").args(["-n", "-u", &site.runtime.name, "sudo", "-n", "-l"]).output() {
            Ok(output) if output.status.success() => fail(
                "Runtime sudo authority is absent",
                format!("runtime user {} has a sudo policy", site.runtime.name),
            ),
            Ok(_) => pass(
                "Runtime sudo authority is absent",
                format!("runtime user {} has no sudo policy", site.runtime.name),
            ),
            Err(error) => unverified(
                "Runtime sudo authority is absent",
                format!("cannot inspect sudo policy for {}: {error}", site.runtime.name),
            ),
        })
        .collect()
}

pub(super) fn evaluate_privileged_paths(sites: &[Site], deploy: &Account) -> Vec<Finding> {
    let protected = [
        paths::bonesremote_config_root(),
        PathBuf::from(paths::ETC_SYSTEMD_SYSTEM),
        PathBuf::from(paths::ETC_SUDOERS_D),
        PathBuf::from(paths::ETC_NGINX_SITES_AVAILABLE),
        PathBuf::from(paths::ETC_NGINX_SITES_ENABLED),
        PathBuf::from(paths::ETC_APPARMOR_D),
        PathBuf::from(paths::SUDOERS_PATH),
        paths::bonesremote_global_link(),
    ];
    let mut untrusted: Vec<_> = sites.iter().flat_map(|site| [&site.runtime, &site.build]).collect();
    untrusted.push(deploy);
    protected
        .iter()
        .map(|path| match writable_in_path_chain(path, &untrusted) {
            Ok(None) => pass(
                "Privileged configuration is root-controlled",
                format!("no untrusted account can modify {} or its parents", path.display()),
            ),
            Ok(Some((account, writable))) => fail(
                "Privileged configuration is root-controlled",
                format!("{} can write {} in the path to {}", account.name, writable.display(), path.display()),
            ),
            Err(error) => unverified("Privileged configuration is root-controlled", error),
        })
        .collect()
}

pub(super) fn evaluate_active_release(site: &Site) -> Finding {
    let current = site.project_root.join(paths::CURRENT_LINK);
    let Some(parent) = current.parent() else {
        return unverified("Release activation is root-controlled", format!("{} has no parent", current.display()));
    };
    match account_can_write(parent, &site.runtime) {
        Ok(true) => {
            return fail(
                "Release activation is root-controlled",
                format!("runtime user {} can write {} and replace current", site.runtime.name, parent.display()),
            );
        }
        Ok(false) => {}
        Err(error) => return unverified("Release activation is root-controlled", error),
    }
    let target = match try_canonicalize(&current) {
        Ok(Some(target)) => target,
        Ok(None) => {
            return Finding {
                status: Status::NotApplicable,
                rule: "Release activation is root-controlled",
                principle: "Write access to a directory permits replacement of its entries.",
                expected: "The runtime user cannot modify current or an active release.".to_string(),
                observed: "No active release exists yet.".to_string(),
                risk: "No active release is exposed.".to_string(),
                remediation: "Deploy once, then rerun doctor.".to_string(),
            };
        }
        Err(error) => return unverified("Release activation is root-controlled", error),
    };
    match find_runtime_writable(&target, &site.runtime) {
        Ok(None) => pass(
            "Release activation is root-controlled",
            format!("{} and its active release are not writable by {}", current.display(), site.runtime.name),
        ),
        Ok(Some(path)) => fail(
            "Release activation is root-controlled",
            format!("runtime user {} can modify active release path {}", site.runtime.name, path.display()),
        ),
        Err(error) => unverified("Release activation is root-controlled", error),
    }
}
