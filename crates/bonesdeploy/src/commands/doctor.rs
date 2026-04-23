use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use console::style;

use crate::config;
use crate::ssh;

pub async fn run(local_only: bool) -> Result<()> {
    println!("{}", style("bonesdeploy doctor").bold());

    let mut issues: Vec<String> = Vec::new();

    check_bones_structure(&mut issues);
    check_deployment_naming(&mut issues);
    check_pre_push_symlink(&mut issues);

    if !local_only {
        let bones_toml = Path::new(config::Constants::BONES_TOML);
        match config::load(bones_toml) {
            Ok(cfg) => check_remote(&cfg, &mut issues).await,
            Err(e) => issues.push(format!("Cannot load config: {e}")),
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
    let bones_dir = Path::new(config::Constants::BONES_DIR);

    if !bones_dir.exists() {
        issues.push(format!("{}/ directory does not exist", config::Constants::BONES_DIR));
        return;
    }

    let expected = [
        config::Constants::BONES_TOML,
        config::Constants::BONES_HOOKS_SCRIPT,
        config::Constants::BONES_HOOKS_DIR,
        config::Constants::BONES_DEPLOYMENT_DIR,
    ];

    for path in &expected {
        if !Path::new(path).exists() {
            issues.push(format!("{path} is missing"));
        }
    }
}

fn check_deployment_naming(issues: &mut Vec<String>) {
    let deployment_dir = Path::new(config::Constants::BONES_DEPLOYMENT_DIR);
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
    let link = Path::new(config::Constants::GIT_PRE_PUSH_HOOK_PATH);

    if !link.symlink_metadata().is_ok_and(|m| m.is_symlink()) {
        issues.push(format!("{} is not symlinked", config::Constants::GIT_PRE_PUSH_HOOK_PATH));
        return;
    }

    let Ok(target) = fs::read_link(link) else {
        issues.push(format!("{}: cannot read symlink target", config::Constants::GIT_PRE_PUSH_HOOK_PATH));
        return;
    };

    let expected = Path::new(config::Constants::PRE_PUSH_HOOK_TARGET);
    if target != expected {
        issues.push(format!(
            "{} points to '{}', expected '{}'",
            config::Constants::GIT_PRE_PUSH_HOOK_PATH,
            target.display(),
            expected.display()
        ));
    }
}

async fn check_remote(cfg: &config::BonesConfig, issues: &mut Vec<String>) {
    let session = match ssh::connect(cfg).await {
        Ok(s) => s,
        Err(e) => {
            issues.push(format!("Cannot connect to remote: {e}"));
            return;
        }
    };

    let git_dir = &cfg.data.git_dir;

    // Check bonesremote is globally available
    if ssh::run_cmd(&session, "command -v bonesremote").await.is_err() {
        issues.push("bonesremote is not available on the remote".into());
    }

    // Check bones/ folder exists on remote
    let check_bones = format!("test -d {git_dir}/{}", config::Constants::REMOTE_BONES_DIR);
    if ssh::run_cmd(&session, &check_bones).await.is_err() {
        issues.push(format!(
            "{git_dir}/{}/ does not exist on remote (run 'bonesdeploy push')",
            config::Constants::REMOTE_BONES_DIR
        ));
    }

    // Check local .bones/ is in sync with remote
    check_rsync_sync(cfg, issues);

    // Check hooks are symlinked properly
    let check_hooks = format!(
        "for hook in {git_dir}/{}/{}/{}; do \
            name=$(basename \"$hook\"); \
            link=\"{git_dir}/{}/$name\"; \
            if [ ! -L \"$link\" ] || [ \"$(readlink \"$link\")\" != \"$hook\" ]; then \
                echo \"$name\"; \
            fi; \
         done",
        config::Constants::REMOTE_BONES_DIR,
        config::Constants::REMOTE_HOOKS_DIR,
        "*",
        config::Constants::REMOTE_HOOKS_DIR
    );
    match ssh::run_cmd(&session, &check_hooks).await {
        Ok(output) => {
            for hook in output.lines() {
                let hook = hook.trim();
                if !hook.is_empty() {
                    issues.push(format!(
                        "{git_dir}/{}/{hook} is not properly symlinked to {}/{}/{hook}",
                        config::Constants::REMOTE_HOOKS_DIR,
                        config::Constants::REMOTE_BONES_DIR,
                        config::Constants::REMOTE_HOOKS_DIR
                    ));
                }
            }
        }
        Err(e) => issues.push(format!("Failed to check remote hook symlinks: {e}")),
    }

    let _ = session.close().await;
}

fn check_rsync_sync(cfg: &config::BonesConfig, issues: &mut Vec<String>) {
    let user = &cfg.permissions.defaults.deploy_user;
    let host = &cfg.data.host;
    let port = &cfg.data.port;
    let git_dir = &cfg.data.git_dir;
    let dest = format!("{user}@{host}:{git_dir}/{}/", config::Constants::REMOTE_BONES_DIR);

    let output = Command::new("rsync")
        .args([
            "-avnc",
            "--delete",
            "-e",
            &format!("ssh -p {port}"),
            &format!("{}/", config::Constants::BONES_DIR),
            &dest,
        ])
        .output();

    let output = match output {
        Ok(o) => o,
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
