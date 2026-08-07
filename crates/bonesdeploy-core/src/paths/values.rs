//! Flat accessors for the embedded path values in `crates/bonesdeploy-core/specs/paths.ron`.
//!
//! Kept in a separate module so `paths` (path derivation) stays under the line limit;
//! everything here is re-exported through `paths::*`.

use crate::specs;

#[must_use]
pub fn default_repo_parent() -> &'static str {
    &specs::paths().repo_parent
}

#[must_use]
pub fn default_project_root_parent() -> &'static str {
    &specs::paths().project_root_parent
}

#[must_use]
pub fn default_conf_root_parent() -> &'static str {
    &specs::paths().conf_root_parent
}

#[must_use]
pub fn default_web_root() -> String {
    specs::paths().web_root.clone()
}

#[must_use]
pub fn deploy_user() -> &'static str {
    &specs::paths().deploy_user
}

#[must_use]
pub fn default_group() -> &'static str {
    &specs::paths().default_group
}

#[must_use]
pub fn etc_nginx_sites_available() -> &'static str {
    &specs::paths().etc_nginx_sites_available
}

#[must_use]
pub fn etc_nginx_sites_enabled() -> &'static str {
    &specs::paths().etc_nginx_sites_enabled
}

#[must_use]
pub fn etc_systemd_system() -> &'static str {
    &specs::paths().etc_systemd_system
}

#[must_use]
pub fn etc_apparmor_d() -> &'static str {
    &specs::paths().etc_apparmor_d
}

#[must_use]
pub fn etc_letsencrypt_live() -> &'static str {
    &specs::paths().etc_letsencrypt_live
}

#[must_use]
pub fn etc_sudoers_d() -> &'static str {
    &specs::paths().etc_sudoers_d
}

#[must_use]
pub fn etc_os_release() -> &'static str {
    &specs::paths().etc_os_release
}

#[must_use]
pub fn etc_passwd() -> &'static str {
    &specs::paths().etc_passwd
}

#[must_use]
pub fn etc_group() -> &'static str {
    &specs::paths().etc_group
}

#[must_use]
pub fn apparmor_enabled_param() -> &'static str {
    &specs::paths().apparmor_enabled_param
}

#[must_use]
pub fn apparmor_profiles() -> &'static str {
    &specs::paths().apparmor_profiles
}

#[must_use]
pub fn usr_local_bin() -> &'static str {
    &specs::paths().usr_local_bin
}

#[must_use]
pub fn local_bones_dir() -> &'static str {
    &specs::paths().local_bones_dir
}

#[must_use]
pub fn local_bones_toml() -> &'static str {
    &specs::paths().local_bones_toml
}

#[must_use]
pub fn local_bones_deployment_dir() -> &'static str {
    &specs::paths().local_bones_deployment_dir
}

#[must_use]
pub fn local_bones_secrets_dir() -> &'static str {
    &specs::paths().local_bones_secrets_dir
}

#[must_use]
pub fn dot_env() -> &'static str {
    &specs::paths().dot_env
}

#[must_use]
pub fn env_build_file() -> &'static str {
    &specs::paths().env_build_file
}

#[must_use]
pub fn bones_dir() -> &'static str {
    &specs::paths().bones_dir
}

#[must_use]
pub fn bones_toml() -> &'static str {
    &specs::paths().bones_toml
}

#[must_use]
pub fn nginx_conf() -> &'static str {
    &specs::paths().nginx_conf
}

#[must_use]
pub fn index_html() -> &'static str {
    &specs::paths().index_html
}

#[must_use]
pub fn git_head() -> &'static str {
    &specs::paths().git_head
}

#[must_use]
pub fn deployment_dir() -> &'static str {
    &specs::paths().deployment_dir
}

#[must_use]
pub fn deployment_functions_file() -> &'static str {
    &specs::paths().deployment_functions_file
}

#[must_use]
pub fn deployment_build_dir() -> &'static str {
    &specs::paths().deployment_build_dir
}

#[must_use]
pub fn deployment_prepare_dir() -> &'static str {
    &specs::paths().deployment_prepare_dir
}

#[must_use]
pub fn releases_dir() -> &'static str {
    &specs::paths().releases_dir
}

#[must_use]
pub fn shared_dir() -> &'static str {
    &specs::paths().shared_dir
}

#[must_use]
pub fn build_dir() -> &'static str {
    &specs::paths().build_dir
}

#[must_use]
pub fn workspace_dir() -> &'static str {
    &specs::paths().workspace_dir
}

#[must_use]
pub fn logs_dir() -> &'static str {
    &specs::paths().logs_dir
}

#[must_use]
pub fn current_link() -> &'static str {
    &specs::paths().current_link
}

#[must_use]
pub fn staged_release_file() -> &'static str {
    &specs::paths().staged_release_file
}

#[must_use]
pub fn active_deployment_file() -> &'static str {
    &specs::paths().active_deployment_file
}

#[must_use]
pub fn deployment_state_file() -> &'static str {
    &specs::paths().deployment_state_file
}

#[must_use]
pub fn deployment_lock_file() -> &'static str {
    &specs::paths().deployment_lock_file
}

#[must_use]
pub fn recovery_dir() -> &'static str {
    &specs::paths().recovery_dir
}

#[must_use]
pub fn tmp_builds_dir() -> &'static str {
    &specs::paths().tmp_builds_dir
}

#[must_use]
pub fn placeholder_release_name() -> &'static str {
    &specs::paths().placeholder_release_name
}

#[must_use]
pub fn sudoers_file() -> &'static str {
    &specs::paths().sudoers_file
}

#[must_use]
pub fn sudoers_path() -> &'static str {
    &specs::paths().sudoers_path
}

#[must_use]
pub fn bonesdeploy_binary() -> &'static str {
    &specs::paths().bonesdeploy_binary
}

#[must_use]
pub fn bonesremote_binary() -> &'static str {
    &specs::paths().bonesremote_binary
}

#[must_use]
pub fn bonesremote_config_dir() -> &'static str {
    &specs::paths().bonesremote_config_dir
}

#[must_use]
pub fn bonesremote_sites_dir() -> &'static str {
    &specs::paths().bonesremote_sites_dir
}

#[must_use]
pub fn bonesremote_repos_dir() -> &'static str {
    &specs::paths().bonesremote_repos_dir
}

#[must_use]
pub fn bonesdeploy_users_root() -> &'static str {
    &specs::paths().bonesdeploy_users_root
}

#[must_use]
pub fn build_cache_dir() -> &'static str {
    &specs::paths().build_cache_dir
}

#[must_use]
pub fn nginx_socket() -> &'static str {
    &specs::paths().nginx_socket
}

#[must_use]
pub fn nginx_pid() -> &'static str {
    &specs::paths().nginx_pid
}

#[must_use]
pub fn php_fpm_socket() -> &'static str {
    &specs::paths().php_fpm_socket
}

#[must_use]
pub fn default_nginx_site() -> &'static str {
    &specs::paths().default_nginx_site
}

#[must_use]
pub fn systemd_service_suffix() -> &'static str {
    &specs::paths().systemd_service_suffix
}

#[must_use]
pub fn git_hooks_dir() -> &'static str {
    &specs::paths().git_hooks_dir
}

#[must_use]
pub fn git_pre_push_hook() -> &'static str {
    &specs::paths().git_pre_push_hook
}

#[must_use]
pub fn pre_push_hook_name() -> &'static str {
    &specs::paths().pre_push_hook_name
}

#[must_use]
pub fn hooks_dir() -> &'static str {
    &specs::paths().hooks_dir
}

#[must_use]
pub fn confs_dir() -> &'static str {
    &specs::paths().confs_dir
}

#[must_use]
pub fn kit_deployment_dir() -> &'static str {
    &specs::paths().kit_deployment_dir
}

#[must_use]
pub fn kit_confs_dir() -> &'static str {
    &specs::paths().kit_confs_dir
}

#[must_use]
pub fn bones_config_projects_dir() -> &'static str {
    &specs::paths().bones_config_projects_dir
}

#[must_use]
pub fn bonesdeploy_dir() -> &'static str {
    &specs::paths().bonesdeploy_dir
}

#[must_use]
pub fn gitignore_file() -> &'static str {
    &specs::paths().gitignore_file
}
