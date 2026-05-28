use std::env;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use anyhow::{Context, Result};
use console::style;

use crate::commands::remote_setup;
use crate::config;
use crate::embedded;
use crate::git;
use crate::prompts;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    // Extract scaffold to .bones/
    let bones_dir = Path::new(config::Constants::BONES_DIR);
    if bones_dir.exists() {
        println!(".bones/ already exists, skipping scaffold extraction.");
    } else {
        let available_templates = embedded::available_templates();
        let selected_template = prompts::choose_template(&available_templates)?;

        println!("Creating .bones/ scaffold...");
        embedded::scaffold(bones_dir)?;

        if let Some(template_name) = selected_template {
            embedded::scaffold_template(&template_name, bones_dir)?;
            println!("Applied template: {template_name}");
        } else {
            println!("Using build-from-scratch scaffold.");
        }
    }

    // Update .gitignore
    update_gitignore()?;

    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = load_or_collect_config(bones_yaml)?;

    // Save config
    config::save(&cfg, bones_yaml)?;
    println!("Saved config to {}", config::Constants::BONES_YAML);
    ensure_local_remote(&cfg)?;

    // Symlink pre-push hook
    symlink_pre_push()?;

    if prompts::confirm_remote_setup()? {
        remote_setup::run()?;
    } else {
        println!(
            "{} Run {} before your first deploy.",
            style("Next:").cyan().bold(),
            style("bonesdeploy remote setup").cyan()
        );
    }

    println!(
        "{} Run {} after setup to sync .bones/ to the remote.",
        style("Done!").green().bold(),
        style("bonesdeploy push").cyan()
    );

    Ok(())
}

fn collect(project_name_hint: &str) -> Result<config::BonesConfig> {
    collect_from_seed(project_name_hint, None)
}

fn collect_from_seed(project_name_hint: &str, seed: Option<&config::BonesConfig>) -> Result<config::BonesConfig> {
    let project_name = prompts::prompt_project_name(project_name_hint, seed)?;
    let branch = prompts::prompt_branch(seed)?;
    let remote_name = prompts::prompt_remote_name(seed)?;
    let inferred_remote =
        if git::remote_exists(&remote_name)? { git::infer_remote_connection_details(&remote_name)? } else { None };
    let host = prompts::prompt_host(seed, inferred_remote.as_ref())?;
    let port = prompts::prompt_port(seed, inferred_remote.as_ref())?;
    let repo_path = resolve_repo_path(&project_name, seed, inferred_remote.as_ref());
    let project_root =
        seed_path_override(seed, |cfg| &cfg.data.project_root, &project_name, config::default_project_root_for);
    let web_root = seed_string(seed, |cfg| &cfg.data.web_root, config::default_web_root().as_str());
    let deploy_on_push = seed.is_none_or(|cfg| cfg.data.deploy_on_push);
    let deploy_user = seed_string(seed, |cfg| &cfg.permissions.defaults.deploy_user, "git");
    let service_user = seed_string(seed, |cfg| &cfg.permissions.defaults.service_user, &project_name);
    let group = seed_string(seed, |cfg| &cfg.permissions.defaults.group, "www-data");
    let dir_mode = seed_string(seed, |cfg| &cfg.permissions.defaults.dir_mode, "750");
    let file_mode = seed_string(seed, |cfg| &cfg.permissions.defaults.file_mode, "640");
    let releases_keep = seed.map_or(5, |cfg| cfg.releases.keep.max(1));
    let shared_files = seed
        .map(|cfg| cfg.releases.shared_files.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| vec![String::from(".env")]);
    let shared_dirs = seed
        .map(|cfg| cfg.releases.shared_dirs.clone())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| vec![String::from("storage")]);
    let path_overrides = seed.map_or_else(Vec::new, |cfg| cfg.permissions.paths.clone());

    Ok(config::BonesConfig {
        data: config::Data {
            remote_name,
            project_name,
            host,
            port,
            repo_path,
            project_root,
            web_root,
            branch,
            deploy_on_push,
        },
        permissions: config::Permissions {
            defaults: config::PermissionDefaults { deploy_user, service_user, group, dir_mode, file_mode },
            paths: path_overrides,
        },
        releases: config::Releases { keep: releases_keep, shared_files, shared_dirs },
        ssl: seed.map_or_else(config::Ssl::default, |cfg| cfg.ssl.clone()),
    })
}

fn seed_string(
    seed: Option<&config::BonesConfig>,
    field: impl Fn(&config::BonesConfig) -> &String,
    fallback: &str,
) -> String {
    seed.map(field).filter(|value| !value.is_empty()).map_or_else(|| fallback.to_string(), Clone::clone)
}

fn resolve_repo_path(
    project_name: &str,
    seed: Option<&config::BonesConfig>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> String {
    if let Some(details) = inferred_remote {
        return details.repo_path.clone();
    }

    seed.map(|cfg| cfg.data.repo_path.as_str())
        .filter(|value| !value.is_empty())
        .map_or_else(|| format!("/home/git/{project_name}.git"), |value| value.replace("<project_name>", project_name))
}

// Returns the seed's path override only when it differs from the project-derived
// default at the time the seed was loaded. Empty result means "no override" —
// save() will then omit the field from bones.yaml.
fn seed_path_override(
    seed: Option<&config::BonesConfig>,
    field: impl Fn(&config::BonesConfig) -> &String,
    current_project_name: &str,
    default_for: fn(&str) -> String,
) -> String {
    let Some(cfg) = seed else { return String::new() };
    let value = field(cfg);
    if value.is_empty() {
        return String::new();
    }

    let resolved = value.replace("<project_name>", current_project_name);
    if resolved == default_for(&cfg.data.project_name) || resolved == default_for(current_project_name) {
        return String::new();
    }
    resolved
}

fn load_or_collect_config(bones_yaml: &Path) -> Result<config::BonesConfig> {
    if bones_yaml.exists() {
        let existing = config::load(bones_yaml)?;
        if config::is_configured(&existing) {
            println!("Loading existing config from {}...", config::Constants::BONES_YAML);
            return Ok(existing);
        }
        println!("Config is incomplete, running prompts...");
        let project_name = repo_directory_name()?;
        return collect_from_seed(&project_name, Some(&existing));
    }
    let project_name = repo_directory_name()?;
    collect(&project_name)
}

fn update_gitignore() -> Result<()> {
    let gitignore = Path::new(".gitignore");
    let entry = config::Constants::BONES_DIR;

    if gitignore.exists() {
        let content = fs::read_to_string(gitignore)?;
        if content.lines().any(|line| line.trim() == entry) {
            return Ok(());
        }
        let separator = if content.ends_with('\n') { "" } else { "\n" };
        fs::write(gitignore, format!("{content}{separator}{entry}\n"))?;
    } else {
        fs::write(gitignore, format!("{entry}\n"))?;
    }

    println!("Added .bones to .gitignore");
    Ok(())
}

pub(crate) fn symlink_pre_push() -> Result<()> {
    let hooks_dir = Path::new(config::Constants::GIT_HOOKS_DIR);
    fs::create_dir_all(hooks_dir)?;

    let link = hooks_dir.join(config::Constants::PRE_PUSH_HOOK);
    let target = Path::new(config::Constants::PRE_PUSH_HOOK_TARGET);

    if link.exists() || link.symlink_metadata().is_ok() {
        fs::remove_file(&link).with_context(|| format!("Failed to remove existing {}", link.display()))?;
    }

    unix_fs::symlink(target, &link).with_context(|| format!("Failed to symlink {}", link.display()))?;

    println!("Symlinked {} -> {}", config::Constants::GIT_PRE_PUSH_HOOK_PATH, config::Constants::PRE_PUSH_HOOK_TARGET);
    Ok(())
}

fn repo_directory_name() -> Result<String> {
    let cwd = env::current_dir()?;
    let name = cwd.file_name().map_or_else(|| "project".into(), |n| n.to_string_lossy().to_string());
    Ok(name)
}

fn ensure_local_remote(cfg: &config::BonesConfig) -> Result<()> {
    if git::remote_exists(&cfg.data.remote_name)? {
        return Ok(());
    }

    let remote_url = format!("{}@{}:{}", cfg.permissions.defaults.deploy_user, cfg.data.host, cfg.data.repo_path);
    git::add_remote(&cfg.data.remote_name, &remote_url)?;
    println!("Configured local git remote {} -> {}", cfg.data.remote_name, remote_url);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    const TEMPLATES: [&str; 7] = [
        "templates/django/bones.yaml",
        "templates/laravel/bones.yaml",
        "templates/next/bones.yaml",
        "templates/nuxt/bones.yaml",
        "templates/rails/bones.yaml",
        "templates/sveltekit/bones.yaml",
        "templates/vue/bones.yaml",
    ];
    const TEMPLATE_SETUP_PLAYBOOKS: [&str; 7] = [
        "templates/django/remote/playbooks/setup.yml",
        "templates/laravel/remote/playbooks/setup.yml",
        "templates/next/remote/playbooks/setup.yml",
        "templates/nuxt/remote/playbooks/setup.yml",
        "templates/rails/remote/playbooks/setup.yml",
        "templates/sveltekit/remote/playbooks/setup.yml",
        "templates/vue/remote/playbooks/setup.yml",
    ];

    #[test]
    fn template_service_user_defaults_to_project_name_not_applications() {
        for template in TEMPLATES {
            let content = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(template));
            assert!(content.is_ok(), "failed to read {template}");
            let content = content.unwrap_or_default();

            assert!(
                !content.contains("service_user: 'applications'"),
                "template {template} still hardcodes applications as the service user\n{content}"
            );
        }
    }

    #[test]
    fn remote_setup_playbook_includes_apparmor_role() {
        let playbook = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/playbooks/setup.yml");
        let content = fs::read_to_string(&playbook);
        assert!(content.is_ok(), "failed to read {}", playbook.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("- role: apparmor"),
            "remote setup playbook must include apparmor role before runtime service provisioning\n{content}"
        );
    }

    #[test]
    fn nginx_service_template_sets_apparmor_profile() {
        let service_template =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/nginx/site-nginx.service.j2");
        let content = fs::read_to_string(&service_template);
        assert!(content.is_ok(), "failed to read {}", service_template.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("AppArmorProfile="),
            "per-site systemd service must pin an AppArmor profile\n{content}"
        );
    }

    #[test]
    fn nginx_service_template_waits_for_apparmor_service() {
        let service_template =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/nginx/site-nginx.service.j2");
        let content = fs::read_to_string(&service_template);
        assert!(content.is_ok(), "failed to read {}", service_template.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("After=network.target apparmor.service"),
            "per-site systemd service must start after apparmor.service\n{content}"
        );
        assert!(
            content.contains("Requires=apparmor.service"),
            "per-site systemd service must require apparmor.service\n{content}"
        );
    }

    #[test]
    fn apparmor_profile_template_exists() {
        let profile_template =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
        assert!(profile_template.exists(), "expected AppArmor profile template at {}", profile_template.display());
    }

    #[test]
    fn apparmor_profile_template_allows_repo_nginx_conf() {
        let profile_template =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
        let content = fs::read_to_string(&profile_template);
        assert!(content.is_ok(), "failed to read {}", profile_template.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("{{ repo_path }}/bones/nginx.conf r,"),
            "AppArmor template must allow reading repo-local nginx.conf used by bonesremote landlock nginx\n{content}"
        );
    }

    #[test]
    fn apparmor_profile_template_does_not_deny_repo_path_parent_home() {
        let profile_template =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
        let content = fs::read_to_string(&profile_template);
        assert!(content.is_ok(), "failed to read {}", profile_template.display());
        let content = content.unwrap_or_default();

        assert!(
            !content.contains("deny /home/** r,"),
            "AppArmor template must not deny all /home reads because default repo_path lives under /home/git\n{content}"
        );
        assert!(
            !content.contains("deny /home/{{ deploy_user }}/** r,"),
            "AppArmor template must not deny deploy user home globally because repo_path defaults under that path\n{content}"
        );
    }

    #[test]
    fn apparmor_profile_template_limits_network_to_unix_stream() {
        let profile_template =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/apparmor/project-nginx-profile.j2");
        let content = fs::read_to_string(&profile_template);
        assert!(content.is_ok(), "failed to read {}", profile_template.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("network unix stream,"),
            "AppArmor template must permit unix stream sockets for per-site nginx\n{content}"
        );
        assert!(
            !content.contains("network inet stream,"),
            "AppArmor template should not permit inet stream by default for unix-socket based per-site nginx\n{content}"
        );
        assert!(
            !content.contains("network inet6 stream,"),
            "AppArmor template should not permit inet6 stream by default for unix-socket based per-site nginx\n{content}"
        );
    }

    #[test]
    fn template_playbooks_include_apparmor_role() {
        for playbook in TEMPLATE_SETUP_PLAYBOOKS {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(playbook);
            let content = fs::read_to_string(&path);
            assert!(content.is_ok(), "failed to read {}", path.display());
            let content = content.unwrap_or_default();

            assert!(
                content.contains("- role: apparmor"),
                "template playbook {playbook} must include apparmor role\n{content}"
            );
        }
    }

    #[test]
    fn playbooks_apply_apparmor_before_nginx_role() {
        let mut playbooks = Vec::from(TEMPLATE_SETUP_PLAYBOOKS);
        playbooks.push("kit/remote/playbooks/setup.yml");

        for playbook in playbooks {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(playbook);
            let content = fs::read_to_string(&path);
            assert!(content.is_ok(), "failed to read {}", path.display());
            let content = content.unwrap_or_default();

            let apparmor_idx = content.find("- role: apparmor");
            let nginx_idx = content.find("- role: nginx");

            assert!(apparmor_idx.is_some(), "playbook {playbook} must include apparmor role\n{content}");
            assert!(nginx_idx.is_some(), "playbook {playbook} must include nginx role\n{content}");
            assert!(
                apparmor_idx < nginx_idx,
                "playbook {playbook} must apply apparmor role before nginx role\n{content}"
            );
        }
    }

    #[test]
    fn apparmor_role_assets_exist() {
        let role_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor");

        assert!(role_root.join("tasks/main.yml").is_file(), "missing apparmor role tasks/main.yml");
        assert!(role_root.join("defaults/main.yml").is_file(), "missing apparmor role defaults/main.yml");
        assert!(role_root.join("handlers/main.yml").is_file(), "missing apparmor role handlers/main.yml");
        assert!(role_root.join("README.md").is_file(), "missing apparmor role README.md");
    }

    #[test]
    fn apparmor_role_enforces_project_profile() {
        let tasks_file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
        let content = fs::read_to_string(&tasks_file);
        assert!(content.is_ok(), "failed to read {}", tasks_file.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("aa-enforce"),
            "apparmor role must explicitly set project profile to enforce mode\n{content}"
        );
    }

    #[test]
    fn apparmor_role_verifies_profile_loaded() {
        let tasks_file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
        let content = fs::read_to_string(&tasks_file);
        assert!(content.is_ok(), "failed to read {}", tasks_file.display());
        let content = content.unwrap_or_default();

        assert!(content.contains("aa-status"), "apparmor role must check loaded profiles via aa-status\n{content}");
        assert!(
            content.contains("apparmor_profile_name"),
            "apparmor role must verify the expected project profile name is present\n{content}"
        );
    }

    #[test]
    fn apparmor_role_verifies_profile_enforce_mode() {
        let tasks_file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
        let content = fs::read_to_string(&tasks_file);
        assert!(content.is_ok(), "failed to read {}", tasks_file.display());
        let content = content.unwrap_or_default();

        assert!(
            !content.contains("--profiled"),
            "apparmor role must not rely on unsupported aa-status --profiled\n{content}"
        );
        assert!(
            content.contains("profiles are in enforce mode")
                && content.contains("apparmor_profile_name | regex_escape"),
            "apparmor role must verify the target profile appears in the enforce-mode section of aa-status output\n{content}"
        );
    }

    #[test]
    fn apparmor_role_verifies_kernel_enabled() {
        let tasks_file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/tasks/main.yml");
        let content = fs::read_to_string(&tasks_file);
        assert!(content.is_ok(), "failed to read {}", tasks_file.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("/sys/module/apparmor/parameters/enabled"),
            "apparmor role must verify kernel apparmor enabled parameter\n{content}"
        );
        assert!(
            content.contains("in ['y', 'yes', '1']"),
            "apparmor role must assert enabled value is affirmative\n{content}"
        );
        assert!(
            content.contains("| trim | lower"),
            "apparmor role kernel-enabled assertion must trim aa parameter output before comparison\n{content}"
        );
    }

    #[test]
    fn remote_setup_doc_includes_apparmor_linux_verification_runbook() {
        let doc_file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("docs/commands/bonesdeploy/remote-setup.md");
        let content = fs::read_to_string(&doc_file);
        assert!(content.is_ok(), "failed to read {}", doc_file.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("AppArmor Verification Runbook (Linux Host)"),
            "remote setup docs must include an AppArmor Linux verification runbook section\n{content}"
        );
        assert!(
            content.contains("profiles are in enforce mode") && content.contains("grep 'bonesdeploy-<project>-nginx'"),
            "runbook must include a compatible profile-specific AppArmor enforce verification command\n{content}"
        );
        assert!(
            content.contains("systemctl cat <project>-nginx.service"),
            "runbook must include systemd service AppArmor binding verification\n{content}"
        );
    }

    #[test]
    fn goal_doc_tracks_apparmor_provisioning_and_linux_validation_gap() {
        let goal_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("docs/goal.md");
        let content = fs::read_to_string(&goal_file);
        assert!(content.is_ok(), "failed to read {}", goal_file.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("AppArmor Ansible provisioning now exists"),
            "goal doc should reflect implemented AppArmor provisioning\n{content}"
        );
        assert!(
            content.contains("Run Linux validation"),
            "goal doc should keep Linux validation as the remaining execution gap\n{content}"
        );
        assert!(
            !content.contains("Repo scan indicates no explicit AppArmor provisioning tasks/templates found yet."),
            "goal doc should not claim AppArmor provisioning is absent now that it is implemented\n{content}"
        );
    }

    #[test]
    fn apparmor_role_readme_includes_linux_verification_commands() {
        let readme_file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("kit/remote/roles/apparmor/README.md");
        let content = fs::read_to_string(&readme_file);
        assert!(content.is_ok(), "failed to read {}", readme_file.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("cat /sys/module/apparmor/parameters/enabled"),
            "apparmor role readme must include kernel enabled verification command\n{content}"
        );
        assert!(
            content.contains("profiles are in enforce mode") && content.contains("grep 'bonesdeploy-<project>-nginx'"),
            "apparmor role readme must include a compatible profile enforce verification command\n{content}"
        );
        assert!(
            content.contains("systemctl cat <project>-nginx.service"),
            "apparmor role readme must include service binding verification command\n{content}"
        );
    }

    #[test]
    fn landlock_nginx_doc_matches_current_policy_shape() {
        let doc_file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("docs/commands/bonesremote/landlock-nginx.md");
        let content = fs::read_to_string(&doc_file);
        assert!(content.is_ok(), "failed to read {}", doc_file.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("build_policy(&active_web_root, &socket_dir, &nginx_conf)"),
            "landlock-nginx docs should reflect current build_policy signature with nginx_conf path\n{content}"
        );
        assert!(
            content.contains("{{ repo_path }}/bones/nginx.conf") || content.contains("{repo_path}/bones/nginx.conf"),
            "landlock-nginx docs should describe repo-local nginx.conf as an allowed read path\n{content}"
        );
    }

    #[test]
    fn security_audit_checklist_includes_apparmor_service_binding_checks() {
        let checklist_file =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("docs/security/19-agent-audit-checklist.md");
        let content = fs::read_to_string(&checklist_file);
        assert!(content.is_ok(), "failed to read {}", checklist_file.display());
        let content = content.unwrap_or_default();

        assert!(
            content.contains("profiles are in enforce mode") && content.contains("grep 'bonesdeploy-<project>-nginx'"),
            "security audit checklist must include a compatible profile-specific apparmor status command\n{content}"
        );
        assert!(
            content.contains("systemctl cat <service>"),
            "security audit checklist must include service unit inspection for apparmor binding\n{content}"
        );
        assert!(
            content.contains("AppArmorProfile"),
            "security audit checklist should explicitly check AppArmorProfile binding in service units\n{content}"
        );
    }
}
