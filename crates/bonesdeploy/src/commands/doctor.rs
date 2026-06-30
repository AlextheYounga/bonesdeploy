use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::config;
use crate::infra::ssh;
use crate::ui::output;
use shared::paths;

pub async fn run(local_only: bool) -> Result<()> {
    println!("Checking deployment...");

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML)).ok();
    let deploy_on_push = cfg.as_ref().is_some_and(|c| c.deploy_on_push);

    let mut issues = 0usize;

    issues += print_check(".bones config", check_bones_config(), Some(output::run_command("bonesdeploy init")));
    issues += print_check(
        "deployment scripts",
        check_deployment_scripts(),
        Some(String::from("rename it with a numeric prefix, like 01_build.sh")),
    );

    if deploy_on_push {
        issues += print_check("pre-push hook", check_pre_push_hook(), Some(output::run_command("bonesdeploy init")));
    }

    if !local_only {
        match &cfg {
            Some(cfg) => {
                let remote_ssh_issue = check_remote_ssh(cfg).await;
                issues += print_check(
                    "remote SSH",
                    remote_ssh_issue.clone(),
                    Some(String::from("check host, port, and SSH access.")),
                );
                if remote_ssh_issue.is_none() {
                    issues += print_check(
                        "remote doctor",
                        check_remote_doctor(cfg).await,
                        Some(format!(
                            "{} or {}",
                            output::run_command("bonesdeploy push"),
                            output::run_command("bonesdeploy remote setup")
                        )),
                    );
                }
            }
            None => {
                issues +=
                    print_failure("remote SSH", "Missing .bones config", Some(output::run_command("bonesdeploy init")));
            }
        }
    }

    if issues == 0 {
        println!();
        println!("All checks passed.");
        Ok(())
    } else {
        println!();
        let issue_word = if issues == 1 { "issue" } else { "issues" };
        anyhow::bail!("Doctor found {issues} {issue_word}.");
    }
}

fn print_check(label: &str, issue: Option<String>, next: Option<String>) -> usize {
    match issue {
        None => {
            println!("✓ {label}");
            0
        }
        Some(issue) => print_failure(label, &issue, next),
    }
}

fn print_failure(label: &str, issue: &str, next: Option<String>) -> usize {
    println!("✗ {label}");
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

        let entries = fs::read_dir(&scripts_dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let has_numeric_prefix = name.chars().take_while(char::is_ascii_digit).count() > 0;
            if !has_numeric_prefix {
                return Some(format!("Deployment script is not ordered: {subdir}/{name}"));
            }
        }
    }

    None
}

fn check_pre_push_hook() -> Option<String> {
    let link = Path::new(paths::GIT_PRE_PUSH_HOOK);

    if !link.symlink_metadata().is_ok_and(|m| m.is_symlink()) {
        return Some(String::from("pre-push hook is not installed"));
    }

    let target = fs::read_link(link).ok()?;
    let expected = Path::new(paths::PRE_PUSH_HOOK_TARGET);
    if target != expected {
        return Some(String::from("pre-push hook is not installed"));
    }

    None
}

async fn check_remote_ssh(cfg: &config::Bones) -> Option<String> {
    match ssh::connect(cfg).await {
        Ok(session) => {
            let _ = session.close().await;
            None
        }
        Err(error) => Some(format!("Cannot connect to remote\n  {error}")),
    }
}

async fn check_remote_doctor(cfg: &config::Bones) -> Option<String> {
    let session = match ssh::connect_privileged(cfg).await {
        Ok(session) => session,
        Err(error) => return Some(format!("Cannot connect as privileged remote user\n  {error}")),
    };
    let command = format!("bonesremote doctor --site {}", &cfg.project_name);
    let result = ssh::run_cmd(&session, &command).await;
    let _ = session.close().await;

    result.err().map(|error| format!("remote doctor failed\n  {error}"))
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
        fs::write(root.join(".bones/deployment/prepare/02_prepare.sh"), "")?;

        env::set_current_dir(&root)?;
        let result = check_deployment_scripts();
        env::set_current_dir(cwd)?;

        fs::remove_dir_all(&root).ok();
        assert!(result.is_none(), "nested deployment layout should be accepted: {result:?}");
        Ok(())
    }
}
