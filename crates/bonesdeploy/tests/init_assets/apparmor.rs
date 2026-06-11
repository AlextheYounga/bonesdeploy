use std::fs;

use super::project_root;

/// Sets an `AppArmor` profile in the per-site nginx systemd service template.
#[test]
fn nginx_service_template_sets_apparmor_profile() {
    let service_template = project_root().join("kit/setup/nginx/site-nginx.service.j2");
    let content = fs::read_to_string(&service_template);
    assert!(content.is_ok(), "failed to read {}", service_template.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("AppArmorProfile="), "per-site systemd service must pin an AppArmor profile\n{content}");
}

/// Requires the `AppArmor` service in the nginx systemd service template.
#[test]
fn nginx_service_template_waits_for_apparmor_service() {
    let service_template = project_root().join("kit/setup/nginx/site-nginx.service.j2");
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

/// Ensures the `AppArmor` profile template file exists at the expected path.
#[test]
fn apparmor_profile_template_exists() {
    let profile_template = project_root().join("kit/setup/apparmor/project-nginx-profile.j2");
    assert!(profile_template.exists(), "expected AppArmor profile template at {}", profile_template.display());
}

/// Allows reading the site nginx configuration in the `AppArmor` profile template.
#[test]
fn apparmor_profile_template_allows_site_nginx_conf() {
    let profile_template = project_root().join("kit/setup/apparmor/project-nginx-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.site_nginx_config }} r,"),
        "AppArmor template must allow reading site nginx.conf used by the per-site nginx service\n{content}"
    );
}

/// Allows reading the per-site conf root in the Laravel PHP-FPM AppArmor profile.
#[test]
fn laravel_php_fpm_apparmor_profile_allows_site_conf_root() {
    let profile_template =
        project_root().join("templates/laravel/setup/roles/laravel_runtime/templates/site-php-fpm-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.conf_root }}/ r,"),
        "Laravel PHP-FPM AppArmor profile must allow reading the site config directory itself\n{content}"
    );
    assert!(
        content.contains("{{ paths.conf_root }}/** r,"),
        "Laravel PHP-FPM AppArmor profile must allow reading files beneath the site config directory\n{content}"
    );
}

/// Does not deny the parent home path when the repo path is derived from the shared helper.
#[test]
fn apparmor_profile_template_does_not_deny_repo_path_parent_home() {
    let profile_template = project_root().join("kit/setup/apparmor/project-nginx-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        !content.contains("deny /home/** r,"),
        "AppArmor template must not deny all /home reads because default repo_path is derived from the shared helper\n{content}"
    );
    assert!(
        !content.contains("deny /home/{{ deploy_user }}/** r,"),
        "AppArmor template must not deny deploy user home globally because repo_path defaults under that path\n{content}"
    );
}

/// Limits network access to unix stream sockets and denies inet sockets.
#[test]
fn apparmor_profile_template_limits_network_to_unix_stream() {
    let profile_template = project_root().join("kit/setup/apparmor/project-nginx-profile.j2");
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

/// Ensures all expected `AppArmor` role asset files exist.
#[test]
fn apparmor_role_assets_exist() {
    let role_root = project_root().join("kit/setup/roles/apparmor");

    for file in ["tasks/main.yml", "defaults/main.yml", "handlers/main.yml", "README.md"] {
        assert!(role_root.join(file).is_file(), "missing apparmor role {file}");
    }
}

/// Enforces the project `AppArmor` profile into enforce mode.
#[test]
fn apparmor_role_enforces_project_profile() {
    let tasks_file = project_root().join("kit/setup/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("aa-enforce"),
        "apparmor role must explicitly set project profile to enforce mode\n{content}"
    );
}

/// Verifies the `AppArmor` profile is loaded in the kernel.
#[test]
fn apparmor_role_verifies_profile_loaded() {
    let tasks_file = project_root().join("kit/setup/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.apparmor_profiles }}"),
        "apparmor role must check loaded profiles via kernel apparmor profile list\n{content}"
    );
    assert!(content.contains("apparmor_profile_name"), "apparmor role must verify expected profile name\n{content}");
}

/// Verifies the `AppArmor` profile is in enforce mode via kernel output.
#[test]
fn apparmor_role_verifies_profile_enforce_mode() {
    let tasks_file = project_root().join("kit/setup/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.apparmor_profiles }}")
            && content.contains("\\(enforce\\)")
            && content.contains("apparmor_profile_name | regex_escape"),
        "apparmor role must verify enforce mode directly from kernel AppArmor profiles output\n{content}"
    );
}

/// Verifies `AppArmor` is enabled in the kernel parameters.
#[test]
fn apparmor_role_verifies_kernel_enabled() {
    let tasks_file = project_root().join("kit/setup/roles/apparmor/tasks/main.yml");
    let content = fs::read_to_string(&tasks_file);
    assert!(content.is_ok(), "failed to read {}", tasks_file.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.apparmor_enabled_param }}"),
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
