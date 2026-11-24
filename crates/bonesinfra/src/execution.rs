use bonesdeploy_core::names;

use std::time::Duration;

use anyhow::{Context, Result};
use rustinfra::api::{Config, Data, HostResults, HostSpec, Inventory, InventorySpec, State};
use rustinfra::operations::{DeployError, DeployHost, DeployRuntime, ExecutionStrategy, PreparedDeploy};
use serde_json::json;

use crate::ServerContext;

/// Native RustInfra inventory and execution boundary for a single BonesDeploy
/// target host.
pub struct ProvisioningRuntime {
    inventory: Inventory,
    config: Config,
}

impl ProvisioningRuntime {
    /// Creates the inventory used for all provisioning entry points.
    ///
    /// # Errors
    /// Returns an error when the inventory cannot be constructed.
    pub fn new(server: &ServerContext) -> Result<Self> {
        let mut data = Data::new();
        data.insert(names::SSH_USER.to_owned(), json!(server.ssh_user));
        data.insert(names::SSH_PORT.to_owned(), json!(server.port));
        let inventory = Inventory::from_spec(InventorySpec {
            hosts: vec![HostSpec::ssh(&server.host).with_data(data)],
            ..InventorySpec::default()
        })?;
        let mut config = Config::default();
        config.sudo = true;
        Ok(Self { inventory, config })
    }

    /// Evaluates a typed provisioning plan without connecting to the host.
    /// This is the testable planning boundary for provisioning modules.
    ///
    /// # Errors
    /// Returns an error when operation registration or ordering is invalid.
    pub fn prepare(
        &self,
        define: impl FnMut(&mut DeployHost<'_>) -> Result<(), DeployError>,
    ) -> Result<PreparedDeploy> {
        PreparedDeploy::prepare(&self.inventory, define).map_err(Into::into)
    }

    /// Executes a prepared provisioning plan and returns stable operation
    /// accounting for presentation by BonesDeploy.
    ///
    /// # Errors
    /// Returns an error when SSH connection, fact collection, an operation, or
    /// disconnect fails.
    pub async fn execute(&self, deploy: &PreparedDeploy) -> Result<ProvisioningReport> {
        let runtime =
            DeployRuntime::from_inventory(&self.inventory, Duration::from_secs(self.config.connect_timeout_seconds))?;
        let mut state = State::new(self.inventory.clone(), self.config.clone());
        runtime
            .execute(&mut state, deploy, ExecutionStrategy::Lockstep)
            .await
            .context("RustInfra provisioning failed")?;
        Ok(ProvisioningReport::from_state(&state))
    }

    #[must_use]
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
}

/// Stable BonesInfra-facing execution report, independent of RustInfra's
/// internal scheduler representation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProvisioningReport {
    pub hosts: Vec<HostProvisioningReport>,
}

impl ProvisioningReport {
    fn from_state(state: &State) -> Self {
        let hosts = state
            .inventory()
            .hosts()
            .map(|host| HostProvisioningReport::new(host.name(), state.host_results(host.name()).copied()))
            .collect();
        Self { hosts }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HostProvisioningReport {
    pub host: String,
    pub operations: usize,
    pub successful_operations: usize,
    pub error_operations: usize,
    pub ignored_error_operations: usize,
    pub partial_operations: usize,
}

impl HostProvisioningReport {
    fn new(host: &str, results: Option<HostResults>) -> Self {
        let results = results.unwrap_or_default();
        Self {
            host: host.to_owned(),
            operations: results.operations,
            successful_operations: results.successful_operations,
            error_operations: results.error_operations,
            ignored_error_operations: results.ignored_error_operations,
            partial_operations: results.partial_operations,
        }
    }
}
