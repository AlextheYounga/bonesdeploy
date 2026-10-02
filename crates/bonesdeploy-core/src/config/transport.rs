use std::collections::BTreeMap;
use std::path::{Component, Path};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use super::model::{Backup, Bones, Runtime, RuntimeBackend};

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
    pub template: String,
    pub backend: String,
    pub web_root: String,
    pub node_version: String,
    pub compose_port: Option<u16>,
    pub compose_wait_timeout: u16,
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
                template: config.runtime.template.clone(),
                backend: match config.runtime.backend {
                    RuntimeBackend::Native => "native",
                    RuntimeBackend::Docker => "docker",
                }
                .into(),
                web_root: config.runtime.web_root.clone(),
                node_version: config.runtime.node_version.clone(),
                compose_port: config.runtime.compose_port,
                compose_wait_timeout: config.runtime.compose_wait_timeout,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "backend", rename_all = "lowercase", deny_unknown_fields)]
pub enum RemoteRuntime {
    Native {
        web_root: String,
        // Read descriptors written before production switched to distribution runtimes.
        #[serde(default, skip_serializing)]
        ruby_version: Option<String>,
        #[serde(default, skip_serializing)]
        python_version: Option<String>,
    },
    Docker {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        compose_port: Option<u16>,
        compose_wait_timeout: u16,
    },
}

impl RemoteRuntime {
    #[must_use]
    pub fn backend(&self) -> RuntimeBackend {
        match self {
            Self::Native { .. } => RuntimeBackend::Native,
            Self::Docker { .. } => RuntimeBackend::Docker,
        }
    }

    #[must_use]
    pub fn compose_port(&self) -> Option<u16> {
        match self {
            Self::Native { .. } => None,
            Self::Docker { compose_port, .. } => *compose_port,
        }
    }

    #[must_use]
    pub fn compose_wait_timeout(&self) -> Option<u16> {
        match self {
            Self::Native { .. } => None,
            Self::Docker { compose_wait_timeout, .. } => Some(*compose_wait_timeout),
        }
    }

    fn from_runtime(runtime: &Runtime) -> Self {
        match runtime.backend {
            RuntimeBackend::Native => {
                Self::Native { web_root: runtime.web_root.clone(), ruby_version: None, python_version: None }
            }
            RuntimeBackend::Docker => {
                Self::Docker { compose_port: runtime.compose_port, compose_wait_timeout: runtime.compose_wait_timeout }
            }
        }
    }

    fn into_runtime(self) -> Runtime {
        match self {
            Self::Native { web_root, .. } => Runtime { web_root, ..Runtime::default() },
            Self::Docker { compose_port, compose_wait_timeout } => {
                Runtime { backend: RuntimeBackend::Docker, compose_port, compose_wait_timeout, ..Runtime::default() }
            }
        }
    }

    fn validate(&self) -> Result<()> {
        if let Self::Native { web_root, .. } = self
            && Path::new(web_root)
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
        {
            bail!("web_root must remain within the release")
        }
        if self.compose_port() == Some(0) {
            bail!("compose_port must be between 1 and 65535")
        }
        if let Some(timeout) = self.compose_wait_timeout()
            && !(1..=3600).contains(&timeout)
        {
            bail!("compose_wait_timeout must be between 1 and 3600")
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteDeploymentConfig {
    pub releases_keep: usize,
    pub runtime: RemoteRuntime,
}

impl RemoteDeploymentConfig {
    #[must_use]
    pub fn from_bones(config: &Bones) -> Self {
        Self { releases_keep: config.releases_keep, runtime: RemoteRuntime::from_runtime(&config.runtime) }
    }

    /// Validates the transported runtime at the remote descriptor boundary.
    pub fn validate(&self) -> Result<()> {
        self.runtime.validate()
    }

    #[must_use]
    pub fn into_site_config(self, site: &str) -> Bones {
        let mut config = Bones::for_site(site);
        config.releases_keep = self.releases_keep;
        config.runtime = self.runtime.into_runtime();
        config
    }
}
