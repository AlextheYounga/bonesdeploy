use bonesdeploy_core::names;

use std::collections::BTreeSet;
use std::path::{Component, Path};

use anyhow::{Result, bail};
use bonesdeploy_core::paths;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::ProvisioningContext;

mod declarations;
mod inspection;

pub use declarations::{Artifact, ManagedService, declarations};
pub use inspection::{
    ArtifactState, ComposeReport, ComposeService, ManifestReport, ManifestStrategy, ResolvedArtifact, ResolvedService,
    inspect,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    File,
    Directory,
    Link,
    Socket,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeletionArtifact {
    pub name: String,
    pub path: String,
    pub kind: ArtifactKind,
    pub owner: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeletionService {
    pub name: String,
    pub unit: String,
    pub owner: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeletionPlan {
    pub artifacts: Vec<DeletionArtifact>,
    pub services: Vec<DeletionService>,
}

impl DeletionPlan {
    /// Resolves and validates the current built-in manifest into the exact
    /// secret-free inventory permitted for deletion.
    ///
    /// # Errors
    /// Returns an error for an unknown framework, invalid framework setting,
    /// unsafe artifact path, or unsafe service unit.
    pub fn resolve(context: &ProvisioningContext) -> Result<Self> {
        let (artifacts, services, _) = declarations(context)?;
        let retained = [
            context.paths.backup_repository.as_str(),
            context.paths.backup_passphrase_file.as_str(),
            context.paths.backup_cron_file.as_str(),
        ];
        let mut artifacts = artifacts
            .into_iter()
            .filter(|artifact| !retained.contains(&artifact.path.as_str()))
            .map(|artifact| DeletionArtifact {
                name: artifact.name,
                path: artifact.path,
                kind: artifact.kind,
                owner: artifact.owner,
            })
            .collect::<Vec<_>>();
        artifacts.extend([
            deletion_artifact(
                "scheduled backup cron entry",
                &context.paths.backup_cron_file,
                ArtifactKind::File,
                "backup",
            ),
            deletion_artifact(
                "quick tunnel service",
                &context.paths.systemd_cloudflared_service,
                ArtifactKind::File,
                "tunnel",
            ),
            deletion_artifact(
                "quick tunnel nginx route",
                &context.paths.nginx_cloudflared_site_available,
                ArtifactKind::File,
                "tunnel",
            ),
            deletion_artifact(
                "enabled quick tunnel nginx route",
                &context.paths.nginx_cloudflared_site_enabled,
                ArtifactKind::Link,
                "tunnel",
            ),
        ]);
        deduplicate_deletion_artifacts(&mut artifacts);

        let mut services = services
            .into_iter()
            .map(|service| DeletionService { name: service.name, unit: service.unit, owner: service.owner })
            .collect::<Vec<_>>();
        services.push(DeletionService {
            name: "quick tunnel".to_owned(),
            unit: names::service_unit(&context.project_name, "cloudflared"),
            owner: "tunnel".to_owned(),
        });
        deduplicate_deletion_services(&mut services);
        let plan = Self { artifacts, services };
        plan.validate(context)?;
        Ok(plan)
    }

    /// Parses the stable persisted JSON format and revalidates every path and
    /// unit against the current site before it may be executed.
    ///
    /// # Errors
    /// Returns an error for malformed JSON, unknown fields, unsafe paths, or
    /// unsafe service names.
    pub fn parse(value: &str, context: &ProvisioningContext) -> Result<Self> {
        let plan: Self =
            serde_json::from_str(value).map_err(|error| anyhow::anyhow!("invalid persisted deletion plan: {error}"))?;
        plan.validate(context)?;
        Ok(plan)
    }

    /// Serializes the stable preflight representation consumed by BonesDeploy.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    fn validate(&self, context: &ProvisioningContext) -> Result<()> {
        for artifact in &self.artifacts {
            validate_deletion_artifact(context, artifact)?;
        }
        for service in &self.services {
            validate_deletion_service(context, service)?;
        }
        Ok(())
    }
}

fn validate_deletion_artifact(context: &ProvisioningContext, artifact: &DeletionArtifact) -> Result<()> {
    let path = Path::new(&artifact.path);
    if !path.is_absolute() || path.components().any(|component| component == Component::ParentDir) {
        bail!("unsafe deletion artifact path: {}", artifact.path);
    }
    for retained in [&context.paths.backup_repository, &context.paths.backup_passphrase_file] {
        let retained = Path::new(retained);
        if within(path, retained) || within(retained, path) {
            bail!("deletion artifact would remove retained backup data: {}", artifact.path);
        }
    }
    let roots = [
        &context.paths.project_root,
        &context.paths.conf_root,
        &context.paths.site_root,
        &context.paths.runtime_socket_dir,
        &context.paths.site_log_dir,
        &context.paths.acme_webroot,
        &context.paths.backup_repository,
    ];
    if roots.iter().any(|root| within(path, Path::new(root))) || derived_system_path(context, path)? {
        return Ok(());
    }
    bail!("deletion artifact is outside site-owned paths: {}", artifact.path)
}

fn validate_deletion_service(context: &ProvisioningContext, service: &DeletionService) -> Result<()> {
    let pattern = format!(r"^{}-[a-z0-9][a-z0-9_-]*\.service$", regex::escape(&context.project_name));
    if Regex::new(&pattern)?.is_match(&service.unit) {
        return Ok(());
    }
    bail!("unsafe deletion service: {}", service.unit)
}

fn derived_system_path(context: &ProvisioningContext, path: &Path) -> Result<bool> {
    let p = &context.paths;
    let value = path.to_string_lossy();
    let allowed = [
        &p.nginx_site_available,
        &p.nginx_site_enabled,
        &p.systemd_site_target,
        &p.systemd_site_target_requires,
        &p.systemd_site_nginx_service,
        &p.systemd_site_nginx_requirement,
        &p.systemd_cloudflared_service,
        &p.nginx_cloudflared_site_available,
        &p.nginx_cloudflared_site_enabled,
        &p.nginx_apparmor_profile,
        &p.backup_cron_file,
    ];
    if allowed.iter().any(|allowed| value == allowed.as_str())
        || within(path, Path::new(&p.systemd_site_target_requires))
        || value == paths::ssl_certificate_path(&context.domain)
        || value == paths::ssl_certificate_key_path(&context.domain)
    {
        return Ok(true);
    }
    for prefix in [
        format!("/etc/systemd/system/{}-", context.project_name),
        format!("/etc/apparmor.d/bonesdeploy-{}-", context.project_name),
        format!("/etc/bonesinfra/services/{}-", context.project_name),
    ] {
        if value.starts_with(&prefix) {
            return Ok(true);
        }
    }
    let project = regex::escape(&context.project_name);
    let php_pool = Regex::new(&format!(r"^/etc/php/[0-9]+(?:\.[0-9]+)?/fpm/pool\.d/{project}\.conf$"))?;
    let php_socket = Regex::new(&format!(r"^/run/php/php[0-9]+(?:\.[0-9]+)?-fpm-{project}\.sock$"))?;
    if php_pool.is_match(&value) || php_socket.is_match(&value) {
        return Ok(true);
    }
    Ok(["mysql", "mariadb", "mongodb", "redis", "valkey"]
        .iter()
        .any(|service| within(path, Path::new(&format!("/var/lib/{service}/{}", context.project_name)))))
}

fn within(path: &Path, root: &Path) -> bool {
    path == root || path.starts_with(root)
}

fn deletion_artifact(name: &str, path: &str, kind: ArtifactKind, owner: &str) -> DeletionArtifact {
    DeletionArtifact { name: name.to_owned(), path: path.to_owned(), kind, owner: owner.to_owned() }
}

fn deduplicate_deletion_artifacts(artifacts: &mut Vec<DeletionArtifact>) {
    let mut paths = BTreeSet::new();
    artifacts.retain(|artifact| paths.insert(artifact.path.clone()));
}

fn deduplicate_deletion_services(services: &mut Vec<DeletionService>) {
    let mut units = BTreeSet::new();
    services.retain(|service| units.insert(service.unit.clone()));
}
