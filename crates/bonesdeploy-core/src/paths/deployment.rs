use std::path::Path;

use serde::Serialize;

use crate::state::service_unit;

use super::{
    APPARMOR_ENABLED_PARAM, APPARMOR_PROFILES, BACKUPS_ROOT, BONESDEPLOY_LOG_ROOT, BONESDEPLOY_NGINX_DEFAULT_DENY_CERT,
    BONESDEPLOY_NGINX_DEFAULT_DENY_KEY, BONESDEPLOY_NGINX_DEFAULT_DENY_SITE, BONESREMOTE_BINARY,
    BONESREMOTE_CONFIG_DIR, BONESREMOTE_SITES_DIR, BORG_PASSPHRASE_FILE, CURRENT_LINK, DEFAULT_CONF_ROOT_PARENT,
    DEFAULT_NGINX_SITE, DEFAULT_PROJECT_ROOT_PARENT, DEFAULT_WEB_ROOT, ETC_APPARMOR_D, ETC_NGINX_SITES_AVAILABLE,
    ETC_NGINX_SITES_ENABLED, ETC_SSL_CERTS, ETC_SSL_PRIVATE, ETC_SUDOERS_D, ETC_SYSTEMD_SYSTEM, INDEX_HTML, NGINX_CONF,
    NGINX_PID, NGINX_SOCKET, PHP_FPM_SOCKET, PLACEHOLDER_RELEASE_NAME, RELEASES_DIR, RUNTIME_SOCKET_PARENT, SHARED_DIR,
    USR_LOCAL_BIN,
};

/// Complete convention-derived path vocabulary used while provisioning a site.
///
/// Keeping this representation in the dependency-free core crate gives local
/// orchestration, provisioning, and remote deployment one authoritative model.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeploymentPaths {
    pub project_name: String,
    pub site_nginx_config: String,
    pub site_root: String,
    pub conf_root: String,
    pub project_root: String,
    pub project_root_parent: String,
    pub releases: String,
    pub shared: String,
    pub shared_env: String,
    pub current: String,
    pub current_web_root: String,
    pub placeholder_release: String,
    pub placeholder_web_root: String,
    pub placeholder_index: String,
    pub nginx_site_available: String,
    pub nginx_site_enabled: String,
    pub nginx_default_deny_site_available: String,
    pub nginx_default_deny_site_enabled: String,
    pub nginx_default_deny_ssl_certificate: String,
    pub nginx_default_deny_ssl_certificate_key: String,
    pub nginx_default_site_enabled: String,
    pub nginx_cloudflared_site_available: String,
    pub nginx_cloudflared_site_enabled: String,
    pub systemd_site_nginx_service: String,
    pub systemd_cloudflared_service: String,
    pub systemd_site_nginx_requirement: String,
    pub systemd_site_target: String,
    pub systemd_site_target_requires: String,
    pub nginx_apparmor_profile: String,
    pub runtime_socket_dir: String,
    pub runtime_nginx_dir: String,
    pub runtime_nginx_socket: String,
    pub runtime_nginx_pid: String,
    pub runtime_php_fpm_socket: String,
    pub site_log_dir: String,
    pub acme_webroot: String,
    pub backup_repository: String,
    pub backup_passphrase_file: String,
    pub backup_cron_file: String,
    pub sudoers_path: String,
    pub usr_local_bin: String,
    pub bonesremote_global_link: String,
    pub apparmor_enabled_param: String,
    pub apparmor_profiles: String,
}

impl DeploymentPaths {
    #[must_use]
    pub fn new(project_name: &str, project_root: &str, web_root: &str) -> Self {
        let project_root_path = Path::new(project_root);
        let conf_root = Path::new(DEFAULT_CONF_ROOT_PARENT).join(project_name);
        let releases = project_root_path.join(RELEASES_DIR);
        let shared = project_root_path.join(SHARED_DIR);
        let current = project_root_path.join(CURRENT_LINK);
        let placeholder_release = releases.join(PLACEHOLDER_RELEASE_NAME);
        let runtime_socket_dir = Path::new(RUNTIME_SOCKET_PARENT).join(project_name);
        let runtime_nginx_dir = runtime_socket_dir.join("nginx");
        let systemd_site_target_requires =
            Path::new(ETC_SYSTEMD_SYSTEM).join(format!("{project_name}.target.requires"));
        let cloudflared_name = format!("bonesdeploy-cloudflared-{project_name}.conf");

        Self {
            project_name: project_name.to_owned(),
            site_nginx_config: display(conf_root.join(NGINX_CONF)),
            site_root: display(Path::new(BONESREMOTE_CONFIG_DIR).join(BONESREMOTE_SITES_DIR).join(project_name)),
            conf_root: display(&conf_root),
            project_root: project_root.to_owned(),
            project_root_parent: project_root_path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map_or_else(|| DEFAULT_PROJECT_ROOT_PARENT.to_owned(), display),
            releases: display(&releases),
            shared_env: display(shared.join(super::DOT_ENV)),
            shared: display(&shared),
            current_web_root: display(current.join(web_root)),
            current: display(&current),
            placeholder_web_root: display(placeholder_release.join(web_root)),
            placeholder_index: display(placeholder_release.join(web_root).join(INDEX_HTML)),
            placeholder_release: display(&placeholder_release),
            nginx_site_available: display(Path::new(ETC_NGINX_SITES_AVAILABLE).join(format!("{project_name}.conf"))),
            nginx_site_enabled: display(Path::new(ETC_NGINX_SITES_ENABLED).join(format!("{project_name}.conf"))),
            nginx_default_deny_site_available: display(
                Path::new(ETC_NGINX_SITES_AVAILABLE).join(BONESDEPLOY_NGINX_DEFAULT_DENY_SITE),
            ),
            nginx_default_deny_site_enabled: display(
                Path::new(ETC_NGINX_SITES_ENABLED).join(BONESDEPLOY_NGINX_DEFAULT_DENY_SITE),
            ),
            nginx_default_deny_ssl_certificate: display(
                Path::new(ETC_SSL_CERTS).join(BONESDEPLOY_NGINX_DEFAULT_DENY_CERT),
            ),
            nginx_default_deny_ssl_certificate_key: display(
                Path::new(ETC_SSL_PRIVATE).join(BONESDEPLOY_NGINX_DEFAULT_DENY_KEY),
            ),
            nginx_default_site_enabled: display(Path::new(ETC_NGINX_SITES_ENABLED).join(DEFAULT_NGINX_SITE)),
            nginx_cloudflared_site_available: display(Path::new(ETC_NGINX_SITES_AVAILABLE).join(&cloudflared_name)),
            nginx_cloudflared_site_enabled: display(Path::new(ETC_NGINX_SITES_ENABLED).join(cloudflared_name)),
            systemd_site_nginx_service: display(
                Path::new(ETC_SYSTEMD_SYSTEM).join(format!("{project_name}-nginx.service")),
            ),
            systemd_cloudflared_service: display(
                Path::new(ETC_SYSTEMD_SYSTEM).join(format!("{project_name}-cloudflared.service")),
            ),
            systemd_site_nginx_requirement: display(
                systemd_site_target_requires.join(format!("{project_name}-nginx.service")),
            ),
            systemd_site_target: display(Path::new(ETC_SYSTEMD_SYSTEM).join(format!("{project_name}.target"))),
            systemd_site_target_requires: display(&systemd_site_target_requires),
            nginx_apparmor_profile: display(
                Path::new(ETC_APPARMOR_D).join(format!("bonesdeploy-{project_name}-nginx")),
            ),
            runtime_nginx_socket: display(runtime_nginx_dir.join(NGINX_SOCKET)),
            runtime_nginx_pid: display(runtime_nginx_dir.join(NGINX_PID)),
            runtime_nginx_dir: display(&runtime_nginx_dir),
            runtime_php_fpm_socket: display(runtime_socket_dir.join(PHP_FPM_SOCKET)),
            runtime_socket_dir: display(&runtime_socket_dir),
            site_log_dir: display(Path::new(BONESDEPLOY_LOG_ROOT).join(project_name)),
            acme_webroot: display(Path::new("/var/www").join(project_name)),
            backup_repository: display(Path::new(BACKUPS_ROOT).join(format!("{project_name}.borg"))),
            backup_passphrase_file: display(
                Path::new(BONESREMOTE_CONFIG_DIR)
                    .join(BONESREMOTE_SITES_DIR)
                    .join(project_name)
                    .join(BORG_PASSPHRASE_FILE),
            ),
            backup_cron_file: format!("/etc/cron.d/bonesdeploy-{project_name}-backup"),
            sudoers_path: display(Path::new(ETC_SUDOERS_D).join("bonesdeploy")),
            usr_local_bin: USR_LOCAL_BIN.to_owned(),
            bonesremote_global_link: display(Path::new(USR_LOCAL_BIN).join(BONESREMOTE_BINARY)),
            apparmor_enabled_param: APPARMOR_ENABLED_PARAM.to_owned(),
            apparmor_profiles: APPARMOR_PROFILES.to_owned(),
        }
    }

    #[must_use]
    pub fn systemd_service(&self, name: &str) -> String {
        display(Path::new(ETC_SYSTEMD_SYSTEM).join(service_unit(&self.project_name, name)))
    }

    #[must_use]
    pub fn systemd_service_requirement(&self, name: &str) -> String {
        display(Path::new(&self.systemd_site_target_requires).join(service_unit(&self.project_name, name)))
    }

    #[must_use]
    pub fn apparmor_profile(&self, name: &str) -> String {
        display(Path::new(ETC_APPARMOR_D).join(format!("bonesdeploy-{}-{name}", self.project_name)))
    }

    #[must_use]
    pub fn runtime_service_dir(&self, name: &str) -> String {
        display(Path::new(&self.runtime_socket_dir).join(name))
    }

    #[must_use]
    pub fn runtime_service_socket(&self, name: &str) -> String {
        display(Path::new(&self.runtime_service_dir(name)).join(format!("{name}.sock")))
    }
}

impl Default for DeploymentPaths {
    fn default() -> Self {
        Self::new("", DEFAULT_PROJECT_ROOT_PARENT, DEFAULT_WEB_ROOT)
    }
}

fn display(path: impl AsRef<Path>) -> String {
    path.as_ref().display().to_string()
}
