use std::fs;
use std::path::Path;

use anyhow::Result;
use console::style;

use crate::config;
use crate::infra::rsync;
use crate::infra::ssh;
use shared::config::default_deploy_user;
use shared::paths;

pub async fn run(local_only: bool) -> Result<()> {
    println!("{}", style("bonesdeploy doctor").bold());

    let mut issues: Vec<String> = Vec::new();

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML)).ok();
    let deploy_on_push = cfg.as_ref().is_some_and(|c| c.deploy_on_push);

    check_bones_structure(&mut issues);
    check_deployment_naming(&mut issues);

    if deploy_on_push {
        check_pre_push_symlink(&mut issues);
    }

    if !local_only {
        match &cfg {
            Some(cfg) => check_remote(cfg, deploy_on_push, &mut issues).await,
            None => issues.push("Cannot load .bones/bones.toml; skipping remote checks".into()),
        }
    }

    if issues.is_empty() {
        println!("\n{} All checks passed.", style("OK").green().bold());
        Ok(())
    } else {
        println!();
        for issue in &issues {
            println!("  {} {issue}", style("!").red().bold());
        }
        anyhow::bail!("Doctor found {} issue{}", issues.len(), if issues.len() == 1 { "" } else { "s" });
    }
}

fn check_bones_structure(issues: &mut Vec<String>) {
    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);

    if !bones_dir.exists() {
        issues.push(format!("{}/ does not exist", paths::LOCAL_BONES_DIR));
        return;
    }

    if !bones_dir.is_symlink() {
        issues.push(format!(
            "{}/ is not a symlink — expected a symlink to ~/.config/bonesdeploy/<project>.bones/",
            paths::LOCAL_BONES_DIR
        ));
        return;
    }

    let expected = [
        paths::LOCAL_BONES_TOML,
        paths::LOCAL_BONES_HOOKS_SCRIPT,
        paths::LOCAL_BONES_HOOKS_DIR,
        paths::LOCAL_BONES_DEPLOYMENT_DIR,
    ];

    for path in &expected {
        if !Path::new(path).exists() {
            issues.push(format!("{path} is missing"));
        }
    }
}

fn check_deployment_naming(issues: &mut Vec<String>) {
    let deployment_dir = Path::new(paths::LOCAL_BONES_DEPLOYMENT_DIR);
    if !deployment_dir.exists() {
        return;
    }

    let Ok(entries) = fs::read_dir(deployment_dir) else {
        return;
    };

    for entry in entries {
        let Ok(entry) = entry else { continue };
        let name = entry.file_name();
        let name = name.to_string_lossy();

        // Scripts must start with a numeric prefix like "01_"
        let has_numeric_prefix = name.chars().take_while(char::is_ascii_digit).count() > 0;

        if !has_numeric_prefix {
            issues.push(format!("Deployment script '{name}' does not start with a numeric prefix (e.g. 01_)"));
        }
    }
}

fn check_pre_push_symlink(issues: &mut Vec<String>) {
    let link = Path::new(paths::GIT_PRE_PUSH_HOOK);

    if !link.symlink_metadata().is_ok_and(|m| m.is_symlink()) {
        issues.push(format!("{} is not symlinked", paths::GIT_PRE_PUSH_HOOK));
        return;
    }

    let Ok(target) = fs::read_link(link) else {
        issues.push(format!("{}: cannot read symlink target", paths::GIT_PRE_PUSH_HOOK));
        return;
    };

    let expected = Path::new(paths::PRE_PUSH_HOOK_TARGET);
    if target != expected {
        issues.push(format!(
            "{} points to '{}', expected '{}'",
            paths::GIT_PRE_PUSH_HOOK,
            target.display(),
            expected.display()
        ));
    }
}

async fn check_remote(cfg: &config::Bones, deploy_on_push: bool, issues: &mut Vec<String>) {
    let session = match ssh::connect(cfg).await {
        Ok(s) => s,
        Err(e) => {
            issues.push(format!("Cannot connect to remote: {e}"));
            return;
        }
    };

    let repo_path = &cfg.repo_path;

    if ssh::run_cmd(&session, "command -v bonesremote").await.is_err() {
        issues.push("bonesremote is not available on the remote".into());
    }

    let check_bones = format!("test -d {repo_path}/{}", paths::BONES_DIR);
    if ssh::run_cmd(&session, &check_bones).await.is_err() {
        issues.push(format!("{repo_path}/{}/ does not exist on remote (run 'bonesdeploy push')", paths::BONES_DIR));
    }

    check_rsync_sync(cfg, issues);

    // Check hooks are symlinked properly (only when git-triggered deploy is enabled)
    if deploy_on_push {
        let check_hooks = format!(
            "for hook in {repo_path}/{}/{}/{}; do \
            name=$(basename \"$hook\"); \
            link=\"{repo_path}/{}/$name\"; \
            if [ ! -L \"$link\" ] || [ \"$(readlink \"$link\")\" != \"$hook\" ]; then \
                echo \"$name\"; \
            fi; \
         done",
            paths::BONES_DIR,
            paths::HOOKS_DIR,
            "*",
            paths::HOOKS_DIR
        );
        match ssh::run_cmd(&session, &check_hooks).await {
            Ok(output) => {
                for hook in output.lines() {
                    let hook = hook.trim();
                    if !hook.is_empty() {
                        issues.push(format!(
                            "{repo_path}/{}/{hook} is not properly symlinked to {}/{}/{hook}",
                            paths::HOOKS_DIR,
                            paths::BONES_DIR,
                            paths::HOOKS_DIR
                        ));
                    }
                }
            }
            Err(e) => issues.push(format!("Failed to check remote hook symlinks: {e}")),
        }
    }

    let _ = session.close().await;
}

fn check_rsync_sync(cfg: &config::Bones, issues: &mut Vec<String>) {
    let user = default_deploy_user();
    let host = &cfg.host;
    let port = &cfg.port;
    let repo_path = &cfg.repo_path;
    let dest = format!("{user}@{host}:{repo_path}/{}/", paths::BONES_DIR);

    let ssh_arg = format!("ssh -p {port}");
    let source = format!("{}/", paths::LOCAL_BONES_DIR);
    let output = match rsync::output(&["-avnc", "--delete", "--exclude=.lib/", "-e", &ssh_arg, &source, &dest]) {
        Ok(output) => output,
        Err(e) => {
            issues.push(format!("Failed to run rsync sync check: {e}"));
            return;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        issues.push(format!("rsync sync check failed: {stderr}"));
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let changed: Vec<&str> = stdout
        .lines()
        .filter(|line| {
            let line = line.trim();
            // Skip rsync summary/header lines and directory-only entries
            !line.is_empty()
                && !line.starts_with("sending ")
                && !line.starts_with("sent ")
                && !line.starts_with("total ")
                && !line.ends_with('/')
        })
        .collect();

    if !changed.is_empty() {
        issues.push(format!(
            "Local .bones/ is out of sync with remote (run 'bonesdeploy push'). Changed files:\n{}",
            changed.iter().map(|f| format!("      {f}")).collect::<Vec<_>>().join("\n")
        ));
    }
}
