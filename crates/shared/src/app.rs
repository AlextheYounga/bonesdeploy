use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct App {
    pub remote_name: String,
    pub project_name: String,
    pub ssh_user: String,
    pub host: String,
    pub port: String,
    pub repo_path: String,
    pub project_root: String,
    pub branch: String,
    pub preview_domain: String,
    pub deploy_on_push: bool,
    pub releases_keep: usize,
    pub ssl_enabled: bool,
    pub domain: String,
    pub email: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct AppFile {
    remote_name: String,
    project_name: String,
    repo_path: String,
    project_root: String,
    server: Server,
    dns: Dns,
    deploy: Deploy,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct Server {
    host: String,
    ssh_user: String,
    port: String,
}

impl Default for Server {
    fn default() -> Self {
        Self { host: String::new(), ssh_user: String::from("root"), port: String::from("22") }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct Dns {
    domain: String,
    preview_domain: String,
    email: String,
    ssl_enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct Deploy {
    branch: String,
    #[serde(rename = "deploy_on_push")]
    on_push: bool,
    releases: usize,
}

impl Default for Deploy {
    fn default() -> Self {
        Self { branch: String::from("master"), on_push: false, releases: 5 }
    }
}

#[derive(Serialize)]
struct AppDocument<'a> {
    remote_name: &'a str,
    project_name: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    repo_path: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    project_root: &'a str,
    server: ServerDocument<'a>,
    dns: DnsDocument<'a>,
    deploy: DeployDocument<'a>,
}

#[derive(Serialize)]
struct ServerDocument<'a> {
    host: &'a str,
    ssh_user: &'a str,
    port: &'a str,
}

#[derive(Serialize)]
struct DnsDocument<'a> {
    domain: &'a str,
    preview_domain: &'a str,
    email: &'a str,
    ssl_enabled: bool,
}

#[derive(Serialize)]
struct DeployDocument<'a> {
    branch: &'a str,
    #[serde(rename = "deploy_on_push")]
    on_push: bool,
    releases: usize,
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
            preview_domain: String::new(),
            deploy_on_push: false,
            releases_keep: 5,
            ssl_enabled: false,
            domain: String::new(),
            email: String::new(),
        }
    }
}

impl<'de> Deserialize<'de> for App {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let file = AppFile::deserialize(deserializer)?;
        Ok(Self {
            remote_name: file.remote_name,
            project_name: file.project_name,
            ssh_user: file.server.ssh_user,
            host: file.server.host,
            port: file.server.port,
            repo_path: file.repo_path,
            project_root: file.project_root,
            branch: file.deploy.branch,
            preview_domain: file.dns.preview_domain,
            deploy_on_push: file.deploy.on_push,
            releases_keep: file.deploy.releases,
            ssl_enabled: file.dns.ssl_enabled,
            domain: file.dns.domain,
            email: file.dns.email,
        })
    }
}

impl Serialize for App {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AppDocument {
            remote_name: &self.remote_name,
            project_name: &self.project_name,
            repo_path: &self.repo_path,
            project_root: &self.project_root,
            server: ServerDocument { host: &self.host, ssh_user: &self.ssh_user, port: &self.port },
            dns: DnsDocument {
                domain: &self.domain,
                preview_domain: &self.preview_domain,
                email: &self.email,
                ssl_enabled: self.ssl_enabled,
            },
            deploy: DeployDocument { branch: &self.branch, on_push: self.deploy_on_push, releases: self.releases_keep },
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::App;

    #[test]
    fn omitted_nested_sections_keep_app_defaults() {
        let app: App = toml::from_str("").expect("empty app config should parse");

        assert_eq!(app.ssh_user, "root");
        assert_eq!(app.port, "22");
        assert_eq!(app.branch, "master");
        assert_eq!(app.releases_keep, 5);
    }
}
