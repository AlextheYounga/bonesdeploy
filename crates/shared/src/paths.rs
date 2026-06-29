use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const DEFAULT_REPO_PARENT: &str = "/home/git";
pub const DEFAULT_PROJECT_ROOT_PARENT: &str = "/srv/sites";
pub const DEFAULT_CONF_ROOT_PARENT: &str = "/srv/conf";
pub const DEFAULT_WEB_ROOT: &str = "public";

pub const DEPLOY_USER: &str = "git";
pub const DEFAULT_GROUP: &str = "www-data";

pub const ETC_NGINX: &str = "/etc/nginx";
pub const ETC_NGINX_SITES_AVAILABLE: &str = "/etc/nginx/sites-available";
pub const ETC_NGINX_SITES_ENABLED: &str = "/etc/nginx/sites-enabled";
pub const ETC_SYSTEMD_SYSTEM: &str = "/etc/systemd/system";
pub const ETC_APPARMOR_D: &str = "/etc/apparmor.d";
pub const ETC_LETSENCRYPT_LIVE: &str = "/etc/letsencrypt/live";
pub const ETC_SUDOERS_D: &str = "/etc/sudoers.d";
pub const ETC_OS_RELEASE: &str = "/etc/os-release";
pub const ETC_PASSWD: &str = "/etc/passwd";
pub const ETC_GROUP: &str = "/etc/group";
pub const APPARMOR_ENABLED_PARAM: &str = "/sys/module/apparmor/parameters/enabled";
pub const APPARMOR_PROFILES: &str = "/sys/kernel/security/apparmor/profiles";
pub const PROC_MODULES: &str = "/proc/modules";
pub const ETC_MODPROBE_D: &str = "/etc/modprobe.d";
pub const USR_LOCAL_BIN: &str = "/usr/local/bin";
pub const OPT_BONESDEPLOY: &str = "/opt/bonesdeploy";
pub const TMP_ROOT: &str = "/tmp";

pub const LOCAL_BONES_DIR: &str = ".bones";
pub const LOCAL_BONES_TOML: &str = ".bones/bones.toml";
pub const LOCAL_BONES_HOOKS_DIR: &str = ".bones/hooks";
pub const LOCAL_BONES_DEPLOYMENT_DIR: &str = ".bones/deployment";
pub const LOCAL_BONES_RUNTIME_TOML: &str = ".bones/runtime.toml";
pub const LOCAL_BONES_SECRETS_DIR: &str = ".bones/secrets";
pub const DOT_ENV: &str = ".env";
pub const RUNTIME_TOML: &str = "runtime.toml";
pub const REGISTRY_TOML: &str = "registry.toml";

pub const BONES_DIR: &str = "bones";
pub const BONES_TOML: &str = "bones.toml";
pub const NGINX_CONF: &str = "nginx.conf";
pub const INDEX_HTML: &str = "index.html";
pub const GIT_HEAD: &str = "HEAD";
pub const DEPLOYMENT_DIR: &str = "deployment";
pub const DEPLOYMENT_BUILD_DIR: &str = "build";
pub const DEPLOYMENT_PREPARE_DIR: &str = "prepare";
pub const RELEASES_DIR: &str = "releases";
pub const SHARED_DIR: &str = "shared";
pub const BUILD_DIR: &str = "build";
pub const WORKSPACE_DIR: &str = "workspace";
pub const LOGS_DIR: &str = "logs";
pub const CURRENT_LINK: &str = "current";
pub const STAGED_RELEASE_FILE: &str = "staged-release";
pub const TMP_BUILDS_DIR: &str = "tmp";
pub const INSTALL_VERSIONS_DIR: &str = "versions";
pub const INSTALL_CURRENT_LINK: &str = "current";
pub const BONESDEPLOY_SWAP_LINK: &str = ".bonesdeploy_swap";
pub const BONESREMOTE_SWAP_LINK_PREFIX: &str = ".bonesremote_swap_";
pub const PLACEHOLDER_RELEASE_NAME: &str = "19700101_000000";
pub const SUDOERS_FILE: &str = "bonesdeploy";
pub const SUDOERS_PATH: &str = "/etc/sudoers.d/bonesdeploy";
pub const BONESDEPLOY_BINARY: &str = "bonesdeploy";
pub const BONESREMOTE_BINARY: &str = "bonesremote";
pub const BONESREMOTE_CONFIG_DIR: &str = "/root/.config/bonesremote";
pub const BONESREMOTE_SITES_DIR: &str = "sites";
pub const NGINX_SOCKET: &str = "nginx.sock";
pub const NGINX_PID: &str = "nginx.pid";
pub const PHP_FPM_SOCKET: &str = "php-fpm.sock";
pub const DEFAULT_NGINX_SITE: &str = "default";

pub const GIT_HOOKS_DIR: &str = ".git/hooks";
pub const GIT_PRE_PUSH_HOOK: &str = ".git/hooks/pre-push";
pub const PRE_PUSH_HOOK_NAME: &str = "pre-push";
pub const PRE_PUSH_HOOK_TARGET: &str = "../../.bones/hooks/pre-push";
pub const HOOKS_DIR: &str = "hooks";
pub const KIT_HOOKS_DIR: &str = "hooks/";
pub const KIT_DEPLOYMENT_DIR: &str = "deployment/";
pub const KIT_SECRETS_DIR: &str = "secrets/";

const RUNTIME_SOCKET_PARENT: &str = "/run";

#[must_use]
pub fn default_repo_path_for(project_name: &str) -> String {
    Path::new(DEFAULT_REPO_PARENT).join(format!("{project_name}.git")).display().to_string()
}

#[must_use]
pub fn default_project_root_for(project_name: &str) -> String {
    Path::new(DEFAULT_PROJECT_ROOT_PARENT).join(project_name).display().to_string()
}

#[must_use]
pub fn default_web_root() -> String {
    DEFAULT_WEB_ROOT.to_string()
}

#[must_use]
pub fn ssl_certificate_path(domain: &str) -> String {
    Path::new(ETC_LETSENCRYPT_LIVE).join(domain).join("fullchain.pem").display().to_string()
}

#[must_use]
pub fn ssl_certificate_key_path(domain: &str) -> String {
    Path::new(ETC_LETSENCRYPT_LIVE).join(domain).join("privkey.pem").display().to_string()
}

#[must_use]
pub fn nginx_service_name(project_name: &str) -> String {
    format!("{project_name}-nginx.service")
}

#[must_use]
pub fn bonesremote_staging_path(version: &str) -> String {
    Path::new(TMP_ROOT).join(format!("{BONESREMOTE_BINARY}-{version}")).display().to_string()
}

#[must_use]
pub fn install_root() -> PathBuf {
    PathBuf::from(OPT_BONESDEPLOY)
}

#[must_use]
pub fn bonesremote_config_root() -> PathBuf {
    PathBuf::from(BONESREMOTE_CONFIG_DIR)
}

#[must_use]
pub fn bonesremote_sites_root() -> PathBuf {
    bonesremote_config_root().join(BONESREMOTE_SITES_DIR)
}

#[must_use]
pub fn bonesremote_site_root(site: &str) -> PathBuf {
    bonesremote_sites_root().join(site)
}

#[must_use]
pub fn bonesremote_registry_path(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(REGISTRY_TOML)
}

#[must_use]
pub fn bonesremote_bones_toml_path(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(BONES_TOML)
}

#[must_use]
pub fn bonesremote_staged_release_path(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(STAGED_RELEASE_FILE)
}

#[must_use]
pub fn bonesremote_tmp_builds_root(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(TMP_BUILDS_DIR)
}

/// Canonical list of release-tree paths that should be linked from `shared/`.
pub const SHARED_LEAVES: &[&str] = &[DOT_ENV, "storage", "bootstrap/cache", "database/database.sqlite"];

#[must_use]
pub fn bonesremote_sites_root_resolved() -> PathBuf {
    if let Some(root) = env::var_os("BONESREMOTE_SITES_ROOT") {
        let raw = root.to_string_lossy().to_string();
        if !raw.trim().is_empty() {
            return PathBuf::from(raw);
        }
    }
    bonesremote_sites_root()
}

#[must_use]
pub fn install_versions_dir() -> PathBuf {
    install_root().join(INSTALL_VERSIONS_DIR)
}

#[must_use]
pub fn install_current_dir() -> PathBuf {
    install_root().join(INSTALL_CURRENT_LINK)
}

#[must_use]
pub fn bonesdeploy_global_link() -> PathBuf {
    Path::new(USR_LOCAL_BIN).join(BONESDEPLOY_BINARY)
}

#[must_use]
pub fn bonesremote_global_link() -> PathBuf {
    Path::new(USR_LOCAL_BIN).join(BONESREMOTE_BINARY)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deployment {
    pub repo: String,
    pub repo_parent: String,
    pub repo_head: String,
    pub repo_bones: String,
    pub repo_bones_toml: String,
    pub repo_deployment: String,
    pub site_nginx_config: String,
    pub conf_root: String,
    pub project_root: String,
    pub project_root_parent: String,
    pub releases: String,
    pub shared: String,
    pub build_root: String,
    pub build_logs: String,
    pub current: String,
    pub current_web_root: String,
    pub placeholder_release: String,
    pub placeholder_web_root: String,
    pub placeholder_index: String,
    pub nginx_site_available: String,
    pub nginx_site_enabled: String,
    pub nginx_default_site_enabled: String,
    pub systemd_site_nginx_service: String,
    pub apparmor_profile_path: String,
    pub runtime_socket_dir: String,
    pub runtime_nginx_socket: String,
    pub runtime_nginx_pid: String,
    pub runtime_php_fpm_socket: String,
    pub sudoers_path: String,
    pub usr_local_bin: String,
    pub bonesremote_global_link: String,
    pub apparmor_enabled_param: String,
    pub apparmor_profiles: String,
}

impl Deployment {
    #[must_use]
    pub fn new(project_name: &str, repo_path: &str, project_root: &str, web_root: &str) -> Self {
        let repo = repo_path.to_string();
        let project_root = project_root.to_string();
        let placeholder_release = Path::new(&project_root).join(RELEASES_DIR).join(PLACEHOLDER_RELEASE_NAME);
        let current = Path::new(&project_root).join(CURRENT_LINK);
        let runtime_socket_dir = Path::new(RUNTIME_SOCKET_PARENT).join(project_name);
        let repo_bones = Path::new(&repo).join(BONES_DIR);
        let conf_root = Path::new(DEFAULT_CONF_ROOT_PARENT).join(project_name);

        Self {
            repo: repo.clone(),
            repo_parent: parent_or_default(&repo, DEFAULT_REPO_PARENT),
            repo_head: Path::new(&repo).join(GIT_HEAD).display().to_string(),
            repo_bones: repo_bones.display().to_string(),
            repo_bones_toml: repo_bones.join(BONES_TOML).display().to_string(),
            site_nginx_config: conf_root.join(NGINX_CONF).display().to_string(),
            repo_deployment: repo_bones.join(DEPLOYMENT_DIR).display().to_string(),
            conf_root: conf_root.display().to_string(),
            project_root: project_root.clone(),
            project_root_parent: parent_or_default(&project_root, DEFAULT_PROJECT_ROOT_PARENT),
            releases: Path::new(&project_root).join(RELEASES_DIR).display().to_string(),
            shared: Path::new(&project_root).join(SHARED_DIR).display().to_string(),
            build_root: Path::new(&project_root).join(BUILD_DIR).join(WORKSPACE_DIR).display().to_string(),
            build_logs: Path::new(&project_root).join(BUILD_DIR).join(LOGS_DIR).display().to_string(),
            current: current.display().to_string(),
            current_web_root: current.join(web_root).display().to_string(),
            placeholder_release: placeholder_release.display().to_string(),
            placeholder_web_root: placeholder_release.join(web_root).display().to_string(),
            placeholder_index: placeholder_release.join(web_root).join(INDEX_HTML).display().to_string(),
            nginx_site_available: Path::new(ETC_NGINX_SITES_AVAILABLE)
                .join(format!("{project_name}.conf"))
                .display()
                .to_string(),
            nginx_site_enabled: Path::new(ETC_NGINX_SITES_ENABLED)
                .join(format!("{project_name}.conf"))
                .display()
                .to_string(),
            nginx_default_site_enabled: Path::new(ETC_NGINX_SITES_ENABLED)
                .join(DEFAULT_NGINX_SITE)
                .display()
                .to_string(),
            systemd_site_nginx_service: Path::new(ETC_SYSTEMD_SYSTEM)
                .join(nginx_service_name(project_name))
                .display()
                .to_string(),
            apparmor_profile_path: Path::new(ETC_APPARMOR_D)
                .join(format!("bonesdeploy-{project_name}-nginx"))
                .display()
                .to_string(),
            runtime_socket_dir: runtime_socket_dir.display().to_string(),
            runtime_nginx_socket: runtime_socket_dir.join(NGINX_SOCKET).display().to_string(),
            runtime_nginx_pid: runtime_socket_dir.join(NGINX_PID).display().to_string(),
            runtime_php_fpm_socket: runtime_socket_dir.join(PHP_FPM_SOCKET).display().to_string(),
            sudoers_path: Path::new(ETC_SUDOERS_D).join(SUDOERS_FILE).display().to_string(),
            usr_local_bin: USR_LOCAL_BIN.to_string(),
            bonesremote_global_link: Path::new(USR_LOCAL_BIN).join(BONESREMOTE_BINARY).display().to_string(),
            apparmor_enabled_param: APPARMOR_ENABLED_PARAM.to_string(),
            apparmor_profiles: APPARMOR_PROFILES.to_string(),
        }
    }
}

fn parent_or_default(path: &str, fallback: &str) -> String {
    Path::new(path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or_else(|| fallback.to_string(), |parent| parent.display().to_string())
}

fn home_dir() -> PathBuf {
    env::var("HOME").map_or_else(|_| PathBuf::from("/root"), PathBuf::from)
}

#[must_use]
pub fn bones_config_root() -> PathBuf {
    if let Some(dir) = env::var("XDG_CONFIG_HOME").ok().filter(|v| !v.is_empty()) {
        Path::new(&dir).join("bonesdeploy")
    } else {
        home_dir().join(".config/bonesdeploy")
    }
}

#[must_use]
pub fn bones_state_root() -> PathBuf {
    if let Some(dir) = env::var("XDG_STATE_HOME").ok().filter(|v| !v.is_empty()) {
        Path::new(&dir).join("bonesdeploy")
    } else {
        home_dir().join(".local/state/bonesdeploy")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bones_config_root_uses_xdg_config_home() {
        let home = home_dir();
        let expected = home.join(".config/bonesdeploy");
        assert_eq!(bones_config_root(), expected);
    }

    #[test]
    fn bones_state_root_uses_xdg_state_home() {
        let home = home_dir();
        let expected = home.join(".local/state/bonesdeploy");
        assert_eq!(bones_state_root(), expected);
    }
}
