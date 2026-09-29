use std::collections::BTreeMap;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use super::model::{Backup, Bones, BuildMode, RuntimeBackend};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConnection {
    pub host: String,
    pub ssh_user: String,
    pub port: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteFields {
    pub project_name: String,
    pub domain: String,
    pub email: String,
    pub ssl_enabled: bool,
    pub template: String,
    pub backend: String,
    pub web_root: String,
    pub branch: String,
    pub node_version: String,
    pub compose_port: Option<u16>,
    pub compose_wait_timeout: u16,
    #[serde(default = "default_build_mode")]
    pub build_mode: String,
    pub backup: Backup,
    pub extras: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvisioningRequest {
    pub server: ServerConnection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<SiteFields>,
}

impl ProvisioningRequest {
    /// Builds a site-scoped request from local configuration.
    /// # Errors
    /// Returns an error when an extra is an array or table.
    pub fn from_bones(config: &Bones) -> Result<Self> {
        super::model::validate_build_mode(&config.runtime, &config.build)?;
        let mut extras = BTreeMap::new();
        for (key, value) in &config.runtime.extra {
            let json = match value {
                toml::Value::String(v) => serde_json::Value::String(v.clone()),
                toml::Value::Integer(v) => serde_json::Value::Number((*v).into()),
                toml::Value::Float(v) => {
                    let number =
                        serde_json::Number::from_f64(*v).ok_or_else(|| anyhow::anyhow!("invalid extra `{key}`"))?;
                    serde_json::Value::Number(number)
                }
                toml::Value::Boolean(v) => serde_json::Value::Bool(*v),
                toml::Value::Datetime(v) => serde_json::Value::String(v.to_string()),
                toml::Value::Array(_) | toml::Value::Table(_) => {
                    bail!("Runtime framework value `{key}` must be a scalar")
                }
            };
            extras.insert(key.clone(), json);
        }
        Ok(Self {
            server: ServerConnection {
                host: config.host.clone(),
                ssh_user: config.ssh_user.clone(),
                port: config.port.clone(),
            },
            site: Some(SiteFields {
                project_name: config.project_name.clone(),
                domain: config.domain.clone(),
                email: config.email.clone(),
                ssl_enabled: config.ssl_enabled,
                template: config.runtime.template.clone(),
                backend: match config.runtime.backend {
                    RuntimeBackend::Native => "native",
                    RuntimeBackend::Docker => "docker",
                }
                .into(),
                web_root: config.runtime.web_root.clone(),
                branch: config.branch.clone(),
                node_version: config.runtime.node_version.clone(),
                compose_port: config.runtime.compose_port,
                compose_wait_timeout: config.runtime.compose_wait_timeout,
                build_mode: match config.build.mode {
                    BuildMode::Local => "local".into(),
                    BuildMode::Remote => "remote".into(),
                },
                backup: config.backup.clone(),
                extras,
            }),
        })
    }

    #[must_use]
    pub fn server_only(host: &str, ssh_user: &str, port: &str) -> Self {
        Self {
            server: ServerConnection { host: host.into(), ssh_user: ssh_user.into(), port: port.into() },
            site: None,
        }
    }
}

fn default_build_mode() -> String {
    "remote".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteDeploymentConfig {
    pub branch: String,
    pub releases_keep: usize,
    pub runtime: super::model::Runtime,
    pub build: super::model::Build,
}

impl RemoteDeploymentConfig {
    #[must_use]
    pub fn from_bones(config: &Bones) -> Self {
        Self {
            branch: config.branch.clone(),
            releases_keep: config.releases_keep,
            runtime: config.runtime.clone(),
            build: config.build.clone(),
        }
    }

    /// Validates the transported runtime/build combination at the remote
    /// descriptor boundary.
    pub fn validate(&self) -> Result<()> {
        super::model::validate_runtime(&self.runtime)?;
        super::model::validate_build_mode(&self.runtime, &self.build)
    }

    #[must_use]
    pub fn into_site_config(self, site: &str) -> Bones {
        let mut config = Bones::for_site(site);
        config.branch = self.branch;
        config.releases_keep = self.releases_keep;
        config.runtime = self.runtime;
        config.build = self.build;
        config
    }
}
