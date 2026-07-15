use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::config;
use crate::infra::ssh;
use crate::ui::output;
use shared::paths;

pub async fn run(local_only: bool) -> Result<bool> {
    println!("{} Checking deployment...", console::style("bonesdeploy doctor").bold());

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML)).ok();
    let deploy_on_push = cfg.as_ref().is_some_and(|c| c.deploy_on_push);

    let mut issues = 0usize;
    let mut pending = false;

    issues += print_check(".bones config", check_bones_config(), Some(output::run_command("bonesdeploy init")));
    issues += print_check(
        "deployment scripts",
        check_deployment_scripts(),
        Some(String::from("rename it with a numeric prefix, like 01_build.sh")),
    );

    let local_branch_issue = cfg.as_ref().and_then(check_local_branch);
    issues += print_check(
        "deploy branch",
        local_branch_issue,
        cfg.as_ref().map(|c| format!("git checkout -b {} && git push {} {}", c.branch, c.remote_name, c.branch)),
    );

    if deploy_on_push {
        issues += print_check("pre-push hook", check_pre_push_hook(), Some(output::run_command("bonesdeploy init")));
    }

    if !local_only {
        let (remote_issues, remote_pending) = check_remote(cfg.as_ref()).await;
        issues += remote_issues;
        pending |= remote_pending;
    }

    if issues == 0 {
        println!();
        if pending {
            println!("{} Deployment is provisioned and waiting for the first Git push.", output::pending_marker());
        } else {
            println!("{} All checks passed.", output::success_marker());
        }
        Ok(pending)
    } else {
        println!();
        let issue_word = if issues == 1 { "issue" } else { "issues" };
        anyhow::bail!("Doctor found {issues} {issue_word}.");
    }
}

async fn check_remote(cfg: Option<&config::Bones>) -> (usize, bool) {
    match cfg {
        Some(cfg) => {
            let remote_ssh_issue = check_remote_ssh(cfg).await;
            let mut issues = print_check(
                "remote SSH",
                remote_ssh_issue.clone(),
                Some(String::from("check host, port, and SSH access.")),
            );
            if remote_ssh_issue.is_none() {
                let (remote_issue, pending) = check_remote_doctor(cfg).await;
                issues += print_check(
                    "remote doctor",
                    remote_issue,
                    Some(format!(
                        "{} or {}",
                        output::run_command("bonesdeploy push"),
                        output::run_command("bonesdeploy remote setup")
                    )),
                );
                return (issues, pending);
            }
            (issues, false)
        }
        None => {
            (print_failure("remote SSH", "Missing .bones config", Some(output::run_command("bonesdeploy init"))), false)
        }
    }
}

fn print_check(label: &str, issue: Option<String>, next: Option<String>) -> usize {
    match issue {
        None => {
            println!("{} {label}", output::success_marker());
            0
        }
        Some(issue) => print_failure(label, &issue, next),
    }
}

fn print_failure(label: &str, issue: &str, next: Option<String>) -> usize {
    println!("{} {label}", output::failure_marker());
    let issue = issue.replace('\n', "\n  ");
    println!("  {issue}");
    if let Some(next) = next {
        println!("  Next: {next}");
    }
    1
}

fn check_bones_config() -> Option<String> {
    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);

    if !bones_dir.exists() {
        return Some(String::from("Missing .bones config"));
    }

    if !bones_dir.is_symlink() {
        return Some(String::from(".bones is not managed by bonesdeploy"));
    }

    if !Path::new(paths::LOCAL_BONES_TOML).exists() {
        return Some(format!("Missing {}", paths::LOCAL_BONES_TOML));
    }

    if let Err(error) = config::load(Path::new(paths::LOCAL_BONES_TOML)) {
        return Some(format!("Invalid {}: {error:#}", paths::LOCAL_BONES_TOML));
    }

    None
}

fn check_deployment_scripts() -> Option<String> {
    let deployment_dir = Path::new(paths::LOCAL_BONES_DEPLOYMENT_DIR);
    if !deployment_dir.exists() {
        return None;
    }

    for subdir in ["build", "prepare"] {
        let scripts_dir = deployment_dir.join(subdir);
        if !scripts_dir.exists() {
            continue;
        }

        let entries = match fs::read_dir(&scripts_dir) {
            Ok(entries) => entries,
            Err(error) => return Some(format!("Cannot read {}: {error}", scripts_dir.display())),
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => return Some(format!("Cannot read an entry in {}: {error}", scripts_dir.display())),
            };
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.extension().is_none_or(|extension| extension != "sh") {
                continue;
            }
            if !is_deployment_script(&name) {
                return Some(format!("Deployment script must use the NN_name.sh convention: {subdir}/{name}"));
            }
        }
    }

    None
}

fn is_deployment_script(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() > 6
        && bytes[0].is_ascii_digit()
        && bytes[1].is_ascii_digit()
        && bytes[2] == b'_'
        && Path::new(name).extension().is_some_and(|extension| extension == "sh")
}

fn check_local_branch(cfg: &config::Bones) -> Option<String> {
    if cfg.branch.is_empty() {
        return None;
    }
    let ref_name = format!("refs/heads/{}", cfg.branch);
    let output = match Command::new("git").args(["rev-parse", "--verify", &ref_name]).output() {
        Ok(output) => output,
        Err(error) => return Some(format!("Unable to run git: {error}")),
    };
    if output.status.success() {
        return None;
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Some(format!("Local branch '{}' does not exist", cfg.branch))
    } else {
        Some(format!("Local branch '{}' does not exist: {}", cfg.branch, stderr))
    }
}

fn check_pre_push_hook() -> Option<String> {
    let link = Path::new(paths::GIT_PRE_PUSH_HOOK);

    if !link.symlink_metadata().is_ok_and(|m| m.is_symlink()) {
        return Some(String::from("pre-push hook is not installed"));
    }

    let target = match fs::read_link(link) {
        Ok(target) => target,
        Err(error) => return Some(format!("Cannot read pre-push hook link: {error}")),
    };
    let expected = Path::new(paths::PRE_PUSH_HOOK_TARGET);
    if target != expected {
        return Some(String::from("pre-push hook is not installed"));
    }

    None
}

async fn check_remote_ssh(cfg: &config::Bones) -> Option<String> {
    match ssh::connect(cfg).await {
        Ok(session) => {
            // This check only asks whether SSH can connect; ignore failure while closing the test session.
            let _ = session.close().await;
            None
        }
        Err(error) => Some(format!("Cannot connect to remote\n  {error}")),
    }
}

async fn check_remote_doctor(cfg: &config::Bones) -> (Option<String>, bool) {
    let session = match ssh::connect_privileged(cfg).await {
        Ok(session) => session,
        Err(error) => return (Some(format!("Cannot connect as privileged remote user\n  {error}")), false),
    };
    let command = format!("bonesremote doctor --site {}", &cfg.project_name);
    let result = ssh::run_cmd(&session, &command).await;
    // The remote command has finished; ignore failure while closing this short-lived SSH session.
    let _ = session.close().await;

    match result {
        Ok(output) => {
            let pending = output.contains("has not been pushed yet");
            if pending {
                for line in output.lines().filter(|line| line.contains("has not been pushed yet")) {
                    println!("{} {}", output::pending_marker(), line.trim());
                }
            }
            (None, pending)
        }
        Err(error) => (Some(format!("remote doctor failed\n  {error}")), false),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process;

    use anyhow::Result;

    use super::check_deployment_scripts;
    use crate::commands::push_state;

    #[test]
    fn doctor_points_at_correct_remote_import_flow() {
        assert_eq!(push_state::remote_import_command("acme"), "bonesremote site import --site 'acme'");
    }

    #[test]
    fn deployment_script_check_accepts_nested_build_and_prepare_layout() -> Result<()> {
        let cwd = env::current_dir()?;
        let root = env::temp_dir().join(format!("bonesdeploy-doctor-nested-layout-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(root.join(".bones/deployment/build"))?;
        fs::create_dir_all(root.join(".bones/deployment/prepare"))?;
        fs::write(root.join(".bones/deployment/build/01_build.sh"), "")?;
        fs::write(root.join(".bones/deployment/build/README.md"), "# Build Scripts")?;
        fs::write(root.join(".bones/deployment/prepare/02_prepare.sh"), "")?;
        fs::write(root.join(".bones/deployment/prepare/README.md"), "# Prepare Scripts")?;

        env::set_current_dir(&root)?;
        let result = check_deployment_scripts();
        env::set_current_dir(cwd)?;

        fs::remove_dir_all(&root).ok();
        assert!(result.is_none(), "nested deployment layout should be accepted: {result:?}");
        Ok(())
    }
}
