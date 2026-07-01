use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};
use shared::{config, paths};

pub(crate) fn check(site: &str, issues: &mut Vec<String>) {
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
    let current_path = Path::new(project_root).join(paths::CURRENT_LINK);
    let runtime_user = config::runtime_user_for(&cfg.project_name);
    let runtime_group = config::runtime_group_for(&cfg.project_name);

    check_repo_exists(&cfg.repo_path, issues);
    check_branch_ref(&cfg.repo_path, &cfg.branch, issues);
    check_thin_hook(&cfg.repo_path, issues);
    check_runtime_identity(&runtime_user, &runtime_group, issues);
    check_site_layout(&shared_root, &releases_root, &current_path, issues);
    check_service_exists(&cfg.project_name, issues);
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

    if let Err(error) = config::load_runtime(&site_root) {
        issues.push(format!("control-plane runtime.toml is invalid: {error}"));
    }

    Some(cfg)
}

fn check_repo_exists(repo_path: &str, issues: &mut Vec<String>) {
    let repo_path = Path::new(repo_path);
    if !repo_path.is_dir() {
        issues.push(format!("bare repo is missing: {}", repo_path.display()));
    }
}

fn check_branch_ref(repo_path: &str, branch: &str, issues: &mut Vec<String>) {
    if branch.is_empty() {
        return;
    }
    let ref_name = format!("refs/heads/{branch}");
    let ok = Command::new("git")
        .args(["--git-dir", repo_path, "rev-parse", "--verify", &ref_name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !ok {
        issues.push(format!(
            "deploy branch '{branch}' has not been pushed to {}\n  {}",
            repo_path, "Run 'git push <remote> {branch}' first.",
        ));
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
    if runtime_user == paths::DEPLOY_USER {
        issues.push(format!("runtime user must not be {}", paths::DEPLOY_USER));
    }

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

fn check_site_layout(shared_root: &Path, releases_root: &Path, current_path: &Path, issues: &mut Vec<String>) {
    if !shared_root.is_dir() {
        issues.push(format!("shared root is missing: {}", shared_root.display()));
    }

    if !releases_root.is_dir() {
        issues.push(format!("releases root is missing: {}", releases_root.display()));
    }

    match current_path.parent() {
        Some(parent) if parent.is_dir() => {}
        Some(parent) => issues.push(format!("current parent is missing: {}", parent.display())),
        None => issues.push(format!("current path has no parent: {}", current_path.display())),
    }
}

fn validate_site_name(site: &str) -> Result<()> {
    if site.is_empty() {
        bail!("Site name cannot be empty");
    }

    if site.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        return Ok(());
    }

    bail!("Invalid site name: {site}")
}

fn check_service_exists(site: &str, issues: &mut Vec<String>) {
    let service_name = paths::nginx_service_name(site);
    let output = Command::new("systemctl").args(["show", "--property=LoadState", "--value", &service_name]).output();

    match output {
        Ok(output) if output.status.success() && service_exists(&String::from_utf8_lossy(&output.stdout)) => {}
        Ok(_) => issues.push(format!("service is missing: {service_name}")),
        Err(error) => issues.push(format!("could not inspect service {service_name} ({error})")),
    }
}

fn account_exists(passwd: &str, account: &str) -> bool {
    passwd.lines().any(|line| line.starts_with(&format!("{account}:")))
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
    use super::{account_exists, group_members, hook_uses_thin_trigger, service_exists};

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
    fn group_members_reads_group_member_list() {
        assert_eq!(
            group_members("demo:x:1000:git,www-data\n", "demo"),
            Some(vec!["git".to_string(), "www-data".to_string()])
        );
        assert_eq!(group_members("demo:x:1000:\n", "demo"), Some(Vec::new()));
        assert_eq!(group_members("demo:x:1000:\n", "nope"), None);
    }
}
