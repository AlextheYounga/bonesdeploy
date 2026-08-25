use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::Command;

use bonesdeploy_core::paths;

use crate::inspection::systemd;

const APT_AUTO_UPGRADES: &str = "/etc/apt/apt.conf.d/20auto-upgrades";
const APT_UNATTENDED_UPGRADES: &str = "/etc/apt/apt.conf.d/50unattended-upgrades";

pub(super) fn check(issues: &mut Vec<String>) {
    check_root_directory(paths::BONESREMOTE_CONFIG_DIR, issues);
    check_root_directory(&paths::bonesremote_sites_root().display().to_string(), issues);
    check_root_directory(paths::IMAGE_STORE_GRAPH_ROOT, issues);
    check_root_directory(paths::IMAGE_STORE_RUN_ROOT, issues);
    check_root_executable(&paths::bonesremote_global_link(), issues);
    check_root_file(paths::SUDOERS_PATH, issues);
    check_sudoers_syntax(issues);
    check_root_file(paths::IMAGE_STORE_STORAGE_CONF, issues);
    check_seeded_image(issues);
    check_ufw(issues);
    check_active_service("fail2ban", issues);
    check_root_file(APT_AUTO_UPGRADES, issues);
    check_root_file(APT_UNATTENDED_UPGRADES, issues);
}

fn check_root_directory(path: &str, issues: &mut Vec<String>) {
    if let Some(issue) = root_owned_path_issue(Path::new(path), true, false) {
        issues.push(format!("server baseline directory {path}: {issue}"));
    }
}

fn check_root_file(path: &str, issues: &mut Vec<String>) {
    if let Some(issue) = root_owned_path_issue(Path::new(path), false, false) {
        issues.push(format!("server baseline file {path}: {issue}"));
    }
}

fn check_root_executable(path: &Path, issues: &mut Vec<String>) {
    if let Some(issue) = root_owned_path_issue(path, false, true) {
        issues.push(format!("server baseline BonesRemote binary {}: {issue}", path.display()));
    }
}

fn root_owned_path_issue(path: &Path, directory: bool, executable: bool) -> Option<String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => return Some(format!("is missing or inaccessible ({error})")),
    };
    let kind_matches = if directory { metadata.file_type().is_dir() } else { metadata.file_type().is_file() };
    if !kind_matches || metadata.file_type().is_symlink() {
        return Some(if directory { "must be a directory".to_string() } else { "must be a regular file".to_string() });
    }
    if metadata.uid() != 0 || metadata.gid() != 0 {
        return Some("must be owned by root:root".to_string());
    }
    if metadata.mode() & 0o022 != 0 {
        return Some("must not be writable by group or other users".to_string());
    }
    if executable && metadata.mode() & 0o111 == 0 {
        return Some("must be executable".to_string());
    }
    None
}

fn check_sudoers_syntax(issues: &mut Vec<String>) {
    match Command::new("visudo").args(["-c", "-f", paths::SUDOERS_PATH]).status() {
        Ok(status) if status.success() => {}
        Ok(status) => issues.push(format!("server baseline sudoers policy is invalid (visudo exited {status})")),
        Err(error) => issues.push(format!("could not validate server baseline sudoers policy: {error}")),
    }
}

fn check_seeded_image(issues: &mut Vec<String>) {
    match Command::new("podman")
        .env("CONTAINERS_STORAGE_CONF", paths::IMAGE_STORE_STORAGE_CONF)
        .args(["image", "exists", paths::IMAGE_STORE_BASE_IMAGE])
        .status()
    {
        Ok(status) if status.success() => {}
        Ok(_) => {
            issues.push(format!("server baseline shared image store is missing {}", paths::IMAGE_STORE_BASE_IMAGE));
        }
        Err(error) => issues.push(format!("could not inspect server baseline shared image store: {error}")),
    }
}

fn check_ufw(issues: &mut Vec<String>) {
    match Command::new("ufw").arg("status").output() {
        Ok(output) if output.status.success() && String::from_utf8_lossy(&output.stdout).contains("Status: active") => {
        }
        Ok(_) => issues.push("server baseline firewall is not active".to_string()),
        Err(error) => issues.push(format!("could not inspect server baseline firewall: {error}")),
    }
}

fn check_active_service(service: &str, issues: &mut Vec<String>) {
    match systemd::active_status(service) {
        Ok(true) => {}
        Ok(false) => issues.push(format!("server baseline security service is not active: {service}")),
        Err(error) => issues.push(format!("could not inspect server baseline security service {service}: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::io::Result;
    use std::process;

    use super::root_owned_path_issue;

    #[test]
    fn baseline_paths_reject_missing_and_non_root_owned_artifacts() -> Result<()> {
        let path = env::temp_dir().join(format!("bonesremote-baseline-{}", process::id()));
        let _ = fs::remove_file(&path);

        assert!(root_owned_path_issue(&path, false, false).is_some());

        fs::write(&path, "baseline")?;
        assert_eq!(root_owned_path_issue(&path, false, false).as_deref(), Some("must be owned by root:root"));
        fs::remove_file(path)?;
        Ok(())
    }
}
