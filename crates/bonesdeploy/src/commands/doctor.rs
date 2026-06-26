use std::fs;
use std::path::Path;

use anyhow::Result;
use shared::paths::bonesremote_registry_path;
use tokio::runtime::Runtime;

use crate::config;
use crate::infra::ssh;
use shared::paths;

pub async fn run(local_only: bool) -> Result<()> {
    println!("Checking deployment...");

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML)).ok();
    let deploy_on_push = cfg.as_ref().is_some_and(|c| c.deploy_on_push);

    let mut issues = 0usize;

    issues += print_check(".bones config", check_bones_config(), Some("run bonesdeploy init"));
    issues += print_check(
        "deployment scripts",
        check_deployment_scripts(),
        Some("rename it with a numeric prefix, like 01_build.sh"),
    );

    if deploy_on_push {
        issues += print_check("pre-push hook", check_pre_push_hook(), Some("run bonesdeploy init"));
    }

    if !local_only {
        match &cfg {
            Some(cfg) => {
                let remote_ssh_issue = check_remote_ssh(cfg).await;
                issues +=
                    print_check("remote SSH", remote_ssh_issue.clone(), Some("check host, port, and SSH access."));
                if remote_ssh_issue.is_none() {
                    issues += print_check(
                        "bonesremote",
                        check_bonesremote(cfg).await,
                        Some("run bonesdeploy remote bootstrap"),
                    );
                    issues += print_check(".bones sync", check_bones_sync(cfg), Some("run bonesdeploy push"));
                }
            }
            None => {
                issues += print_failure("remote SSH", "Missing .bones config", Some("run bonesdeploy init"));
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

fn print_check(label: &str, issue: Option<String>, next: Option<&str>) -> usize {
    match issue {
        None => {
            println!("✓ {label}");
            0
        }
        Some(issue) => print_failure(label, &issue, next),
    }
}

fn print_failure(label: &str, issue: &str, next: Option<&str>) -> usize {
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

    let entries = fs::read_dir(deployment_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let has_numeric_prefix = name.chars().take_while(char::is_ascii_digit).count() > 0;
        if !has_numeric_prefix {
            return Some(format!("Deployment script is not ordered: {name}"));
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

async fn check_bonesremote(cfg: &config::Bones) -> Option<String> {
    let session = ssh::connect(cfg).await.ok()?;
    let result = ssh::run_cmd(&session, "command -v bonesremote").await;
    let _ = session.close().await;

    if result.is_ok() { None } else { Some(String::from("bonesremote is missing")) }
}

fn check_bones_sync(cfg: &config::Bones) -> Option<String> {
    let registry_path = bonesremote_registry_path(&cfg.project_name);
    let command = format!("test -r {}", shell_quote(&registry_path.display().to_string()));
    let Ok(runtime) = Runtime::new() else {
        return Some(String::from("Could not start runtime to check remote site state"));
    };
    let Ok(session) = runtime.block_on(ssh::connect(cfg)) else {
        return Some(String::from("Could not connect to remote site state"));
    };
    let ok = runtime.block_on(ssh::run_cmd(&session, &command)).is_ok();
    let _ = runtime.block_on(session.close());

    if ok { None } else { Some(String::from("remote bonesremote site state is missing; run bonesdeploy push")) }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::shell_quote;
    use crate::commands::push_state;

    #[test]
    fn doctor_points_at_new_remote_import_flow() {
        assert_eq!(push_state::remote_import_command("acme"), "bonesremote site import --site 'acme'");
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("a'b"), "'a'\\''b'");
    }
}
