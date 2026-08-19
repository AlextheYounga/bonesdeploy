use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// Explicit, persisted deployment phases.
///
/// `created` … `sealed` are the pre-cut-over phases: the new release is
/// prepared and validated while the old release still serves. `activated` /
/// `verified` are the cut-over (the commit point); once committed, a site is
/// serialization-idle even while post-commit maintenance (`prune`, temp
/// cleanup) is still pending. `cleanup_pending` records a post-commit
/// maintenance failure without blocking the next deployment, and `failed` is a
/// pre-commit failure that is being aborted (the record is cleared when the
/// site returns to idle).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentPhase {
    Created,
    SourceExported,
    Built,
    Promoted,
    Prepared,
    Sealed,
    Activated,
    Verified,
    Completed,
    CleanupPending,
    Failed,
}

impl DeploymentPhase {
    /// Whether the phase is at or after the cut-over commit point. A site with a
    /// committed record is serialization-idle: a next deployment may proceed
    /// even while the record remains (e.g. `cleanup_pending`).
    pub fn is_committed(&self) -> bool {
        matches!(self, Self::Activated | Self::Verified | Self::Completed | Self::CleanupPending)
    }

    /// Whether prepare scripts or the cut-over may already have mutated runtime
    /// state, so cancellation is refused without an explicit compatibility
    /// policy. `promoted` is included because prepare scripts run while the
    /// record still reads `promoted` (the phase is advanced only once they
    /// succeed).
    pub fn may_have_mutated_runtime(&self) -> bool {
        matches!(
            self,
            Self::Promoted
                | Self::Prepared
                | Self::Sealed
                | Self::Activated
                | Self::Verified
                | Self::Completed
                | Self::CleanupPending
        )
    }
}

/// The persisted record of an in-flight (or just-aborted) deployment.
///
/// The record is the authoritative model for status, cancellation, crash
/// recovery, and idle checks. It is stored inside the per-site `SiteState` and
/// is cleared (or left as `cleanup_pending`) when the site returns to idle.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeploymentRecord {
    release: String,
    source_revision: String,
    phase: DeploymentPhase,
    #[serde(flatten)]
    process_identity: ProcessIdentity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    previous_release: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    /// Reserved keys: populated by later phases (config revision + protocol
    /// version in Phase 5, build-image digest in Phase 4). Kept in the schema
    /// now so the persisted format is stable across releases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    config_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    protocol_version: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    image_digest: Option<String>,
}

/// Identifies the process that owns an in-flight deployment.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessIdentity {
    pid: u32,
    process_start_ticks: u64,
    started_at: String,
}

impl ProcessIdentity {
    pub fn new(pid: u32, process_start_ticks: u64, started_at: String) -> Self {
        Self { pid, process_start_ticks, started_at }
    }
}

impl DeploymentRecord {
    pub fn release(&self) -> &str {
        &self.release
    }
    pub fn phase(&self) -> &DeploymentPhase {
        &self.phase
    }
    pub fn pid(&self) -> u32 {
        self.process_identity.pid
    }
    pub fn process_start_ticks(&self) -> u64 {
        self.process_identity.process_start_ticks
    }
    pub fn started_at(&self) -> &str {
        &self.process_identity.started_at
    }
    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }
    pub fn set_context(&mut self, context: String) {
        self.context = Some(context);
    }
    pub fn set_previous_release(&mut self, release: Option<String>) {
        self.previous_release = release;
    }
    pub fn set_phase(&mut self, phase: DeploymentPhase) {
        self.phase = phase;
    }
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    pub fn new(
        release: String,
        source_revision: String,
        phase: DeploymentPhase,
        process_identity: ProcessIdentity,
    ) -> Self {
        Self {
            release,
            source_revision,
            phase,
            process_identity,
            previous_release: None,
            context: None,
            error: None,
            config_revision: None,
            protocol_version: None,
            image_digest: None,
        }
    }
}

/// Previous `active-deployment.json` shape written by older `bonesremote`
/// versions, used only to migrate state into the centralized store. The old
/// schema only ever persisted the `building` / `preparing` phases.
#[derive(Deserialize)]
pub struct PreviousDeployment {
    pub release: String,
    pub pid: u32,
    #[serde(default)]
    pub process_start_ticks: u64,
    #[serde(default)]
    pub phase: String,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub context: Option<String>,
}

impl PreviousDeployment {
    pub fn to_record(self) -> Result<DeploymentRecord> {
        let phase = match self.phase.as_str() {
            "building" => DeploymentPhase::Created,
            "preparing" => DeploymentPhase::Prepared,
            other => bail!("Unknown previous deployment phase '{other}'"),
        };
        Ok(DeploymentRecord {
            release: self.release,
            source_revision: String::new(),
            phase,
            process_identity: ProcessIdentity::new(self.pid, self.process_start_ticks, self.started_at),
            previous_release: None,
            context: self.context,
            error: None,
            config_revision: None,
            protocol_version: None,
            image_digest: None,
        })
    }
}
