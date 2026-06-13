use std::fs;

use std::path::Path;

use super::project_root;

/// Sets an AppArmor profile in the per-site nginx systemd service template.
#[test]
fn nginx_service_template_sets_apparmor_profile() {
    let service_template = project_root().join("infra/assets/nginx/site-nginx.service.j2");
    let content = fs::read_to_string(&service_template);
    assert!(content.is_ok(), "failed to read {}", service_template.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("AppArmorProfile="), "per-site systemd service must pin an AppArmor profile\n{content}");
}

/// Requires the AppArmor service in the nginx systemd service template.
#[test]
fn nginx_service_template_waits_for_apparmor_service() {
    let service_template = project_root().join("infra/assets/nginx/site-nginx.service.j2");
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

/// Ensures the AppArmor profile template file exists at the expected path.
#[test]
fn apparmor_profile_template_exists() {
    let profile_template = project_root().join("infra/assets/apparmor/project-nginx-profile.j2");
    assert!(profile_template.exists(), "expected AppArmor profile template at {}", profile_template.display());
}

/// Allows reading the site nginx configuration in the AppArmor profile template.
#[test]
fn apparmor_profile_template_allows_site_nginx_conf() {
    let profile_template = project_root().join("infra/assets/apparmor/project-nginx-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.site_nginx_config }} r,"),
        "AppArmor template must allow reading site nginx.conf used by the per-site nginx service\n{content}"
    );
}

/// Allows the resolved release web root in the `AppArmor` profile template.
#[test]
fn apparmor_profile_template_allows_resolved_release_web_root() {
    let profile_template = project_root().join("infra/assets/apparmor/project-nginx-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.current_web_root }}/** r,"),
        "AppArmor template must still allow the resolved current web root\n{content}"
    );
    assert!(
        content.contains("{{ paths.releases }}/*/{{ web_root }}/** r,"),
        "AppArmor template must allow the resolved release web root because current is a symlink\n{content}"
    );
}

/// Allows reading the per-site conf root in the Laravel PHP-FPM AppArmor profile.
#[test]
fn laravel_php_fpm_apparmor_profile_allows_site_conf_root() {
    let profile_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("embeds/runtimes/laravel/infra/assets/php/site-php-fpm-profile.j2");
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

/// Grants only the minimal capabilities needed by the Laravel PHP-FPM `AppArmor` profile.
#[test]
fn laravel_php_fpm_apparmor_profile_grants_minimal_capabilities() {
    let profile_template =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("embeds/runtimes/laravel/infra/assets/php/site-php-fpm-profile.j2");
    let content = fs::read_to_string(&profile_template);
    assert!(content.is_ok(), "failed to read {}", profile_template.display());
    let content = content.unwrap_or_default();

    assert!(content.contains("capability chown,"), "Laravel PHP-FPM AppArmor profile must allow chown\n{content}");
    assert!(content.contains("capability setgid,"), "Laravel PHP-FPM AppArmor profile must allow setgid\n{content}");
    assert!(content.contains("capability setuid,"), "Laravel PHP-FPM AppArmor profile must allow setuid\n{content}");
    assert!(
        !content.contains("capability dac_override,"),
        "Laravel PHP-FPM AppArmor profile must not allow DAC override\n{content}"
    );
    assert!(
        !content.contains("capability dac_read_search,"),
        "Laravel PHP-FPM AppArmor profile must not allow DAC read search\n{content}"
    );
    assert!(
        !content.contains("capability fsetid,"),
        "Laravel PHP-FPM AppArmor profile must not allow fsetid\n{content}"
    );
    assert!(
        !content.lines().any(|line| line.trim() == "/ rw,"),
        "Laravel PHP-FPM AppArmor profile must not allow root filesystem read-write\n{content}"
    );
}

/// Does not deny the parent home path when the repo path is derived from the shared helper.
#[test]
fn apparmor_profile_template_does_not_deny_repo_path_parent_home() {
    let profile_template = project_root().join("infra/assets/apparmor/project-nginx-profile.j2");
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
    let profile_template = project_root().join("infra/assets/apparmor/project-nginx-profile.j2");
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

/// Verifies apparmor profile enforcement is handled by the runtime deploy script.
#[test]
fn runtime_deploy_enforces_apparmor_profile() {
    let deploy = project_root().join("infra/runtime.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("aa-enforce"),
        "runtime deploy must explicitly set project profile to enforce mode\n{content}"
    );
}

/// Verifies AppArmor profile loading is handled by the runtime deploy script.
#[test]
fn runtime_deploy_loads_apparmor_profile() {
    let deploy = project_root().join("infra/runtime.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("apparmor_parser -r"),
        "runtime deploy must load the per-project apparmor profile\n{content}"
    );
}

/// Verifies AppArmor kernel enabled check is in the runtime deploy script.
#[test]
fn runtime_deploy_verifies_kernel_enabled() {
    let deploy = project_root().join("infra/runtime.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("apparmor_enabled_param"),
        "runtime deploy must verify kernel apparmor enabled parameter\n{content}"
    );
}
