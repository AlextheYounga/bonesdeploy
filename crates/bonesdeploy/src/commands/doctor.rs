use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::config;
use crate::infra::ssh;
use crate::ui::output;
use bonesdeploy_core::{config::is_numbered_shell_script, paths};

pub async fn run(local_only: bool, verbose: bool) -> Result<bool> {
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
        let (remote_issues, remote_pending) = check_remote(cfg.as_ref(), verbose).await;
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

async fn check_remote(cfg: Option<&config::Bones>, verbose: bool) -> (usize, bool) {
    match cfg {
        Some(cfg) => {
            let remote_ssh_issue = check_remote_ssh(cfg).await;
            let mut issues = print_check(
                "remote SSH",
                remote_ssh_issue.clone(),
                Some(String::from("check host, port, and SSH access.")),
            );
            if remote_ssh_issue.is_none() {
                let (remote_issue, pending) = check_remote_doctor(cfg, verbose).await;
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
            if !is_numbered_shell_script(&name) {
                return Some(format!("Deployment script must use the NN_name.sh convention: {subdir}/{name}"));
            }
        }
    }

    None
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
    let guard = Path::new(paths::GIT_PRE_PUSH_HOOK);
    let Ok(contents) = fs::read_to_string(guard) else {
        return Some(String::from("pre-push hook is not installed"));
    };

    if contents.contains("bonesdeploy-pre-push-v1") {
        return None;
    }

    Some(String::from("pre-push hook is missing or stale"))
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

async fn check_remote_doctor(cfg: &config::Bones, verbose: bool) -> (Option<String>, bool) {
    let session = match ssh::connect_privileged(cfg).await {
        Ok(session) => session,
        Err(error) => return (Some(format!("Cannot connect as privileged remote user\n  {error}")), false),
    };
    let command = format!("bonesremote doctor --site {}", cfg.project_name);
    let result = ssh::run_cmd(&session, &command).await;
    // The remote command has finished; ignore failure while closing this short-lived SSH session.
    let _ = session.close().await;

    match result {
        Ok(output) => {
            let pending = render_remote_doctor_output(&output, verbose);
            (None, pending)
        }
        Err(error) => (Some(format!("remote doctor failed\n  {error}")), false),
    }
}

fn render_remote_doctor_output(output: &str, verbose: bool) -> bool {
    let pending = output.contains("has not been pushed yet");
    if verbose {
        print!("{output}");
        if !output.is_empty() && !output.ends_with('\n') {
            println!();
        }
    } else if pending {
        for line in output.lines().filter(|line| line.contains("has not been pushed yet")) {
            let clean = strip_ansi(line);
            let clean = clean.trim().strip_prefix('•').map_or(clean.trim(), str::trim_start);
            println!("{} {}", output::pending_marker(), clean);
        }
    }
    pending
}

fn strip_ansi(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\x1b' {
            i += 1;
            if i < chars.len() && chars[i] == '[' {
                i += 1;
                while i < chars.len() && !('@'..='~').contains(&chars[i]) {
                    i += 1;
                }
                i += 1;
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{render_remote_doctor_output, strip_ansi};

    #[test]
    fn verbose_remote_report_preserves_pending_state() {
        assert!(render_remote_doctor_output(
            "bonesremote doctor\n  • deploy branch 'main' has not been pushed yet\n",
            true
        ));
        assert!(!render_remote_doctor_output("bonesremote doctor\n✓ All checks passed.\n", true));
    }

    #[test]
    fn strip_ansi_removes_sgr_color_sequences() {
        assert_eq!(
            strip_ansi("\x1b[1;33m•\x1b[0m deploy branch 'master' has not been pushed yet"),
            "• deploy branch 'master' has not been pushed yet"
        );
        assert_eq!(strip_ansi("plain text"), "plain text");
    }
}
