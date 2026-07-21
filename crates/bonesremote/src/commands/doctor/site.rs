use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use shared::{config, paths};

use crate::release::script_runner::validate_build_cache;

pub(crate) fn check(site: &str, issues: &mut Vec<String>, pending: &mut Vec<String>) {
    if let Err(error) = validate_site_name(site) {
        issues.push(format!("Invalid site name for doctor: {error}"));
        return;
    }

    let Some(cfg) = check_site_state(site, issues) else {
        return;
    };

    let project_root = &cfg.project_root;
    let shared_root = Path::new(project_root).join(paths::SHARED_DIR);
    let releases_root = Path::new(project_root).join(paths::RELEASES_DIR);
    let runtime_user = config::runtime_user_for(&cfg.project_name);
    let runtime_group = config::runtime_group_for(&cfg.project_name);
    let build_user = config::build_user_for(&cfg.project_name);

    check_repo_exists(&cfg.repo_path, issues);
    check_branch_ref(&cfg.repo_path, &cfg.branch, issues, pending);
    check_thin_hook(&cfg.repo_path, issues);
    check_runtime_identity(&runtime_user, &runtime_group, issues);
    check_build_user(&build_user, issues);
    check_site_layout(&shared_root, &releases_root, issues);
    check_site_target_exists(&cfg.project_name, issues);
}

fn check_build_user(build_user: &str, issues: &mut Vec<String>) {
    let passwd = match fs::read_to_string(paths::ETC_PASSWD) {
        Ok(passwd) => passwd,
        Err(error) => {
            issues.push(format!("could not read {} to validate build user ({error})", paths::ETC_PASSWD));
            return;
        }
    };
    if !account_exists(&passwd, build_user) {
        issues.push(format!("build user does not exist: {build_user}"));
        return;
    }

    let expected_home = paths::bonesdeploy_user_home(build_user);
    if account_home(&passwd, build_user).is_none_or(|home| Path::new(home) != expected_home) {
        issues.push(format!("build user home must be {}: {build_user}", expected_home.display()));
    }

    let Some((uid, gid)) = account_identity(&passwd, build_user) else {
        issues.push(format!("build user has invalid passwd identity: {build_user}"));
        return;
    };
    if let Err(error) = validate_build_cache(&paths::bonesdeploy_user_cache(build_user), uid, gid) {
        issues.push(error.to_string());
    }
}

fn check_site_state(site: &str, issues: &mut Vec<String>) -> Option<config::Bones> {
    let site_root = paths::bonesremote_site_root(site);
    if !site_root.is_dir() {
        issues.push(format!("control-plane site state is missing: {}", site_root.display()));
        return None;
    }

    let bones_path = site_root.join(paths::BONES_TOML);
    let cfg = match config::load(&bones_path) {
        Ok(cfg) => cfg,
        Err(error) => {
            issues.push(format!("control-plane bones.toml is invalid: {error}"));
            return None;
        }
    };

    if cfg.project_name != site {
        issues.push(format!("control-plane bones.toml belongs to '{}', expected '{}'", cfg.project_name, site));
        return None;
    }

    Some(cfg)
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
    let ref_name = format!("refs/heads/{branch}");
    let refs = match Command::new("git").args(["--git-dir", repo_path, "for-each-ref", "--format=%(refname)"]).output()
    {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            issues.push(format!(
                "could not inspect branches in {repo_path}: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
            return;
        }
        Err(error) => {
            issues.push(format!("could not run git while inspecting {repo_path}: {error}"));
            return;
        }
    };
    if refs.stdout.is_empty() {
        pending.push(format!(
            "deploy branch '{branch}' has not been pushed yet. Run 'git push <remote> {branch}' before the first deploy."
        ));
        return;
    }
    let branch_output = Command::new("git").args(["--git-dir", repo_path, "rev-parse", "--verify", &ref_name]).output();
    match branch_output {
        Ok(output) if output.status.success() => {}
        Ok(_) => issues.push(format!(
            "deploy branch '{branch}' has not been pushed to {repo_path}. Run 'git push <remote> {branch}' first."
        )),
        Err(error) => issues.push(format!("could not run git while checking branch '{branch}': {error}")),
    }
}

fn check_thin_hook(repo_path: &str, issues: &mut Vec<String>) {
    let hook_path = Path::new(repo_path).join("hooks").join("post-receive");
    let hook = fs::read_to_string(&hook_path);

    match hook {
        Ok(contents) if hook_uses_thin_trigger(&contents) => {}
        Ok(_) => issues.push(format!("thin post-receive hook is missing or stale: {}", hook_path.display())),
        Err(error) => issues.push(format!("thin post-receive hook is missing: {} ({error})", hook_path.display())),
    }
}

fn check_runtime_identity(runtime_user: &str, runtime_group: &str, issues: &mut Vec<String>) {
    let passwd = match fs::read_to_string(paths::ETC_PASSWD) {
        Ok(passwd) => passwd,
        Err(error) => {
            issues.push(format!("could not read {} to validate runtime user ({error})", paths::ETC_PASSWD));
            return;
        }
    };
    if !account_exists(&passwd, runtime_user) {
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

fn validate_site_name(site: &str) -> Result<()> {
    config::validate_project_name(site).map_err(|error| anyhow::anyhow!("Invalid site name: {error}"))
}

fn check_site_target_exists(site: &str, issues: &mut Vec<String>) {
    let target_name = paths::site_target_name(site);
    let output =
        Command::new("systemctl").args(["show", "--property=LoadState", "--value", "--", &target_name]).output();

    match output {
        Ok(output) if output.status.success() && service_exists(&String::from_utf8_lossy(&output.stdout)) => {
            check_target_membership(&target_name, issues);
        }
        Ok(_) => issues.push(format!("site target is missing: {target_name}")),
        Err(error) => issues.push(format!("could not inspect site target {target_name} ({error})")),
    }
}

fn check_target_membership(target: &str, issues: &mut Vec<String>) {
    let output =
        Command::new("systemctl").args(["show", "--property=Requires", "--value", "--no-pager", "--", target]).output();
    let services = match output {
        Ok(output) if output.status.success() => required_services(&String::from_utf8_lossy(&output.stdout)),
        Ok(_) => {
            issues.push(format!("could not inspect required services for site target: {target}"));
            return;
        }
        Err(error) => {
            issues.push(format!("could not inspect required services for site target {target} ({error})"));
            return;
        }
    };
    if services.is_empty() {
        issues.push(format!("site target has no registered services: {target}"));
        return;
    }

    for service in services {
        check_required_service_active(target, &service, issues);
    }
}

fn required_services(output: &str) -> Vec<String> {
    output.split_whitespace().filter(|name| name.ends_with(paths::SYSTEMD_SERVICE_SUFFIX)).map(str::to_owned).collect()
}

fn check_required_service_active(target: &str, service: &str, issues: &mut Vec<String>) {
    match Command::new("systemctl").args(["is-active", "--quiet", "--", service]).status() {
        Ok(status) if status.success() => {}
        Ok(_) => issues.push(inactive_service_issue(target, service)),
        Err(error) => issues.push(format!("could not inspect required service {service} for {target} ({error})")),
    }
}

fn inactive_service_issue(target: &str, service: &str) -> String {
    format!("required service {service} for site target {target} is not active")
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

fn hook_uses_thin_trigger(contents: &str) -> bool {
    contents.contains("sudo bonesremote hook post-receive --site")
}

fn service_exists(load_state: &str) -> bool {
    load_state.trim() == "loaded"
}

#[cfg(test)]
mod tests {
    use std::{env, fs, process, process::Command};

    use super::{
        account_exists, account_home, account_identity, group_members, hook_uses_thin_trigger, required_services,
        service_exists,
    };

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
    fn hook_uses_thin_trigger_accepts_bonesremote_post_receive_delegate() {
        assert!(hook_uses_thin_trigger("exec sudo bonesremote hook post-receive --site \"$SITE\"\n"));
    }

    #[test]
    fn service_exists_accepts_loaded_unit() {
        assert!(service_exists("loaded\n"));
        assert!(!service_exists("not-found\n"));
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

    #[test]
    fn target_without_required_services_is_rejected() {
        assert!(required_services("").is_empty());
        assert!(required_services("nexttest.target").is_empty());
    }
}
