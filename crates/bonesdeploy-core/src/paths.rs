use std::env;
use std::path::{Path, PathBuf};

pub const DEFAULT_REPO_PARENT: &str = "/home/git";
pub const DEFAULT_PROJECT_ROOT_PARENT: &str = "/srv/sites";
pub const DEFAULT_CONF_ROOT_PARENT: &str = "/srv/conf";
pub const DEFAULT_WEB_ROOT: &str = "public";

pub const DEPLOY_USER: &str = "git";
pub const DEFAULT_GROUP: &str = "www-data";

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
pub const USR_LOCAL_BIN: &str = "/usr/local/bin";

pub const OLD_BONES_DIR: &str = ".bones";
pub const LOCAL_INFRA_DIR: &str = "infra";
pub const LOCAL_INFRA_DEPLOYMENT_DIR: &str = "infra/deployment";
pub const LOCAL_INFRA_TEMPLATES_DIR: &str = "infra/templates";
pub const LOCAL_INFRA_SECRETS_DIR: &str = "infra/secrets";
pub const DOT_ENV: &str = ".env";
pub const ENV_BUILD_FILE: &str = ".env.build";

pub const BONES_DIR: &str = "bones";
pub const NGINX_CONF: &str = "nginx.conf";
pub const INDEX_HTML: &str = "index.html";
pub const GIT_HEAD: &str = "HEAD";
pub const DEPLOYMENT_DIR: &str = "deployment";
pub const DEPLOYMENT_FUNCTIONS_FILE: &str = "functions.sh";
pub const DEPLOYMENT_BUILD_DIR: &str = "build";
pub const DEPLOYMENT_PREPARE_DIR: &str = "prepare";
pub const RELEASES_DIR: &str = "releases";
pub const SHARED_DIR: &str = "shared";
pub const BUILD_DIR: &str = "build";
pub const WORKSPACE_DIR: &str = "workspace";
pub const LOGS_DIR: &str = "logs";
pub const CURRENT_LINK: &str = "current";
pub const STAGED_RELEASE_FILE: &str = "staged-release";
pub const ACTIVE_DEPLOYMENT_FILE: &str = "active-deployment.json";
pub const DEPLOYMENT_STATE_FILE: &str = "deployment.json";
pub const DEPLOYMENT_LOCK_FILE: &str = "deployment.lock";
pub const RECOVERY_DIR: &str = "recovery";
pub const TMP_BUILDS_DIR: &str = "tmp";
pub const PLACEHOLDER_RELEASE_NAME: &str = "19700101_000000";
pub const SUDOERS_FILE: &str = "bonesdeploy";
pub const SUDOERS_PATH: &str = "/etc/sudoers.d/bonesdeploy";
pub const BONESDEPLOY_BINARY: &str = "bonesdeploy";
pub const BONESREMOTE_BINARY: &str = "bonesremote";
pub const BONESREMOTE_CONFIG_DIR: &str = "/root/.config/bonesremote";
pub const BONESREMOTE_SITES_DIR: &str = "sites";
pub const BONESDEPLOY_STATE_ROOT: &str = "/var/lib/bonesdeploy/state";
pub const BONESDEPLOY_LOCK_ROOT: &str = "/var/lib/bonesdeploy/locks";
pub const DEPLOYMENT_SNAPSHOT_ROOT: &str = "/var/lib/bonesdeploy/config";
pub const BACKUPS_ROOT: &str = "/var/lib/bonesdeploy/backups";
pub const BORG_PASSPHRASE_FILE: &str = ".borg_passphrase";
pub const BONESDEPLOY_USERS_ROOT: &str = "/var/lib/bonesdeploy/users";
pub const IMAGE_STORE_GRAPH_ROOT: &str = "/var/lib/bonesdeploy/image-store";
pub const IMAGE_STORE_RUN_ROOT: &str = "/run/bonesdeploy/image-store";
pub const IMAGE_STORE_STORAGE_CONF: &str = "/etc/bonesdeploy/image-store-storage.conf";
pub const IMAGE_STORE_BASE_IMAGE: &str = "docker.io/library/buildpack-deps:bookworm";
pub const BUILD_CACHE_DIR: &str = "cache";
pub const NGINX_SOCKET: &str = "nginx.sock";
pub const NGINX_PID: &str = "nginx.pid";
pub const PHP_FPM_SOCKET: &str = "php-fpm.sock";
pub const DEFAULT_NGINX_SITE: &str = "default";
pub const SYSTEMD_TARGET_SUFFIX: &str = ".target";
pub const SYSTEMD_SERVICE_SUFFIX: &str = ".service";

pub const HOOKS_DIR: &str = "hooks";
pub const KIT_DEPLOYMENT_DIR: &str = "deployment/";
pub const BONES_CONFIG_PROJECTS_DIR: &str = "projects";
pub const BONESDEPLOY_DIR: &str = "bonesdeploy";
pub const GITIGNORE_FILE: &str = ".gitignore";

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
pub fn branch_ref(branch: &str) -> String {
    format!("refs/heads/{branch}")
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
pub fn site_target_name(project_name: &str) -> String {
    format!("{project_name}.target")
}

#[must_use]
pub fn bonesremote_secrets_root() -> PathBuf {
    bonesremote_config_root().join(BONESREMOTE_SITES_DIR)
}

#[must_use]
pub fn bonesremote_secret_site_root(site: &str) -> PathBuf {
    bonesremote_secrets_root().join(site)
}

#[must_use]
pub fn bonesremote_state_root() -> PathBuf {
    PathBuf::from(BONESDEPLOY_STATE_ROOT)
}

#[must_use]
pub fn bonesremote_state_site_root(site: &str) -> PathBuf {
    bonesremote_state_root().join(site)
}

#[must_use]
pub fn bonesremote_lock_root() -> PathBuf {
    PathBuf::from(BONESDEPLOY_LOCK_ROOT)
}

#[must_use]
pub fn bonesremote_deployment_lock_path(site: &str) -> PathBuf {
    bonesremote_lock_root().join(format!(".{site}.{DEPLOYMENT_LOCK_FILE}"))
}

#[must_use]
pub fn bonesremote_snapshot_root() -> PathBuf {
    PathBuf::from(DEPLOYMENT_SNAPSHOT_ROOT)
}

#[must_use]
pub fn bonesremote_snapshot_path(site: &str) -> PathBuf {
    bonesremote_snapshot_root().join(site).join("bones.json")
}

#[must_use]
pub fn bonesremote_site_logs(site: &str) -> PathBuf {
    bonesremote_secret_site_root(site).join(LOGS_DIR)
}

#[must_use]
pub fn site_backup_repository_path(site: &str) -> PathBuf {
    Path::new(BACKUPS_ROOT).join(format!("{site}.borg"))
}

#[must_use]
pub fn bonesremote_site_passphrase_path(site: &str) -> PathBuf {
    bonesremote_secret_site_root(site).join(BORG_PASSPHRASE_FILE)
}

#[must_use]
pub fn bonesdeploy_user_home(user: &str) -> PathBuf {
    Path::new(BONESDEPLOY_USERS_ROOT).join(user)
}

#[must_use]
pub fn bonesdeploy_user_cache(user: &str) -> PathBuf {
    bonesdeploy_user_home(user).join(BUILD_CACHE_DIR)
}

#[must_use]
pub fn bonesremote_global_link() -> PathBuf {
    Path::new(USR_LOCAL_BIN).join(BONESREMOTE_BINARY)
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
pub fn bones_projects_root() -> PathBuf {
    bones_config_root().join(BONES_CONFIG_PROJECTS_DIR)
}

#[must_use]
pub fn bones_data_root() -> PathBuf {
    if let Some(dir) = env::var("XDG_DATA_HOME").ok().filter(|v| !v.is_empty()) {
        return Path::new(&dir).join(BONESDEPLOY_DIR);
    }
    home_dir().join(".local/share").join(BONESDEPLOY_DIR)
}

#[must_use]
pub fn bones_cache_root() -> PathBuf {
    if let Some(dir) = env::var("XDG_CACHE_HOME").ok().filter(|v| !v.is_empty()) {
        return Path::new(&dir).join(BONESDEPLOY_DIR);
    }
    home_dir().join(".cache").join(BONESDEPLOY_DIR)
}

#[must_use]
pub fn bones_state_root() -> PathBuf {
    if let Some(dir) = env::var("XDG_STATE_HOME").ok().filter(|v| !v.is_empty()) {
        Path::new(&dir).join("bonesdeploy")
    } else {
        home_dir().join(".local/state/bonesdeploy")
    }
}
#[must_use]
pub fn bonesremote_config_root() -> PathBuf {
    PathBuf::from(BONESREMOTE_CONFIG_DIR)
}
