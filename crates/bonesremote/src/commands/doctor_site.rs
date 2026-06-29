use std::fs;
use std::path::Path;
use std::process::Command;

use shared::{config, paths, registry};

pub(crate) fn check(site: &str, issues: &mut Vec<String>) {
    if let Err(error) = registry::validate_site_name(site) {
        issues.push(format!("Invalid site name for doctor: {error}"));
        return;
    }

    let Some(cfg) = check_site_state(site, issues) else {
        return;
    };

    check_repo_exists(&cfg, issues);
    check_thin_hook(&cfg, issues);
    check_runtime_identity(&cfg, issues);
    check_site_layout(&cfg, issues);
    check_service_exists(&cfg.site, issues);
}

fn check_site_state(site: &str, issues: &mut Vec<String>) -> Option<registry::Registry> {
    let site_root = paths::bonesremote_site_root(site);
    if !site_root.is_dir() {
        issues.push(format!("control-plane site state is missing: {}", site_root.display()));
        return None;
    }

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = match registry::load(&registry_path) {
        Ok(cfg) => cfg,
        Err(error) => {
            issues.push(format!("control-plane registry is invalid: {error}"));
            return None;
        }
    };

    if let Err(error) = config::load(&site_root.join(paths::BONES_TOML)) {
        issues.push(format!("control-plane bones.toml is invalid: {error}"));
    }
    if let Err(error) = config::load_runtime(&site_root) {
        issues.push(format!("control-plane runtime.toml is invalid: {error}"));
    }

    Some(cfg)
}

fn check_repo_exists(cfg: &registry::Registry, issues: &mut Vec<String>) {
    let repo_path = Path::new(&cfg.repo_path);
    if !repo_path.is_dir() {
        issues.push(format!("bare repo is missing: {}", repo_path.display()));
    }
}

fn check_thin_hook(cfg: &registry::Registry, issues: &mut Vec<String>) {
    let hook_path = Path::new(&cfg.repo_path).join("hooks").join("post-receive");
    let hook = fs::read_to_string(&hook_path);

    match hook {
        Ok(contents) if hook_uses_thin_trigger(&contents) => {}
        Ok(_) => issues.push(format!("thin post-receive hook is missing or stale: {}", hook_path.display())),
        Err(error) => issues.push(format!("thin post-receive hook is missing: {} ({error})", hook_path.display())),
    }
}

fn check_runtime_identity(cfg: &registry::Registry, issues: &mut Vec<String>) {
    if cfg.runtime_user == paths::DEPLOY_USER {
        issues.push(format!("runtime user must not be {}", paths::DEPLOY_USER));
    }

    let passwd = match fs::read_to_string(paths::ETC_PASSWD) {
        Ok(passwd) => passwd,
        Err(error) => {
            issues.push(format!("could not read {} to validate runtime user ({error})", paths::ETC_PASSWD));
            return;
        }
    };
    if !account_exists(&passwd, &cfg.runtime_user) {
        issues.push(format!("runtime user does not exist: {}", cfg.runtime_user));
    }

    let groupfile = match fs::read_to_string(paths::ETC_GROUP) {
        Ok(groupfile) => groupfile,
        Err(error) => {
            issues.push(format!("could not read {} to validate runtime group ({error})", paths::ETC_GROUP));
            return;
        }
    };
    let Some(members) = group_members(&groupfile, &cfg.runtime_group) else {
        issues.push(format!("runtime group does not exist: {}", cfg.runtime_group));
        return;
    };
    if members.iter().any(|member| member == paths::DEPLOY_USER) {
        issues.push(format!("{} must not be a member of runtime group {}", paths::DEPLOY_USER, cfg.runtime_group));
    }
}

fn check_site_layout(cfg: &registry::Registry, issues: &mut Vec<String>) {
    let shared_root = Path::new(&cfg.shared_root);
    if !shared_root.is_dir() {
        issues.push(format!("shared root is missing: {}", shared_root.display()));
    }

    let releases_root = Path::new(&cfg.releases_root);
    if !releases_root.is_dir() {
        issues.push(format!("releases root is missing: {}", releases_root.display()));
    }

    match Path::new(&cfg.current_path).parent() {
        Some(parent) if parent.is_dir() => {}
        Some(parent) => issues.push(format!("current parent is missing: {}", parent.display())),
        None => issues.push(format!("current path has no parent: {}", cfg.current_path)),
    }
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
