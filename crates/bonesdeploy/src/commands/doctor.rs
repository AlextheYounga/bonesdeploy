use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::config;
use crate::infra::{git, ssh};
use crate::ui::output;
use bonesdeploy_core::{config::is_numbered_shell_script, paths};

pub async fn run(local_only: bool, verbose: bool) -> Result<bool> {
    println!("{} Checking deployment...", console::style("bonesdeploy doctor").bold());

    let cfg = config::load(Path::new(paths::DOT_ENV)).ok();
    let mut issues = 0usize;
    let mut pending = false;

    issues += print_check("local project layout", check_local_layout(), Some(output::run_command("bonesdeploy init")));
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
                issues +=
                    print_check("remote doctor", remote_issue, Some(output::run_command("bonesdeploy remote setup")));
                return (issues, pending);
            }
            (issues, false)
        }
        None => (
            print_failure(
                "remote SSH",
                "Missing root .env configuration",
                Some(output::run_command("bonesdeploy init")),
            ),
            false,
        ),
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

fn check_local_layout() -> Option<String> {
    let old_layout = Path::new(paths::OLD_BONES_DIR);
    if fs::symlink_metadata(old_layout).is_ok() {
        return Some(String::from("Old .bones layout detected; run `bonesdeploy update` before using this project"));
    }

    let infra = Path::new(paths::LOCAL_INFRA_DIR);
    if !infra.is_dir() {
        return Some(String::from("Missing infra/ directory; run `bonesdeploy init`"));
    }

    let env_file = Path::new(paths::DOT_ENV);
    if !env_file.is_file() {
        return Some(String::from("Missing root .env; run `bonesdeploy init`"));
    }

    if let Err(error) = config::load(env_file) {
        return Some(format!("Invalid root .env: {error:#}"));
    }

    None
}

fn check_deployment_scripts() -> Option<String> {
    let deployment_dir = Path::new(paths::LOCAL_INFRA_DEPLOYMENT_DIR);
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
    match git::branch_exists(&cfg.branch) {
        Ok(true) => None,
        Ok(false) => Some(format!("Local branch '{}' does not exist", cfg.branch)),
        Err(error) => Some(format!("Unable to inspect git branch '{}': {error}", cfg.branch)),
    }
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

pub fn render_remote_doctor_output(output: &str, verbose: bool) -> bool {
    let pending =
        output.contains("has not been pushed yet") || output.contains("Run 'bonesdeploy secrets push' first.");
    if verbose {
        print!("{output}");
        if !output.is_empty() && !output.ends_with('\n') {
            println!();
        }
    } else if pending {
        for line in output.lines().filter(|line| {
            line.contains("has not been pushed yet") || line.contains("Run 'bonesdeploy secrets push' first.")
        }) {
            let clean = strip_ansi(line);
            let clean = clean.trim().strip_prefix('•').map_or(clean.trim(), str::trim_start);
            println!("{} {}", output::pending_marker(), clean);
        }
    }
    pending
}

pub fn strip_ansi(input: &str) -> String {
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
