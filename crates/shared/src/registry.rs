use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::{config, paths};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Registry {
    pub site: String,
    pub repo_path: String,
    pub site_root: String,
    pub shared_root: String,
    pub releases_root: String,
    pub current_path: String,
    pub runtime_user: String,
    pub runtime_group: String,
    pub branch: String,
    pub deploy_on_push: bool,
    pub releases_keep: usize,
}

impl Registry {
    #[must_use]
    pub fn derive(bones: &config::Bones) -> Self {
        let site_root = bones.project_root.clone();
        Self {
            site: bones.project_name.clone(),
            repo_path: bones.repo_path.clone(),
            shared_root: format!("{site_root}/{}", paths::SHARED_DIR),
            releases_root: format!("{site_root}/{}", paths::RELEASES_DIR),
            current_path: format!("{site_root}/{}", paths::CURRENT_LINK),
            site_root,
            runtime_user: config::runtime_user_for(&bones.project_name),
            runtime_group: config::runtime_group_for(&bones.project_name),
            branch: bones.branch.clone(),
            deploy_on_push: bones.deploy_on_push,
            releases_keep: bones.releases_keep,
        }
    }
}

/// # Errors
///
/// Returns an error when the site name is empty or contains characters outside
/// ASCII lowercase letters, digits, and dashes.
pub fn validate_site_name(site: &str) -> Result<()> {
    if site.is_empty() {
        bail!("Site name cannot be empty");
    }

    if site.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        return Ok(());
    }

    bail!("Invalid site name: {site}")
}

#[cfg(test)]
mod tests {
    use super::{validate_site_name, Registry};
    use crate::config::Bones;

    #[test]
    fn validate_site_name_rejects_path_escapes() {
        assert!(validate_site_name("../evil").is_err());
        assert!(validate_site_name("evil/site").is_err());
        assert!(validate_site_name("good-site").is_ok());
    }

    #[test]
    fn derive_uses_conventional_remote_paths() {
        let bones = Bones {
            project_name: String::from("acme"),
            repo_path: String::from("/home/git/acme.git"),
            project_root: String::from("/srv/sites/acme"),
            branch: String::from("main"),
            deploy_on_push: true,
            releases_keep: 7,
            ..Default::default()
        };

        let registry = Registry::derive(&bones);
        assert_eq!(registry.shared_root, "/srv/sites/acme/shared");
        assert_eq!(registry.releases_root, "/srv/sites/acme/releases");
        assert_eq!(registry.current_path, "/srv/sites/acme/current");
        assert_eq!(registry.runtime_user, "acme");
        assert_eq!(registry.branch, "main");
        assert!(registry.deploy_on_push);
    }
}
