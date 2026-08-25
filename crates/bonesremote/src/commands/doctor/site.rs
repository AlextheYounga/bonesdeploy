use std::fs;
use std::path::Path;
use std::process::Command;

use bonesdeploy_core::{config, paths};

use crate::git::repository_has_refs;
use crate::inspection::accounts;
use crate::release::lifecycle::build::validate_build_cache;
use crate::runtime::docker;

use super::services;

pub fn check(site: &str, issues: &mut Vec<String>, pending: &mut Vec<String>) {
    if let Err(error) = config::validate_site_name(site) {
        issues.push(format!("Invalid site name for doctor: {error}"));
        return;
    }

    if !paths::bonesremote_site_root(site).is_dir() {
        pending.push(format!("first deployment is pending for {site}"));
        return;
    }

    let project_root = paths::default_project_root_for(site);
    let shared_root = Path::new(&project_root).join(paths::SHARED_DIR);
    let releases_root = Path::new(&project_root).join(paths::RELEASES_DIR);
    let runtime_user = config::runtime_user_for(site);
    let runtime_group = config::runtime_group_for(site);
    let build_user = config::build_user_for(site);
    let repo_path = paths::default_repo_path_for(site);

    let shared_env = shared_root.join(paths::DOT_ENV);
    if !shared_env.is_file() {
        pending.push(format!(
            "shared environment is missing: {}. Run 'bonesdeploy secrets push' first.",
            shared_env.display()
        ));
    }

    check_repo_exists(&repo_path, issues);

    // Branch validation requires the repo to exist; skip if it doesn't to
    // avoid duplicate error messages.
    if Path::new(&repo_path).is_dir() {
        check_branch_ref(&repo_path, issues, pending);
    }

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

    // Services and Docker runtime checks need a config only for the target
    // name, which is always derivable from the site name.
    let cfg = config::Bones::for_site(site);
    services::check_target(&cfg, issues);

    // Docker runtime is provisioned from local config at setup time; if the
    // Docker socket directory exists, validate the image is loaded.
    let docker_socket_dir = Path::new("/run").join(site);
    if docker_socket_dir.is_dir() {
        check_docker_runtime(site, issues);
    }
}

fn check_docker_runtime(site: &str, issues: &mut Vec<String>) {
    match Command::new("docker").arg("info").output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            issues.push(format!("Docker daemon is unavailable: {}", String::from_utf8_lossy(&output.stderr).trim()));
        }
        Err(error) => issues.push(format!("Docker is unavailable: {error}")),
    }

    let image = match docker::command::image_name(site) {
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

    let socket_dir = Path::new("/run").join(site);
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

pub fn check_branch_ref(repo_path: &str, issues: &mut Vec<String>, pending: &mut Vec<String>) {
    match repository_has_refs(Path::new(repo_path)) {
        Ok(true) => {}
        Ok(false) => {
            pending.push(
                "repository has no refs yet. Run 'git push <remote> <branch>' before the first deploy.".to_string(),
            );
            return;
        }
        Err(error) => {
            issues.push(format!("could not inspect branches in {repo_path}: {error}"));
        }
    }
    // Branch existence is validated during deploy when the descriptor provides
    // the configured branch. The remote doctor only verifies the repository
    // has refs.
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
