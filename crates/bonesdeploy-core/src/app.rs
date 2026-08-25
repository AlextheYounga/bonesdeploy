use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    pub remote_name: String,
    pub project_name: String,
    pub ssh_user: String,
    pub host: String,
    pub port: String,
    pub repo_path: String,
    pub project_root: String,
    pub branch: String,
    pub releases_keep: usize,
    pub ssl_enabled: bool,
    pub domain: String,
    pub email: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            remote_name: String::new(),
            project_name: String::new(),
            ssh_user: String::from("root"),
            host: String::new(),
            port: String::from("22"),
            repo_path: String::new(),
            project_root: String::new(),
            branch: String::from("master"),
            releases_keep: 5,
            ssl_enabled: false,
            domain: String::new(),
            email: String::new(),
        }
    }
}
