use std::fs;

use super::project_root;

/// Uses resolved placeholder web root paths in the common role.
#[test]
fn shared_setup_playbook_uses_placeholder_web_root_paths() {
    let playbook = project_root().join("kit/.lib/remote/roles/common/tasks/main.yml");
    let content = fs::read_to_string(&playbook);
    assert!(content.is_ok(), "failed to read {}", playbook.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.placeholder_web_root }}"),
        "common role must seed placeholder release using the resolved placeholder web root\n{content}"
    );
    assert!(
        content.contains("{{ paths.placeholder_index }}"),
        "common role must write placeholder index through the resolved path manifest\n{content}"
    );
}

/// Uses the resolved current web root for certbot validation in the SSL role.
#[test]
fn ssl_role_uses_current_web_root_path_manifest() {
    let role = project_root().join("kit/.lib/remote/roles/ssl/tasks/main.yml");
    let content = fs::read_to_string(&role);
    assert!(content.is_ok(), "failed to read {}", role.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("{{ paths.current_web_root }}"),
        "ssl role must use the resolved current web root for certbot webroot validation\n{content}"
    );
}

/// Uses resolved paths in both router nginx and `AppArmor` templates.
#[test]
fn nginx_and_apparmor_templates_use_resolved_paths() {
    let nginx_site = project_root().join("kit/.lib/remote/nginx/router.conf.j2");
    let nginx_conf = fs::read_to_string(&nginx_site);
    assert!(nginx_conf.is_ok(), "failed to read {}", nginx_site.display());
    let nginx_conf = nginx_conf.unwrap_or_default();

    assert!(
        nginx_conf.contains("{{ paths.runtime_nginx_socket }}"),
        "nginx router template must use the resolved runtime socket path\n{nginx_conf}"
    );
    assert!(
        !nginx_conf.contains("default_server"),
        "nginx router template must not claim the default server on shared hosts\n{nginx_conf}"
    );

    let apparmor = project_root().join("kit/.lib/remote/apparmor/project-nginx-profile.j2");
    let apparmor_conf = fs::read_to_string(&apparmor);
    assert!(apparmor_conf.is_ok(), "failed to read {}", apparmor.display());
    let apparmor_conf = apparmor_conf.unwrap_or_default();

    assert!(
        apparmor_conf.contains("{{ paths.current_web_root }}/** r,"),
        "AppArmor profile must read the resolved current web root path\n{apparmor_conf}"
    );
    assert!(
        apparmor_conf.contains("{{ paths.repo_bones_yaml }} r,"),
        "AppArmor profile must read the resolved repo bones yaml path\n{apparmor_conf}"
    );
    assert!(
        apparmor_conf.contains("{{ paths.site_nginx_config }} r,"),
        "AppArmor profile must read the resolved site nginx config path\n{apparmor_conf}"
    );
}

/// Treats SSL enabled as an explicit boolean rather than relying on string truthiness.
#[test]
fn ssl_role_treats_ssl_enabled_as_explicit_boolean() {
    let role = project_root().join("kit/.lib/remote/roles/ssl/tasks/main.yml");
    let content = fs::read_to_string(&role);
    assert!(content.is_ok(), "failed to read {}", role.display());
    let content = content.unwrap_or_default();

    assert!(!content.contains("when: ssl_enabled\n"), "ssl role must not rely on string truthiness\n{content}");
    assert!(content.contains("when: ssl_enabled | bool"), "ssl role should cast CLI vars explicitly\n{content}");
}

/// Defines router template and service defaults to allow tag-based execution without the nginx role.
#[test]
fn ssl_role_defines_nginx_defaults_for_tag_based_execution() {
    let defaults = project_root().join("kit/.lib/remote/roles/ssl/defaults/main.yml");
    let content = fs::read_to_string(&defaults);
    assert!(content.is_ok(), "failed to read {}", defaults.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("nginx_site_template_path:"),
        "ssl role must define nginx_site_template_path for self-contained tag execution\n{content}"
    );
    assert!(
        content.contains("../nginx/router.conf.j2"),
        "ssl role must default to the router nginx template for self-contained tag execution\n{content}"
    );
    assert!(
        content.contains("nginx_service_name:"),
        "ssl role must define nginx_service_name for self-contained tag execution\n{content}"
    );
}
