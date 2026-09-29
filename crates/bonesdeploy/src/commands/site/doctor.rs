use std::fs;
use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::{
    build_contract::TARGET_PLATFORM_NAME,
    config::{RuntimeBackend, is_numbered_shell_script},
    paths,
};

use crate::config;
use crate::infra::{self, git, ssh};
use crate::local_build;
use crate::ui::output;

pub async fn run(local_only: bool, verbose: bool) -> Result<()> {
    run_with_pending(local_only, verbose).await.map(|_| ())
}

pub(super) async fn run_with_pending(local_only: bool, verbose: bool) -> Result<bool> {
    println!("{} Checking deployment...", console::style("bonesdeploy site doctor").bold());

    let cfg = config::load(Path::new(paths::DOT_ENV)).ok();
    let mut issues = 0usize;
    let mut pending = false;

    issues += print_check("local project layout", check_local_layout(), Some(output::run_command("bonesdeploy init")));
    issues += print_check(
        "deployment scripts",
        check_deployment_scripts(),
        Some(String::from("rename it with a numeric prefix, like 01_build.sh")),
    );

    if cfg.as_ref().is_some_and(|config| config.runtime.backend == RuntimeBackend::Native) {
        issues += print_check(
            "local Docker",
            check_local_docker(),
            Some(String::from("install and start Docker with Linux containers enabled")),
        );
        issues += print_check(
            "local builder image",
            check_local_builder_image(),
            Some(format!("run `docker pull --platform {TARGET_PLATFORM_NAME}` for the configured builder image")),
        );
        issues += print_check(
            "local target execution",
            check_local_target_execution(),
            Some(format!("install compatible Docker emulation for {TARGET_PLATFORM_NAME}")),
        );
        issues += print_check(
            "local build cache",
            check_local_cache(),
            Some(String::from("make the XDG cache directory writable")),
        );
    }

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
                issues += print_check(
                    "remote doctor",
                    remote_issue,
                    Some(output::run_command("bonesdeploy site setup --yes")),
                );
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
    check_local_branch_at(Path::new("."), cfg)
}

fn check_local_branch_at(repo: &Path, cfg: &config::Bones) -> Option<String> {
    if cfg.branch.is_empty() {
        return None;
    }
    if cfg.runtime.backend == RuntimeBackend::Native {
        return git::resolve_branch_commit(repo, &cfg.branch)
            .map(|_| ())
            .err()
            .map(|error| format!("Unable to resolve local branch '{}' to an exact commit: {error}", cfg.branch));
    }
    match git::branch_exists_at(repo, &cfg.branch) {
        Ok(true) => None,
        Ok(false) => Some(format!("Local branch '{}' does not exist", cfg.branch)),
        Err(error) => Some(format!("Unable to inspect local branch '{}': {error}", cfg.branch)),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use super::check_local_branch_at;
    use crate::config::Bones;

    #[test]
    fn local_doctor_requires_the_configured_branch_to_resolve_to_a_commit() -> anyhow::Result<()> {
        let repo = tempfile::tempdir()?;
        run_git(repo.path(), ["init", "--initial-branch=main"])?;
        run_git(repo.path(), ["config", "user.email", "test@example.com"])?;
        run_git(repo.path(), ["config", "user.name", "Test"])?;
        fs::write(repo.path().join("tracked"), "content")?;
        run_git(repo.path(), ["add", "."])?;
        run_git(repo.path(), ["commit", "-m", "initial"])?;
        let mut config = Bones::default();
        config.branch = String::from("main");
        assert_eq!(check_local_branch_at(repo.path(), &config), None);
        config.branch = String::from("missing");
        assert!(check_local_branch_at(repo.path(), &config).is_some());
        Ok(())
    }

    fn run_git<const N: usize>(repo: &Path, arguments: [&str; N]) -> anyhow::Result<()> {
        let status = Command::new("git").arg("-C").arg(repo).args(arguments).status()?;
        anyhow::ensure!(status.success(), "git command failed");
        Ok(())
    }
}

fn check_local_docker() -> Option<String> {
    local_build::docker_available_linux().err().map(|error| error.to_string())
}

fn check_local_builder_image() -> Option<String> {
    match local_build::docker_image_available() {
        Ok(true) => None,
        Ok(false) => Some(String::from("Pinned local builder image is not available")),
        Err(error) => Some(error.to_string()),
    }
}

fn check_local_target_execution() -> Option<String> {
    local_build::probe_target_execution().err().map(|error| error.to_string())
}

fn check_local_cache() -> Option<String> {
    let cache = local_build::local_cache_path("doctor");
    let ancestor = cache.ancestors().find(|path| path.exists())?;
    if ancestor.is_dir() && !ancestor.metadata().is_ok_and(|metadata| metadata.permissions().readonly()) {
        None
    } else {
        Some(format!("Local build cache parent is unavailable: {}", ancestor.display()))
    }
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

async fn check_remote_doctor(cfg: &config::Bones, verbose: bool) -> (Option<String>, bool) {
    let session = match ssh::connect_privileged(cfg).await {
        Ok(session) => session,
        Err(error) => return (Some(format!("Cannot connect as privileged remote user\n  {error}")), false),
    };
    let result = match infra::sync_control_plane(&session, cfg).await {
        Ok(()) => {
            let command = format!("bonesremote doctor --site {}", ssh::shell_quote(&cfg.project_name));
            ssh::run_cmd(&session, &command).await
        }
        Err(error) => Err(error),
    };
    let _ = session.close().await;

    match result {
        Ok(output) => {
            let pending = output::render_remote_doctor_output(&output, verbose);
            (None, pending)
        }
        Err(error) => (Some(format!("remote doctor failed\n  {error}")), false),
    }
}
