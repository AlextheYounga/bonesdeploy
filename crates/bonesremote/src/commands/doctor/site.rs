use std::fs;
use std::path::Path;
use std::process::Command;

use bonesdeploy_core::{config, paths};

use crate::git::{branch_exists, repository_has_refs};
use crate::inspection::accounts;
use crate::release::lifecycle::build::validate_build_cache;
use crate::release::lifecycle::load_site_config;
use crate::runtime::docker;

use super::services;

pub(crate) fn check(site: &str, issues: &mut Vec<String>, pending: &mut Vec<String>) {
    if let Err(error) = config::validate_site_name(site) {
        issues.push(format!("Invalid site name for doctor: {error}"));
        return;
    }

    let cfg = match load_site_config(site) {
        Ok(cfg) => cfg,
        Err(error) => {
            issues.push(format!("deployed site configuration is invalid: {error}"));
            return;
        }
    };
    if !paths::bonesremote_site_root(site).is_dir() {
        issues.push(format!("control-plane site state is missing: {}", paths::bonesremote_site_root(site).display()));
        return;
    }

    let project_root = &cfg.project_root;
    let shared_root = Path::new(project_root).join(paths::SHARED_DIR);
    let releases_root = Path::new(project_root).join(paths::RELEASES_DIR);
    let runtime_user = config::runtime_user_for(&cfg.project_name);
    let runtime_group = config::runtime_group_for(&cfg.project_name);
    let build_user = config::build_user_for(&cfg.project_name);

    check_repo_exists(&cfg.repo_path, issues);
    check_branch_ref(&cfg.repo_path, &cfg.branch, issues, pending);

    match fs::read_to_string(paths::ETC_PASSWD) {
        Ok(passwd) => {
            check_runtime_identity(&runtime_user, &runtime_group, &passwd, issues);
            check_build_user(&build_user, &passwd, issues);
        }
        Err(error) => {
            issues.push(format!("could not read {} to validate user accounts ({error})", paths::ETC_PASSWD));
        }
    }

    check_site_layout(&shared_root, &releases_root, issues);
    services::check_target(&cfg, issues);
    if cfg.runtime.backend == config::RuntimeBackend::Docker {
        check_docker_runtime(&cfg, issues);
    }
}

fn check_docker_runtime(cfg: &config::Bones, issues: &mut Vec<String>) {
    match Command::new("docker").arg("info").output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            issues.push(format!("Docker daemon is unavailable: {}", String::from_utf8_lossy(&output.stderr).trim()));
        }
        Err(error) => issues.push(format!("Docker is unavailable: {error}")),
    }

    let image = match docker::command::image_name(&cfg.project_name) {
        Ok(image) => image,
        Err(error) => {
            issues.push(format!("Docker runtime image name is invalid: {error}"));
            return;
        }
    };
    match Command::new("docker").args(["image", "inspect", &image]).status() {
        Ok(status) if status.success() => {}
        Ok(_) => issues.push(format!("Docker runtime image is missing: {image}")),
        Err(error) => issues.push(format!("could not inspect Docker runtime image {image}: {error}")),
    }

    let socket_dir = Path::new("/run").join(&cfg.project_name);
    if !socket_dir.is_dir() {
        issues.push(format!("Docker runtime socket directory is missing: {}", socket_dir.display()));
    }
}

fn check_build_user(build_user: &str, passwd: &str, issues: &mut Vec<String>) {
    if !accounts::account_exists(passwd, build_user) {
        issues.push(format!("build user does not exist: {build_user}"));
        return;
    }

    let expected_home = paths::bonesdeploy_user_home(build_user);
    if accounts::account_home(passwd, build_user).is_none_or(|home| Path::new(home) != expected_home) {
        issues.push(format!("build user home must be {}: {build_user}", expected_home.display()));
    }

    let Some((uid, gid)) = accounts::account_identity(passwd, build_user) else {
        issues.push(format!("build user has invalid passwd identity: {build_user}"));
        return;
    };
    if let Err(error) = validate_build_cache(&paths::bonesdeploy_user_cache(build_user), uid, gid) {
        issues.push(error.to_string());
    }
}

fn check_repo_exists(repo_path: &str, issues: &mut Vec<String>) {
    let repo_path = Path::new(repo_path);
    if !repo_path.is_dir() {
        issues.push(format!("bare repo is missing: {}", repo_path.display()));
    }
}

fn check_branch_ref(repo_path: &str, branch: &str, issues: &mut Vec<String>, pending: &mut Vec<String>) {
    if branch.is_empty() {
        return;
    }
    match repository_has_refs(Path::new(repo_path)) {
        Ok(true) => {}
        Ok(false) => {
            pending.push(format!(
                "deploy branch '{branch}' has not been pushed yet. Run 'git push <remote> {branch}' before the first deploy."
            ));
            return;
        }
        Err(error) => {
            issues.push(format!("could not inspect branches in {repo_path}: {error}"));
            return;
        }
    }
    match branch_exists(Path::new(repo_path), branch) {
        Ok(true) => {}
        Ok(false) => issues.push(format!(
            "deploy branch '{branch}' has not been pushed to {repo_path}. Run 'git push <remote> {branch}' first."
        )),
        Err(error) => issues.push(format!("could not check branch '{branch}': {error}")),
    }
}

fn check_runtime_identity(runtime_user: &str, runtime_group: &str, passwd: &str, issues: &mut Vec<String>) {
    if !accounts::account_exists(passwd, runtime_user) {
        issues.push(format!("runtime user does not exist: {runtime_user}"));
    }

    let groupfile = match fs::read_to_string(paths::ETC_GROUP) {
        Ok(groupfile) => groupfile,
        Err(error) => {
            issues.push(format!("could not read {} to validate runtime group ({error})", paths::ETC_GROUP));
            return;
        }
    };
    let Some(members) = accounts::group_members(&groupfile, runtime_group) else {
        issues.push(format!("runtime group does not exist: {runtime_group}"));
        return;
    };
    if members.iter().any(|member| member == paths::DEPLOY_USER) {
        issues.push(format!("{} must not be a member of runtime group {}", paths::DEPLOY_USER, runtime_group));
    }
}

fn check_site_layout(shared_root: &Path, releases_root: &Path, issues: &mut Vec<String>) {
    if !shared_root.is_dir() {
        issues.push(format!("shared root is missing: {}", shared_root.display()));
    }

    if !releases_root.is_dir() {
        issues.push(format!("releases root is missing: {}", releases_root.display()));
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs, process, process::Command};

    #[test]
    fn empty_bare_repo_is_pending_before_first_push() {
        let root = env::temp_dir().join(format!("bonesremote-doctor-empty-repo-{}", process::id()));
        let _ = fs::remove_dir_all(&root);
        let output = Command::new("git").args(["init", "--bare", root.to_str().unwrap_or_default()]).output();
        assert!(output.is_ok_and(|output| output.status.success()));

        let mut issues = Vec::new();
        let mut pending = Vec::new();
        super::check_branch_ref(root.to_str().unwrap_or_default(), "master", &mut issues, &mut pending);

        let _ = fs::remove_dir_all(root);
        assert!(issues.is_empty());
        assert_eq!(pending.len(), 1);
    }
}
