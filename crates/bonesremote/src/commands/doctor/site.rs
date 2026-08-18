use std::fs;
use std::path::Path;
use std::process::Command;

use bonesdeploy_core::{config, paths};

use crate::release::lifecycle::build::validate_build_cache;
use crate::release::lifecycle::checkout::{branch_exists, repository_has_refs};
use crate::runtime::docker;

use super::services;

pub(crate) fn check(site: &str, issues: &mut Vec<String>, pending: &mut Vec<String>) {
    if let Err(error) = config::validate_site_name(site) {
        issues.push(format!("Invalid site name for doctor: {error}"));
        return;
    }

    let cfg = config::Bones::for_site(site);
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
    if !account_exists(passwd, build_user) {
        issues.push(format!("build user does not exist: {build_user}"));
        return;
    }

    let expected_home = paths::bonesdeploy_user_home(build_user);
    if account_home(passwd, build_user).is_none_or(|home| Path::new(home) != expected_home) {
        issues.push(format!("build user home must be {}: {build_user}", expected_home.display()));
    }

    let Some((uid, gid)) = account_identity(passwd, build_user) else {
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
    if !account_exists(passwd, runtime_user) {
        issues.push(format!("runtime user does not exist: {runtime_user}"));
    }

    let groupfile = match fs::read_to_string(paths::ETC_GROUP) {
        Ok(groupfile) => groupfile,
        Err(error) => {
            issues.push(format!("could not read {} to validate runtime group ({error})", paths::ETC_GROUP));
            return;
        }
    };
    let Some(members) = group_members(&groupfile, runtime_group) else {
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

fn account_exists(passwd: &str, account: &str) -> bool {
    passwd.lines().any(|line| line.starts_with(&format!("{account}:")))
}

fn account_home<'a>(passwd: &'a str, account: &str) -> Option<&'a str> {
    account_field(passwd, account, 5)
}

fn account_identity(passwd: &str, account: &str) -> Option<(u32, u32)> {
    let uid = account_field(passwd, account, 2)?.parse().ok()?;
    let gid = account_field(passwd, account, 3)?.parse().ok()?;
    Some((uid, gid))
}

fn account_field<'a>(passwd: &'a str, account: &str, index: usize) -> Option<&'a str> {
    passwd.lines().find(|line| line.starts_with(&format!("{account}:")))?.split(':').nth(index)
}

fn group_members(groupfile: &str, group: &str) -> Option<Vec<String>> {
    let line = groupfile.lines().find(|line| line.starts_with(&format!("{group}:")))?;
    let fields: Vec<&str> = line.split(':').collect();
    let members = fields.get(3).copied().unwrap_or_default();
    if members.is_empty() {
        return Some(Vec::new());
    }
    Some(members.split(',').map(str::to_string).collect())
}

#[cfg(test)]
mod tests {
    use std::{env, fs, process, process::Command};

    use super::{account_exists, account_home, account_identity, group_members};

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

    #[test]
    fn account_exists_matches_passwd_entries() {
        assert!(account_exists("demo:x:1000:1000::/srv:/usr/sbin/nologin\n", "demo"));
        assert!(!account_exists("demo:x:1000:1000::/srv:/usr/sbin/nologin\n", "git"));
    }

    #[test]
    fn build_user_home_is_parsed() {
        let passwd = "demo-build:x:1002:1002::/var/lib/bonesdeploy/users/demo-build:/usr/sbin/nologin\n";
        assert_eq!(account_home(passwd, "demo-build"), Some("/var/lib/bonesdeploy/users/demo-build"));
        assert_eq!(account_identity(passwd, "demo-build"), Some((1002, 1002)));
    }

    #[test]
    fn group_members_reads_group_member_list() {
        assert_eq!(
            group_members("demo:x:1000:git,www-data\n", "demo"),
            Some(vec!["git".to_string(), "www-data".to_string()])
        );
        assert_eq!(group_members("demo:x:1000:\n", "demo"), Some(Vec::new()));
        assert_eq!(group_members("demo:x:1000:\n", "nope"), None);
    }
}
