use std::path::Path;

use serde::{Deserialize, Serialize};

const RELEASES_DIR: &str = "releases";
const SHARED_DIR: &str = "shared";
const BUILD_DIR: &str = "build";
const WORKSPACE_DIR: &str = "workspace";
const CURRENT_LINK: &str = "current";
const PLACEHOLDER_RELEASE_NAME: &str = "19700101_000000";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentPaths {
    pub repo: String,
    pub repo_parent: String,
    pub repo_bones: String,
    pub repo_bones_yaml: String,
    pub repo_nginx_config: String,
    pub project_root: String,
    pub project_root_parent: String,
    pub releases: String,
    pub shared: String,
    pub build_root: String,
    pub current: String,
    pub current_web_root: String,
    pub placeholder_release: String,
    pub placeholder_web_root: String,
    pub placeholder_index: String,
    pub nginx_site_available: String,
    pub nginx_site_enabled: String,
    pub systemd_site_nginx_service: String,
    pub apparmor_profile_path: String,
    pub runtime_socket_dir: String,
    pub runtime_nginx_socket: String,
}

impl DeploymentPaths {
    pub fn new(project_name: &str, repo_path: &str, project_root: &str, web_root: &str) -> Self {
        let repo = repo_path.to_string();
        let project_root = project_root.to_string();
        let web_root = web_root.to_string();
        let placeholder_release = Path::new(&project_root).join(RELEASES_DIR).join(PLACEHOLDER_RELEASE_NAME);
        let current = Path::new(&project_root).join(CURRENT_LINK);
        let runtime_socket_dir = Path::new("/run").join(project_name);

        Self {
            repo: repo.clone(),
            repo_parent: parent_or_default(&repo, "/home/git"),
            repo_bones: Path::new(&repo).join("bones").display().to_string(),
            repo_bones_yaml: Path::new(&repo).join("bones").join("bones.yaml").display().to_string(),
            repo_nginx_config: Path::new(&repo).join("bones").join("nginx.conf").display().to_string(),
            project_root: project_root.clone(),
            project_root_parent: parent_or_default(&project_root, "/srv/deployments"),
            releases: Path::new(&project_root).join(RELEASES_DIR).display().to_string(),
            shared: Path::new(&project_root).join(SHARED_DIR).display().to_string(),
            build_root: Path::new(&project_root).join(BUILD_DIR).join(WORKSPACE_DIR).display().to_string(),
            current: current.display().to_string(),
            current_web_root: current.join(&web_root).display().to_string(),
            placeholder_release: placeholder_release.display().to_string(),
            placeholder_web_root: placeholder_release.join(&web_root).display().to_string(),
            placeholder_index: placeholder_release.join(&web_root).join("index.html").display().to_string(),
            nginx_site_available: Path::new("/etc/nginx/sites-available")
                .join(format!("{project_name}.conf"))
                .display()
                .to_string(),
            nginx_site_enabled: Path::new("/etc/nginx/sites-enabled")
                .join(format!("{project_name}.conf"))
                .display()
                .to_string(),
            systemd_site_nginx_service: Path::new("/etc/systemd/system")
                .join(format!("{project_name}-nginx.service"))
                .display()
                .to_string(),
            apparmor_profile_path: Path::new("/etc/apparmor.d")
                .join(format!("bonesdeploy-{project_name}-nginx"))
                .display()
                .to_string(),
            runtime_socket_dir: runtime_socket_dir.display().to_string(),
            runtime_nginx_socket: runtime_socket_dir.join("nginx.sock").display().to_string(),
        }
    }
}

fn parent_or_default(path: &str, fallback: &str) -> String {
    Path::new(path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or_else(|| fallback.to_string(), |parent| parent.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::DeploymentPaths;

    #[test]
    fn deployment_paths_include_placeholder_web_root_and_index() {
        let paths =
            DeploymentPaths::new("makebabies", "/home/git/makebabies.git", "/srv/deployments/makebabies", "public");

        assert_eq!(paths.placeholder_web_root, "/srv/deployments/makebabies/releases/19700101_000000/public");
        assert_eq!(paths.placeholder_index, "/srv/deployments/makebabies/releases/19700101_000000/public/index.html");
        assert_eq!(paths.current_web_root, "/srv/deployments/makebabies/current/public");
    }

    #[test]
    fn deployment_paths_include_nginx_and_apparmor_targets() {
        let paths =
            DeploymentPaths::new("makebabies", "/home/git/makebabies.git", "/srv/deployments/makebabies", "public");

        assert_eq!(paths.repo_bones_yaml, "/home/git/makebabies.git/bones/bones.yaml");
        assert_eq!(paths.repo_nginx_config, "/home/git/makebabies.git/bones/nginx.conf");
        assert_eq!(paths.nginx_site_available, "/etc/nginx/sites-available/makebabies.conf");
        assert_eq!(paths.nginx_site_enabled, "/etc/nginx/sites-enabled/makebabies.conf");
        assert_eq!(paths.apparmor_profile_path, "/etc/apparmor.d/bonesdeploy-makebabies-nginx");
    }
}
