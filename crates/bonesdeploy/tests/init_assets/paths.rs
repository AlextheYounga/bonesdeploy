use std::fs;

use super::project_root;

/// Uses resolved placeholder web root paths in the setup deploy script.
#[test]
fn setup_deploy_uses_placeholder_web_root_paths() {
    let deploy = project_root().join("infra/src/setup.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("placeholder_web_root"),
        "setup deploy must seed placeholder release using the resolved placeholder web root\n{content}"
    );
    assert!(
        content.contains("placeholder_index"),
        "setup deploy must write placeholder index through the resolved path manifest\n{content}"
    );
}

/// Uses the resolved current web root for certbot validation in the SSL deploy.
#[test]
fn ssl_deploy_uses_current_web_root_path_manifest() {
    let deploy = project_root().join("infra/src/ssl.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("current_web_root"),
        "ssl deploy must use the resolved current web root for certbot webroot validation\n{content}"
    );
}

/// Uses resolved paths in both router nginx and AppArmor templates.
#[test]
fn nginx_and_apparmor_templates_use_resolved_paths() {
    let nginx_site = project_root().join("infra/assets/nginx/router.conf.j2");
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

    let apparmor = project_root().join("infra/assets/apparmor/project-nginx-profile.j2");
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

/// Defines router template and nginx_defaults in the SSL deploy as self-contained deployment.
#[test]
fn ssl_deploy_defines_nginx_defaults_inline() {
    let deploy = project_root().join("infra/src/ssl.py");
    let content = fs::read_to_string(&deploy);
    assert!(content.is_ok(), "failed to read {}", deploy.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("nginx_server_name") && content.contains("router.conf.j2"),
        "ssl deploy must reference the router nginx template for self-contained deployment\n{content}"
    );
    assert!(content.contains("nginx -t"), "ssl deploy must validate nginx configuration\n{content}");
}

/// Build logs directory is centralized under project_root/build/logs via the resolved path manifest
/// so the deploy runner can persist per-script logs without re-deriving the location.
#[test]
fn build_logs_path_uses_centralized_manifest() {
    let shared = project_root().join("crates/shared/src/paths.rs");
    let content = fs::read_to_string(&shared);
    assert!(content.is_ok(), "failed to read {}", shared.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("pub const LOGS_DIR: &str = \"logs\";"),
        "paths.rs must declare LOGS_DIR as a centralized constant\n{content}"
    );
    assert!(
        content.contains("pub build_logs: String,"),
        "DeploymentPaths must expose build_logs for the centralized logs directory\n{content}"
    );
    assert!(
        content.contains("build_logs: Path::new(&project_root).join(BUILD_DIR).join(LOGS_DIR).display().to_string()"),
        "build_logs must be derived from project_root/build/logs through centralized constants\n{content}"
    );
}

/// Base per-site nginx config writes error/access logs under the runtime socket directory
/// so that the non-root service user can write them under the systemd sandbox and AppArmor profile.
/// Relative "stderr" paths resolve to unwritable locations like /usr/share/nginx/stderr.
#[test]
fn base_site_nginx_config_writes_logs_under_runtime_socket_dir() {
    let nginx_config = project_root().join("infra/assets/nginx/site-nginx.conf.j2");
    let content = fs::read_to_string(&nginx_config);
    assert!(content.is_ok(), "failed to read {}", nginx_config.display());
    let content = content.unwrap_or_default();

    assert!(
        content.contains("error_log {{ paths.runtime_socket_dir }}/error.log"),
        "base per-site nginx config must write error log under the runtime socket directory\n{content}"
    );
    assert!(
        content.contains("access_log {{ paths.runtime_socket_dir }}/access.log"),
        "base per-site nginx config must write access log under the runtime socket directory\n{content}"
    );
    assert!(
        !content.contains("access_log stderr"),
        "base per-site nginx config must not use relative stderr access log (non-root cannot write /usr/share/nginx)\n{content}"
    );
}
