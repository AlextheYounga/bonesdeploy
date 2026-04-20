use anyhow::Result;
use inquire::{Select, Text};

use crate::config::{BonesConfig, Data, PermissionDefaults, Permissions, Releases};

pub fn choose_template(available_templates: &[String]) -> Result<Option<String>> {
    let mut options = Vec::with_capacity(available_templates.len() + 1);
    options.push(String::from("Build from scratch"));
    options.extend(available_templates.iter().map(|name| format!("Use template: {name}")));

    let choice = Select::new("How would you like to initialize this project?", options)
        .with_help_message("Choose scratch for the current flow, or pick a template scaffold")
        .prompt()?;

    if choice == "Build from scratch" {
        return Ok(None);
    }

    let template_name = choice.strip_prefix("Use template: ").unwrap_or_default().to_string();

    if template_name.is_empty() {
        return Ok(None);
    }

    Ok(Some(template_name))
}

pub fn collect(project_name_hint: &str) -> Result<BonesConfig> {
    collect_from_seed(project_name_hint, None)
}

pub fn collect_from_seed(project_name_hint: &str, seed: Option<&BonesConfig>) -> Result<BonesConfig> {
    let default_remote_name =
        seed.map(|cfg| cfg.data.remote_name.as_str()).filter(|value| !value.is_empty()).unwrap_or("production");
    let remote_name = Text::new("Remote name:")
        .with_default(default_remote_name)
        .with_help_message("e.g. production, staging")
        .prompt()?;

    let default_project_name =
        seed.map(|cfg| cfg.data.project_name.as_str()).filter(|value| !value.is_empty()).unwrap_or(project_name_hint);
    let project_name = Text::new("Project name:").with_default(default_project_name).prompt()?;

    let default_host = seed.map(|cfg| cfg.data.host.as_str()).filter(|value| !value.is_empty()).unwrap_or("");
    let host = Text::new("Host:").with_default(default_host).with_help_message("e.g. deploy.example.com").prompt()?;

    let default_port = seed.map(|cfg| cfg.data.port.as_str()).filter(|value| !value.is_empty()).unwrap_or("22");
    let port = Text::new("Port:").with_default(default_port).prompt()?;

    let default_git_dir = seed.map(|cfg| cfg.data.git_dir.as_str()).filter(|value| !value.is_empty()).map_or_else(
        || format!("/home/git/{project_name}.git"),
        |value| value.replace("<project_name>", &project_name),
    );
    let git_dir = Text::new("Git directory (bare repo path on remote):").with_default(&default_git_dir).prompt()?;

    let default_live_root = seed
        .map(|cfg| cfg.data.live_root.as_str())
        .filter(|value| !value.is_empty())
        .map_or_else(|| format!("/var/www/{project_name}"), |value| value.replace("<project_name>", &project_name));
    let live_root = Text::new("Live root on remote:")
        .with_default(&default_live_root)
        .with_help_message("Public path your web server points at")
        .prompt()?;

    let default_deploy_root =
        seed.map(|cfg| cfg.data.deploy_root.as_str()).filter(|value| !value.is_empty()).map_or_else(
            || format!("/srv/deployments/{project_name}"),
            |value| value.replace("<project_name>", &project_name),
        );
    let deploy_root = Text::new("Deploy root on remote:")
        .with_default(&default_deploy_root)
        .with_help_message("Stores releases/, shared/, and current")
        .prompt()?;

    let default_branch = seed.map(|cfg| cfg.data.branch.as_str()).filter(|value| !value.is_empty()).unwrap_or("master");
    let branch = Text::new("Branch:").with_default(default_branch).prompt()?;

    let default_deploy_user =
        seed.map(|cfg| cfg.permissions.defaults.deploy.as_str()).filter(|value| !value.is_empty()).unwrap_or("git");
    let deploy_user = Text::new("Deploy user (SSH user):").with_default(default_deploy_user).prompt()?;

    let default_service_user = seed
        .map(|cfg| cfg.permissions.defaults.owner.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("applications");
    let service_user = Text::new("Service user (final file owner):").with_default(default_service_user).prompt()?;

    let default_service_group =
        seed.map(|cfg| cfg.permissions.defaults.group.as_str()).filter(|value| !value.is_empty()).unwrap_or("www-data");
    let service_group = Text::new("Service group:").with_default(default_service_group).prompt()?;

    let default_dir_mode =
        seed.map(|cfg| cfg.permissions.defaults.dir_mode.as_str()).filter(|value| !value.is_empty()).unwrap_or("750");
    let dir_mode = Text::new("Default directory mode:").with_default(default_dir_mode).prompt()?;

    let default_file_mode =
        seed.map(|cfg| cfg.permissions.defaults.file_mode.as_str()).filter(|value| !value.is_empty()).unwrap_or("640");
    let file_mode = Text::new("Default file mode:").with_default(default_file_mode).prompt()?;

    let default_releases_keep = seed.map(|cfg| cfg.releases.keep).filter(|value| *value > 0).unwrap_or(5).to_string();
    let releases_keep_raw = Text::new("Releases to keep:")
        .with_default(&default_releases_keep)
        .with_help_message("Old releases beyond this count are pruned")
        .prompt()?;
    let releases_keep = releases_keep_raw.parse::<usize>()?;

    let default_shared_paths = seed
        .map(|cfg| cfg.releases.shared_paths.join(", "))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| String::from(".env, storage"));
    let shared_paths_raw = Text::new("Shared paths (comma-separated):")
        .with_default(&default_shared_paths)
        .with_help_message("These paths are symlinked from deploy_root/shared")
        .prompt()?;
    let shared_paths = parse_shared_paths(&shared_paths_raw);

    let path_overrides = seed.map_or_else(Vec::new, |cfg| cfg.permissions.paths.clone());

    Ok(BonesConfig {
        data: Data { remote_name, project_name, host, port, git_dir, live_root, deploy_root, branch },
        permissions: Permissions {
            defaults: PermissionDefaults {
                deploy: deploy_user,
                owner: service_user,
                group: service_group,
                dir_mode,
                file_mode,
            },
            paths: path_overrides,
        },
        releases: Releases { keep: releases_keep, shared_paths },
    })
}

fn parse_shared_paths(raw: &str) -> Vec<String> {
    raw.split(',').map(str::trim).filter(|path| !path.is_empty()).map(ToOwned::to_owned).collect()
}
