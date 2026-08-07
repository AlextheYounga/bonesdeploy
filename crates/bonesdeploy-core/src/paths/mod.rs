//! Path derivation for bonesdeploy.
//!
//! Compiled-in path values live in `crates/bonesdeploy-core/specs/paths.ron` and are
//! exposed through `values`; this module derives concrete paths from projects and
//! sites and re-exports the flat accessors so callers use `paths::*`.

use std::env;
use std::path::{Path, PathBuf};

mod values;
pub use values::*;

#[must_use]
pub fn default_repo_path_for(project_name: &str) -> String {
    Path::new(default_repo_parent()).join(format!("{project_name}.git")).display().to_string()
}

#[must_use]
pub fn default_bones_repo_path_for(project_name: &str) -> String {
    bonesremote_config_root()
        .join(bonesremote_repos_dir())
        .join(format!("{project_name}.bones.git"))
        .display()
        .to_string()
}

#[must_use]
pub fn default_project_root_for(project_name: &str) -> String {
    Path::new(default_project_root_parent()).join(project_name).display().to_string()
}

#[must_use]
pub fn ssl_certificate_path(domain: &str) -> String {
    Path::new(etc_letsencrypt_live()).join(domain).join("fullchain.pem").display().to_string()
}

#[must_use]
pub fn ssl_certificate_key_path(domain: &str) -> String {
    Path::new(etc_letsencrypt_live()).join(domain).join("privkey.pem").display().to_string()
}

#[must_use]
pub fn site_target_name(project_name: &str) -> String {
    format!("{project_name}.target")
}

#[must_use]
pub fn bonesremote_config_root() -> PathBuf {
    PathBuf::from(bonesremote_config_dir())
}

#[must_use]
pub fn bonesremote_sites_root() -> PathBuf {
    bonesremote_config_root().join(bonesremote_sites_dir())
}

#[must_use]
pub fn bonesremote_site_root(site: &str) -> PathBuf {
    bonesremote_sites_root().join(site)
}

#[must_use]
pub fn bonesremote_bones_toml_path(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(bones_toml())
}

#[must_use]
pub fn bonesremote_staged_release_path(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(staged_release_file())
}

#[must_use]
pub fn bonesremote_tmp_builds_root(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(tmp_builds_dir())
}

#[must_use]
pub fn bonesremote_site_logs(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(logs_dir())
}

#[must_use]
pub fn bonesdeploy_user_home(user: &str) -> PathBuf {
    Path::new(bonesdeploy_users_root()).join(user)
}

#[must_use]
pub fn bonesdeploy_user_cache(user: &str) -> PathBuf {
    bonesdeploy_user_home(user).join(build_cache_dir())
}

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
pub fn bonesremote_global_link() -> PathBuf {
    Path::new(usr_local_bin()).join(bonesremote_binary())
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
    bones_config_root().join(bones_config_projects_dir())
}

#[must_use]
pub fn bones_data_root() -> PathBuf {
    if let Some(dir) = env::var("XDG_DATA_HOME").ok().filter(|v| !v.is_empty()) {
        return Path::new(&dir).join(bonesdeploy_dir());
    }
    home_dir().join(".local/share").join(bonesdeploy_dir())
}

#[must_use]
pub fn bones_cache_root() -> PathBuf {
    if let Some(dir) = env::var("XDG_CACHE_HOME").ok().filter(|v| !v.is_empty()) {
        return Path::new(&dir).join(bonesdeploy_dir());
    }
    home_dir().join(".cache").join(bonesdeploy_dir())
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
    use super::{
        default_bones_repo_path_for, default_project_root_for, default_repo_path_for, default_web_root,
        site_target_name, ssl_certificate_key_path, ssl_certificate_path,
    };

    #[test]
    fn site_target_name_is_exactly_project_derived() {
        assert_eq!(site_target_name("nexttest"), "nexttest.target");
        assert_ne!(site_target_name("shop"), "shop-admin.target");
    }

    #[test]
    fn derived_paths_build_from_spec_defaults() {
        assert_eq!(default_repo_path_for("atlas"), "/home/git/atlas.git");
        assert_eq!(default_project_root_for("atlas"), "/srv/sites/atlas");
        assert_eq!(default_bones_repo_path_for("atlas"), "/root/.config/bonesremote/repos/atlas.bones.git");
        assert_eq!(default_web_root(), "public");
        assert_eq!(ssl_certificate_path("example.com"), "/etc/letsencrypt/live/example.com/fullchain.pem");
        assert_eq!(ssl_certificate_key_path("example.com"), "/etc/letsencrypt/live/example.com/privkey.pem");
    }
}
