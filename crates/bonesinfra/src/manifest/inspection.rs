use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use bonesdeploy_core::config::{Bones, RuntimeBackend};
use rustinfra::connectors::Connector;
use rustinfra::facts::files::{Directory, File, Link, PathInfo, PathInput};
use rustinfra::facts::server::{Command, Which};
use rustinfra::facts::systemd::{SystemdEnabled, SystemdInput, SystemdStatus};
use rustinfra::facts::{FactExecutor, FactOptions};
use rustinfra::operations::{OperationOptions, StatefulOperation, StatefulOperationFuture};
use serde::{Deserialize, Serialize};

use super::{Artifact, ArtifactKind, ManagedService, declarations};
use crate::shell_quote;
use crate::{ProvisioningContext, ProvisioningRuntime, reject_custom_provisioning};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactState {
    Present,
    Missing,
    WrongKind,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolvedArtifact {
    pub name: String,
    pub path: String,
    pub kind: ArtifactKind,
    pub owner: String,
    pub state: ArtifactState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_kind: Option<ArtifactKind>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolvedService {
    pub name: String,
    pub unit: String,
    pub owner: String,
    pub running: bool,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManifestStrategy {
    pub framework: String,
    pub mode: String,
    pub backend: String,
    pub ssl: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ComposeService {
    pub service: String,
    pub condition: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ComposeReport {
    pub project_name: String,
    pub files: Vec<String>,
    pub services: Vec<ComposeService>,
    pub engine_available: bool,
    pub plugin_available: bool,
    pub ingress_port: Option<u16>,
    pub wait_timeout: u16,
    pub security_mode: String,
    pub security_warning: String,
    pub persistent_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ManifestReport {
    pub strategy: ManifestStrategy,
    pub entries: Vec<ResolvedArtifact>,
    pub managed_services: Vec<ResolvedService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compose: Option<ComposeReport>,
}

/// Inspects the declared project resources through RustInfra facts.
///
/// # Errors
/// Returns an error for existing custom provisioning, invalid declarations,
/// connection/fact failures, or an incomplete inspection result.
pub async fn inspect(config: &Bones, project_root: &Path) -> Result<ManifestReport> {
    reject_custom_provisioning(project_root)?;
    let context = ProvisioningContext::from_bones(config)?;
    let (artifacts, services, mode) = declarations(&context)?;
    let output = Arc::new(Mutex::new(None));
    let runtime = ProvisioningRuntime::new(&context.server)?;
    let input = InspectionInput { context: context.clone(), artifacts, services, mode };
    let deploy = runtime.prepare(|host| {
        host.add_stateful(
            ManifestInspection { output: Arc::clone(&output) },
            input.clone(),
            OperationOptions { name: Some("Inspect site manifest".to_owned()), ..OperationOptions::default() },
        )?;
        Ok(())
    })?;
    runtime.execute(&deploy).await?;
    output
        .lock()
        .map_err(|_| anyhow::anyhow!("manifest inspection result lock was poisoned"))?
        .take()
        .context("RustInfra completed without a manifest inspection result")
}

#[derive(Clone)]
struct InspectionInput {
    context: ProvisioningContext,
    artifacts: Vec<Artifact>,
    services: Vec<ManagedService>,
    mode: String,
}

struct ManifestInspection {
    output: Arc<Mutex<Option<ManifestReport>>>,
}

impl StatefulOperation for ManifestInspection {
    type Input = InspectionInput;

    fn name(&self) -> &'static str {
        "bonesinfra.manifest_inspection"
    }

    fn is_idempotent(&self) -> bool {
        true
    }

    fn commands<'a>(
        &'a self,
        input: &'a Self::Input,
        connector: &'a dyn Connector,
        options: &'a FactOptions,
    ) -> StatefulOperationFuture<'a> {
        Box::pin(async move {
            let facts = FactExecutor::new(connector, options);
            let entries = inspect_artifacts(&facts, &input.artifacts).await?;
            let managed_services = inspect_services(&facts, &input.services).await?;
            let compose = if input.context.backend == RuntimeBackend::Docker {
                Some(inspect_compose(&facts, &input.context).await?)
            } else {
                None
            };
            let ssl_entries = entries.iter().filter(|entry| entry.owner == "ssl").collect::<Vec<_>>();
            let report = ManifestReport {
                strategy: ManifestStrategy {
                    backend: match input.context.backend {
                        RuntimeBackend::Native => "native",
                        RuntimeBackend::Docker => "docker",
                    }
                    .to_owned(),
                    framework: if input.context.framework.is_empty() {
                        "none".to_owned()
                    } else {
                        input.context.framework.clone()
                    },
                    mode: input.mode.clone(),
                    ssl: !ssl_entries.is_empty()
                        && ssl_entries.iter().all(|entry| entry.state == ArtifactState::Present),
                },
                entries,
                managed_services,
                compose,
            };
            *self.output.lock().map_err(|_| "manifest inspection result lock was poisoned")? = Some(report);
            Ok(Vec::new())
        })
    }
}

async fn inspect_artifacts(
    facts: &FactExecutor<'_>,
    artifacts: &[Artifact],
) -> Result<Vec<ResolvedArtifact>, Box<dyn Error + Send + Sync>> {
    let mut entries = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        let path = PathInput::from(artifact.path.clone());
        let kinds = [
            (ArtifactKind::File, matches!(facts.get(&File, &path).await?, PathInfo::Present(_))),
            (ArtifactKind::Directory, matches!(facts.get(&Directory, &path).await?, PathInfo::Present(_))),
            (ArtifactKind::Link, matches!(facts.get(&Link, &path).await?, PathInfo::Present(_))),
            (
                ArtifactKind::Socket,
                facts
                    .get(
                        &Command,
                        &format!("test -S {} && echo true || echo false", shell_quote::quote(&artifact.path)?),
                    )
                    .await?
                    .as_deref()
                    == Some("true"),
            ),
        ];
        let actual_kind = kinds.into_iter().find_map(|(kind, present)| present.then_some(kind));
        let state = match actual_kind {
            Some(kind) if kind == artifact.kind => ArtifactState::Present,
            Some(_) => ArtifactState::WrongKind,
            None => ArtifactState::Missing,
        };
        entries.push(ResolvedArtifact {
            name: artifact.name.clone(),
            path: artifact.path.clone(),
            kind: artifact.kind,
            owner: artifact.owner.clone(),
            state,
            actual_kind: (state == ArtifactState::WrongKind).then_some(actual_kind).flatten(),
        });
    }
    Ok(entries)
}

async fn inspect_services(
    facts: &FactExecutor<'_>,
    services: &[ManagedService],
) -> Result<Vec<ResolvedService>, Box<dyn Error + Send + Sync>> {
    let input = SystemdInput {
        services: services.iter().map(|service| service.unit.clone()).collect(),
        ..SystemdInput::default()
    };
    let running = facts.get(&SystemdStatus, &input).await?;
    let enabled = facts.get(&SystemdEnabled, &input).await?;
    Ok(services
        .iter()
        .map(|service| ResolvedService {
            name: service.name.clone(),
            unit: service.unit.clone(),
            owner: service.owner.clone(),
            running: running.get(&service.unit).copied().unwrap_or(false),
            enabled: enabled.get(&service.unit).copied().unwrap_or(false),
        })
        .collect())
}

async fn inspect_compose(
    facts: &FactExecutor<'_>,
    context: &ProvisioningContext,
) -> Result<ComposeReport, Box<dyn Error + Send + Sync>> {
    let engine = facts.get(&Which, &"docker".to_owned()).await?.is_some_and(|path| !path.is_empty());
    let plugin = facts
        .get(&Command, &"docker compose version >/dev/null 2>&1 && printf available || printf unavailable".to_owned())
        .await?
        .as_deref()
        == Some("available");
    let status_command = format!("bonesremote status --site {}", shell_quote::quote(&context.project_name)?);
    let status = facts.get(&Command, &status_command).await?;
    let (files, services, error) = compose_status(status.as_deref());
    Ok(ComposeReport {
        project_name: format!("bonesdeploy-{}", context.project_name),
        files,
        services,
        engine_available: engine,
        plugin_available: plugin,
        ingress_port: context.compose_port,
        wait_timeout: context.compose_wait_timeout,
        security_mode: "reduced-guarantee".to_owned(),
        security_warning: "The project Compose file is trusted privileged input; native runtime isolation is not guaranteed. BonesDeploy does not add a Docker socket mount.".to_owned(),
        persistent_data: "Compose named volumes persist across deploy and rollback; their data is not rolled back.".to_owned(),
        error,
    })
}

fn compose_status(value: Option<&str>) -> (Vec<String>, Vec<ComposeService>, Option<String>) {
    let Some(value) = value.filter(|value| !value.is_empty()) else {
        return (Vec::new(), Vec::new(), Some("bonesremote status is unavailable".to_owned()));
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(value) else {
        return (Vec::new(), Vec::new(), Some("bonesremote returned invalid Compose status".to_owned()));
    };
    let compose = value.get("compose").cloned().unwrap_or_default();
    let files = compose
        .get("files")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(str::to_owned)
        .collect();
    let services = compose
        .get("services")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|service| serde_json::from_value(service.clone()).ok())
        .collect();
    let error = compose.get("error").and_then(serde_json::Value::as_str).map(str::to_owned);
    (files, services, error)
}
