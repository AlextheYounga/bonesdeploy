//! Embedded Core specifications.
//!
//! The `specs/` directory holds the topic-oriented RON documents that are the
//! source of truth for shared infrastructure paths and default configuration.
//! Each document is embedded at compile time with `include_str!` and
//! deserialized once into a typed specification.
//!
//! A specification is not user-editable project configuration: it is compiled
//! into the binaries. A malformed embedded document is an authoring error, so
//! the accessors below surface the parse error with context when it happens.

use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::config::PermissionRule;

#[derive(Clone, Debug, Deserialize)]
pub struct Paths {
    // Default deployment locations.
    pub repo_parent: String,
    pub project_root_parent: String,
    pub conf_root_parent: String,
    pub web_root: String,
    // Accounts used during provisioning.
    pub deploy_user: String,
    pub default_group: String,
    // System locations.
    pub etc_nginx_sites_available: String,
    pub etc_nginx_sites_enabled: String,
    pub etc_systemd_system: String,
    pub etc_apparmor_d: String,
    pub etc_letsencrypt_live: String,
    pub etc_sudoers_d: String,
    pub etc_os_release: String,
    pub etc_passwd: String,
    pub etc_group: String,
    pub apparmor_enabled_param: String,
    pub apparmor_profiles: String,
    pub usr_local_bin: String,
    // Project-local configuration in a repository.
    pub local_bones_dir: String,
    pub local_bones_toml: String,
    pub local_bones_deployment_dir: String,
    pub local_bones_secrets_dir: String,
    pub dot_env: String,
    pub env_build_file: String,
    // Layout on a remote site root.
    pub bones_dir: String,
    pub bones_toml: String,
    pub nginx_conf: String,
    pub index_html: String,
    pub git_head: String,
    pub deployment_dir: String,
    pub deployment_functions_file: String,
    pub deployment_build_dir: String,
    pub deployment_prepare_dir: String,
    pub releases_dir: String,
    pub shared_dir: String,
    pub build_dir: String,
    pub workspace_dir: String,
    pub logs_dir: String,
    pub current_link: String,
    pub staged_release_file: String,
    pub active_deployment_file: String,
    pub deployment_state_file: String,
    pub deployment_lock_file: String,
    pub recovery_dir: String,
    pub tmp_builds_dir: String,
    pub placeholder_release_name: String,
    pub sudoers_file: String,
    pub sudoers_path: String,
    pub bonesdeploy_binary: String,
    pub bonesremote_binary: String,
    pub bonesremote_config_dir: String,
    pub bonesremote_sites_dir: String,
    pub bonesremote_repos_dir: String,
    pub bonesdeploy_users_root: String,
    pub build_cache_dir: String,
    pub nginx_socket: String,
    pub nginx_pid: String,
    pub php_fpm_socket: String,
    pub default_nginx_site: String,
    pub systemd_service_suffix: String,
    // Git integration.
    pub git_hooks_dir: String,
    pub git_pre_push_hook: String,
    pub pre_push_hook_name: String,
    // Static kit assets shipped to a project.
    pub hooks_dir: String,
    pub confs_dir: String,
    pub kit_deployment_dir: String,
    pub kit_confs_dir: String,
    // bonesdeploy user configuration.
    pub bones_config_projects_dir: String,
    pub bonesdeploy_dir: String,
    pub gitignore_file: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApplicationDefaults {
    pub ssh_user: String,
    pub port: String,
    pub branch: String,
    pub releases_keep: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RuntimeDefaults {
    pub template: String,
    pub web_root: String,
    pub node_version: String,
    pub release_permissions: Vec<PermissionRule>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BuildDefaults {
    pub timeout_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServiceDefaults {
    pub database_services: Vec<String>,
}

#[must_use]
pub fn paths() -> &'static Paths {
    static CACHE: OnceLock<Paths> = OnceLock::new();
    loaded(&CACHE, "paths.ron", include_str!("../specs/paths.ron"))
}

#[must_use]
pub fn application_defaults() -> &'static ApplicationDefaults {
    static CACHE: OnceLock<ApplicationDefaults> = OnceLock::new();
    loaded(&CACHE, "application_defaults.ron", include_str!("../specs/application_defaults.ron"))
}

#[must_use]
pub fn runtime_defaults() -> &'static RuntimeDefaults {
    static CACHE: OnceLock<RuntimeDefaults> = OnceLock::new();
    loaded(&CACHE, "runtime_defaults.ron", include_str!("../specs/runtime_defaults.ron"))
}

#[must_use]
pub fn build_defaults() -> &'static BuildDefaults {
    static CACHE: OnceLock<BuildDefaults> = OnceLock::new();
    loaded(&CACHE, "build_defaults.ron", include_str!("../specs/build_defaults.ron"))
}

#[must_use]
pub fn service_defaults() -> &'static ServiceDefaults {
    static CACHE: OnceLock<ServiceDefaults> = OnceLock::new();
    loaded(&CACHE, "service_defaults.ron", include_str!("../specs/service_defaults.ron"))
}

/// Parses an embedded RON document exactly once and returns it for the rest of
/// the process lifetime.
///
/// Embedded documents are validated by the unit tests in this module, so a
/// parse failure here is unreachable in practice; it is reported with the
/// underlying error context rather than silently defaulting.
#[expect(clippy::expect_used)]
#[must_use]
fn loaded<'a, T: DeserializeOwned>(cache: &'a OnceLock<T>, name: &str, raw: &str) -> &'a T {
    cache.get_or_init(|| {
        parse::<T>(name, raw).expect("embedded Core specifications are compiled into the binary and validated by tests")
    })
}

/// Parses an embedded RON document into `T`, returning any parse error with
/// context. Used by tests to validate every shipped specification.
pub(crate) fn parse<T: DeserializeOwned>(name: &str, raw: &str) -> Result<T> {
    ron::from_str(raw).with_context(|| format!("Failed to parse embedded Core specification {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PermissionType;

    fn load_all() -> Result<()> {
        parse::<Paths>("paths.ron", include_str!("../specs/paths.ron"))?;
        parse::<ApplicationDefaults>("application_defaults.ron", include_str!("../specs/application_defaults.ron"))?;
        parse::<RuntimeDefaults>("runtime_defaults.ron", include_str!("../specs/runtime_defaults.ron"))?;
        parse::<BuildDefaults>("build_defaults.ron", include_str!("../specs/build_defaults.ron"))?;
        parse::<ServiceDefaults>("service_defaults.ron", include_str!("../specs/service_defaults.ron"))?;
        Ok(())
    }

    #[test]
    fn every_embedded_specification_parses() -> Result<()> {
        load_all()?;
        Ok(())
    }

    #[test]
    fn paths_spec_holds_the_expected_infrastructure_defaults() {
        let spec = paths();
        assert_eq!(spec.repo_parent, "/home/git");
        assert_eq!(spec.project_root_parent, "/srv/sites");
        assert_eq!(spec.conf_root_parent, "/srv/conf");
        assert_eq!(spec.web_root, "public");
        assert_eq!(spec.deploy_user, "git");
        assert_eq!(spec.default_group, "www-data");
        assert_eq!(spec.etc_nginx_sites_available, "/etc/nginx/sites-available");
        assert_eq!(spec.etc_os_release, "/etc/os-release");
        assert_eq!(spec.local_bones_toml, ".bones/bones.toml");
        assert_eq!(spec.env_build_file, ".env.build");
        assert_eq!(spec.releases_dir, "releases");
        assert_eq!(spec.systemd_service_suffix, ".service");
        assert_eq!(spec.bonesremote_config_dir, "/root/.config/bonesremote");
        assert_eq!(spec.bonesdeploy_users_root, "/var/lib/bonesdeploy/users");
        assert_eq!(spec.sudoers_path, "/etc/sudoers.d/bonesdeploy");
        assert_eq!(spec.git_pre_push_hook, ".git/hooks/pre-push");
    }

    #[test]
    fn application_defaults_hold_the_expected_settings() {
        let spec = application_defaults();
        assert_eq!(spec.ssh_user, "root");
        assert_eq!(spec.port, "22");
        assert_eq!(spec.branch, "master");
        assert_eq!(spec.releases_keep, 5);
    }

    #[test]
    fn runtime_defaults_hold_the_expected_settings() {
        let spec = runtime_defaults();
        assert_eq!(spec.template, "");
        assert_eq!(spec.web_root, "public");
        assert_eq!(spec.node_version, "24.18.0");
        assert_eq!(
            spec.release_permissions,
            vec![
                PermissionRule {
                    path: String::from("*"),
                    permission_type: PermissionType::Dir,
                    mode: String::from("750"),
                    recursive: None,
                },
                PermissionRule {
                    path: String::from("*"),
                    permission_type: PermissionType::File,
                    mode: String::from("640"),
                    recursive: None,
                },
            ]
        );
    }

    #[test]
    fn build_defaults_hold_the_expected_timeout() {
        assert_eq!(build_defaults().timeout_seconds, 300);
    }

    #[test]
    fn service_defaults_list_the_supported_databases() {
        assert_eq!(
            service_defaults().database_services,
            vec!["postgres", "mariadb", "mysql", "mongodb", "valkey", "redis"]
        );
    }
}
