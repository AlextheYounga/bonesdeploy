use bonesdeploy_core::state;

use std::collections::BTreeMap;

use anyhow::{Result, bail};
use bonesdeploy_core::config::{
    Backup, Bones, RuntimeBackend, runtime_group_for, runtime_user_for, validate_domain, validate_email, validate_host,
    validate_project_name, validate_runtime,
};
use bonesdeploy_core::paths::{self, DeploymentPaths};
use serde_json::{Map, Value, json};

/// SSH connection details used by the native provisioning runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerContext {
    pub host: String,
    pub ssh_user: String,
    pub port: u16,
}

/// Typed provisioning context derived directly from the canonical `Bones`
/// configuration. There is no serialized transport representation between
/// BonesDeploy and BonesInfra.
#[derive(Clone, Debug, PartialEq)]
pub struct ProvisioningContext {
    pub server: ServerContext,
    pub project_name: String,
    pub domain: String,
    pub email: String,
    pub framework: String,
    pub backend: RuntimeBackend,
    pub web_root: String,
    pub node_version: String,
    pub runtime_user: String,
    pub runtime_group: String,
    pub compose_port: Option<u16>,
    pub compose_wait_timeout: u16,
    pub backup: Backup,
    pub extra: BTreeMap<String, Value>,
    pub paths: DeploymentPaths,
}

impl ProvisioningContext {
    /// Builds the provisioning model without crossing a serialization or
    /// subprocess boundary.
    ///
    /// # Errors
    /// Returns an error when the SSH port is invalid or a framework-specific
    /// value is not scalar.
    pub fn from_bones(config: &Bones) -> Result<Self> {
        validate_host(&config.host)?;
        if config.host.is_empty() {
            bail!("Host is required for provisioning");
        }
        validate_project_name(&config.project_name)?;
        validate_runtime(&config.runtime)?;
        if !config.domain.is_empty() {
            validate_domain(&config.domain)?;
        }
        if !config.email.is_empty() {
            validate_email(&config.email)?;
        }
        let port = config.port.parse::<u16>()?;
        if port == 0 {
            bail!("Invalid port: 0");
        }
        let project_name = config.project_name.clone();
        let project_root = if config.project_root.is_empty() {
            paths::default_project_root_for(&project_name)
        } else {
            config.project_root.clone()
        };
        let extra = config
            .runtime
            .extra
            .iter()
            .map(|(key, value)| Ok((key.clone(), scalar_json(key, value)?)))
            .collect::<Result<_>>()?;

        Ok(Self {
            server: ServerContext { host: config.host.clone(), ssh_user: config.ssh_user.clone(), port },
            project_name: project_name.clone(),
            domain: config.domain.clone(),
            email: config.email.clone(),
            framework: config.runtime.template.clone(),
            backend: config.runtime.backend,
            web_root: config.runtime.web_root.clone(),
            node_version: config.runtime.node_version.clone(),
            runtime_user: runtime_user_for(&project_name),
            runtime_group: runtime_group_for(&project_name),
            compose_port: config.runtime.compose_port,
            compose_wait_timeout: config.runtime.compose_wait_timeout,
            backup: config.backup.clone(),
            extra,
            paths: DeploymentPaths::new(&project_name, &project_root, &config.runtime.web_root),
        })
    }

    /// Flat template data used by project-local MiniJinja templates.
    /// # Errors
    /// Returns an error if the typed path model cannot be projected to JSON.
    pub fn template_data(&self) -> Result<Map<String, Value>> {
        let paths = serde_json::to_value(&self.paths)?;
        let mut data = Map::from_iter([
            (state::PROJECT_NAME.to_owned(), json!(self.project_name)),
            ("project_root".to_owned(), json!(self.paths.project_root)),
            (state::WEB_ROOT.to_owned(), json!(self.web_root)),
            (state::NODE_VERSION.to_owned(), json!(self.node_version)),
            ("template".to_owned(), json!(self.framework)),
            (state::DEPLOY_USER.to_owned(), json!(paths::DEPLOY_USER)),
            ("runtime_user".to_owned(), json!(self.runtime_user)),
            ("runtime_group".to_owned(), json!(self.runtime_group)),
            (
                "runtime_backend".to_owned(),
                json!(match self.backend {
                    RuntimeBackend::Native => "native",
                    RuntimeBackend::Docker => "docker",
                }),
            ),
            ("project_root_parent".to_owned(), json!(self.paths.project_root_parent)),
            (state::SSH_PORT.to_owned(), json!(self.server.port)),
            ("paths".to_owned(), paths),
            ("ssl_domain".to_owned(), json!(self.domain)),
            ("ssl_email".to_owned(), json!(self.email)),
        ]);
        for (key, value) in &self.extra {
            data.entry(key.clone()).or_insert_with(|| value.clone());
        }
        Ok(data)
    }
}

impl ServerContext {
    /// Derives the provisioning connection directly from canonical config.
    ///
    /// # Errors
    /// Returns an error when the SSH port is invalid.
    pub fn from_bones(config: &Bones) -> Result<Self> {
        validate_host(&config.host)?;
        if config.host.is_empty() {
            bail!("Host is required for provisioning");
        }
        let port = config.port.parse::<u16>()?;
        if port == 0 {
            bail!("Invalid port: 0");
        }
        Ok(Self { host: config.host.clone(), ssh_user: config.ssh_user.clone(), port })
    }
}

fn scalar_json(key: &str, value: &toml::Value) -> Result<Value> {
    match value {
        toml::Value::String(value) => Ok(json!(value)),
        toml::Value::Integer(value) => Ok(json!(value)),
        toml::Value::Float(value) => serde_json::Number::from_f64(*value)
            .map(Value::Number)
            .ok_or_else(|| anyhow::anyhow!("invalid framework value `{key}`")),
        toml::Value::Boolean(value) => Ok(json!(value)),
        toml::Value::Datetime(value) => Ok(json!(value.to_string())),
        toml::Value::Array(_) | toml::Value::Table(_) => {
            bail!("Runtime framework value `{key}` must be a scalar")
        }
    }
}
