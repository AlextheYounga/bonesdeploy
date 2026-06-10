use super::{InitArgs, collect_non_interactive};

use anyhow::{Result, bail};
use shared::paths;

use crate::config::{BonesConfig, Data, PermissionDefaults, Permissions, Releases, Ssl};

fn incomplete_seed(project_name: &str) -> BonesConfig {
    BonesConfig {
        data: Data {
            remote_name: String::from("production"),
            project_name: String::from(project_name),
            host: String::new(),
            port: String::from("22"),
            repo_path: String::new(),
            project_root: String::new(),
            web_root: String::new(),
            branch: String::from("main"),
            deploy_on_push: true,
        },
        permissions: Permissions {
            defaults: PermissionDefaults {
                deploy_user: String::from("git"),
                service_user: String::from(project_name),
                group: String::from("www-data"),
                dir_mode: String::from("750"),
                file_mode: String::from("640"),
            },
            paths: Vec::new(),
        },
        releases: Releases { keep: 5, shared_files: Vec::new(), shared_dirs: Vec::new() },
        ssl: Ssl::default(),
    }
}

/// Uses seed config and CLI values without prompting when non-interactive mode is active.
#[test]
fn collect_non_interactive_uses_seed_and_cli_values_without_prompting() -> Result<()> {
    let seed = incomplete_seed("atlas");
    let args = InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: None,
        branch: None,
        remote: None,
        host: Some(String::from("deploy.example.com")),
        port: None,
        template: None,
    };

    let cfg = collect_non_interactive("workspace", Some(&seed), &args)?;

    assert_eq!(cfg.data.project_name, "atlas");
    assert_eq!(cfg.data.host, "deploy.example.com");
    assert_eq!(cfg.data.branch, "main");
    assert_eq!(cfg.data.remote_name, "production");
    assert_eq!(cfg.data.repo_path, paths::default_repo_path_for("atlas"));

    Ok(())
}

/// Requires a host when neither seed config nor CLI provide one.
#[test]
fn collect_non_interactive_requires_host_when_seed_and_cli_are_missing_it() -> Result<()> {
    let seed = incomplete_seed("atlas");
    let args = InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: None,
        branch: None,
        remote: None,
        host: None,
        port: None,
        template: None,
    };

    let result = collect_non_interactive("workspace", Some(&seed), &args);
    let Err(err) = result else {
        bail!("missing host should fail");
    };
    assert!(err.to_string().contains("--host is required"));

    Ok(())
}
