use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config;
use bonesdeploy_core::paths;
use serde::{Deserialize, Serialize};

use super::archive::{ArchiveLimits, available_bytes, extract_archive};
use super::environment::{copy_live_environment, validate_live_environment};
use super::filesystem::{
    chown_tree_without_following, create_transaction_directory, exchange_directories, real_directory_metadata,
    sync_parent, verify_exchange_support,
};
use crate::commands::{ensure_site_idle, service};
use crate::release::SiteMutation;
use crate::release::lifecycle::build::ownership;
use crate::release::state;

const MAX_ARCHIVE_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const SPACE_RESERVE_BYTES: u64 = 512 * 1024 * 1024;
const COPY_BUFFER_BYTES: usize = 64 * 1024;
const TRANSACTION_STATE_FILE: &str = "shared-import.json";

pub trait SiteLifecycle {
    fn stop(&mut self) -> Result<()>;
    fn start_and_verify(&mut self) -> Result<()>;
}

struct SystemdLifecycle<'a> {
    mutation: &'a SiteMutation,
}

impl SiteLifecycle for SystemdLifecycle<'_> {
    fn stop(&mut self) -> Result<()> {
        service::stop(self.mutation)
    }

    fn start_and_verify(&mut self) -> Result<()> {
        service::run(self.mutation)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum Phase {
    Staging,
    Prepared,
    Exchanged,
    Committed,
    Aborted,
}

#[derive(Debug, Deserialize, Serialize)]
struct TransactionState {
    site: String,
    transaction_dir: PathBuf,
    live: PathBuf,
    replacement: PathBuf,
    live_device: u64,
    live_inode: u64,
    replacement_device: u64,
    replacement_inode: u64,
    phase: Phase,
}

pub struct ImportTransaction {
    state_path: PathBuf,
    state: TransactionState,
}

impl ImportTransaction {
    pub fn begin(site: &str, live: PathBuf, replacement: PathBuf, state_path: PathBuf) -> Result<Self> {
        if state_path.try_exists()? {
            bail!("A shared import transaction is already recorded at {}", state_path.display());
        }
        let transaction_dir =
            replacement.parent().context("Shared import replacement has no transaction directory")?.to_path_buf();
        let live_metadata = real_directory_metadata(&live)?;
        let replacement_metadata = real_directory_metadata(&replacement)?;
        if live_metadata.dev() != replacement_metadata.dev() {
            bail!("Shared import staging and live directories are not on the same filesystem");
        }
        let transaction = Self {
            state_path,
            state: TransactionState {
                site: site.to_string(),
                transaction_dir,
                live,
                replacement,
                live_device: live_metadata.dev(),
                live_inode: live_metadata.ino(),
                replacement_device: replacement_metadata.dev(),
                replacement_inode: replacement_metadata.ino(),
                phase: Phase::Staging,
            },
        };
        transaction.persist()?;
        Ok(transaction)
    }

    pub fn load(state_path: PathBuf, expected_site: &str, expected_live: &Path) -> Result<Self> {
        let state_metadata = fs::symlink_metadata(&state_path)
            .with_context(|| format!("Failed to inspect interrupted shared import state {}", state_path.display()))?;
        if state_metadata.file_type().is_symlink() || !state_metadata.is_file() {
            bail!("Interrupted shared import state is not a real file");
        }
        let bytes = fs::read(&state_path)
            .with_context(|| format!("Failed to read interrupted shared import state {}", state_path.display()))?;
        let transaction_state: TransactionState =
            serde_json::from_slice(&bytes).context("Interrupted shared import state is invalid")?;
        if transaction_state.site != expected_site {
            bail!("Interrupted shared import state belongs to a different site");
        }
        if transaction_state.live != expected_live {
            bail!("Interrupted shared import state does not match the site's shared directory");
        }
        let project_root = expected_live.parent().context("Site shared directory has no project root")?;
        if transaction_state.transaction_dir.parent() != Some(project_root)
            || !transaction_state
                .transaction_dir
                .file_name()
                .is_some_and(|name| name.as_encoded_bytes().starts_with(b".shared-import-"))
            || transaction_state.replacement != transaction_state.transaction_dir.join("replacement")
        {
            bail!("Interrupted shared import state contains unsafe transaction paths");
        }
        real_directory_metadata(&transaction_state.transaction_dir)?;
        Ok(Self { state_path, state: transaction_state })
    }

    pub fn mark_prepared(&mut self) -> Result<()> {
        self.state.phase = Phase::Prepared;
        self.persist()
    }

    pub fn cutover(
        &mut self,
        lifecycle: &mut dyn SiteLifecycle,
        prepare_exchange: impl FnOnce() -> Result<()>,
    ) -> Result<()> {
        if self.state.phase != Phase::Prepared {
            bail!("Shared import transaction is not prepared for cutover");
        }

        lifecycle.stop().context("Failed to stop site services for shared import")?;
        if let Err(error) = prepare_exchange() {
            return self.restore_service_after_pre_exchange_failure(lifecycle, error);
        }
        if let Err(error) = exchange_directories(&self.state.live, &self.state.replacement) {
            return self.restore_service_after_pre_exchange_failure(lifecycle, error);
        }
        if let Err(error) = sync_parent(&self.state.live) {
            return self.rollback(lifecycle, error);
        }
        self.state.phase = Phase::Exchanged;
        if let Err(error) = self.persist() {
            return self.rollback(lifecycle, error);
        }

        if let Err(import_error) = lifecycle.start_and_verify() {
            return self.rollback(lifecycle, import_error);
        }

        self.state.phase = Phase::Committed;
        self.persist().with_context(|| {
            format!(
                "Imported services are verified, but completion state could not be persisted; transaction retained at {} for recovery",
                self.state.transaction_dir.display()
            )
        })?;
        self.cleanup()?;
        Ok(())
    }

    fn restore_service_after_pre_exchange_failure(
        &mut self,
        lifecycle: &mut dyn SiteLifecycle,
        error: anyhow::Error,
    ) -> Result<()> {
        if let Err(restart_error) = lifecycle.start_and_verify() {
            return Err(error.context(format!(
                "Shared import stopped before cutover and the original services failed to restart: {restart_error:#}; transaction retained at {}",
                self.state.transaction_dir.display()
            )));
        }
        self.state.phase = Phase::Aborted;
        self.persist()?;
        self.cleanup()?;
        Err(error.context("Shared import stopped before cutover; the original site was restarted"))
    }

    fn rollback(&mut self, lifecycle: &mut dyn SiteLifecycle, import_error: anyhow::Error) -> Result<()> {
        if let Err(exchange_error) = exchange_directories(&self.state.live, &self.state.replacement) {
            return Err(import_error.context(format!(
                "Imported services failed and the previous shared directory could not be restored: {exchange_error:#}; transaction retained at {}",
                self.state.transaction_dir.display()
            )));
        }
        let sync_result = sync_parent(&self.state.live);
        self.state.phase = Phase::Prepared;
        let persist_result = self.persist();
        if let Err(restart_error) = lifecycle.start_and_verify() {
            return Err(import_error.context(format!(
                "Imported services failed; the previous shared directory was restored but its services failed to restart: {restart_error:#}; transaction retained at {}",
                self.state.transaction_dir.display()
            )));
        }
        sync_result.context("Previous shared directory was restored, but its parent could not be synced")?;
        persist_result.context("Previous shared directory was restored, but recovery state could not be synced")?;
        self.state.phase = Phase::Aborted;
        self.persist()?;
        self.cleanup()?;
        Err(import_error.context("Imported services failed; restored the previous shared directory and services"))
    }

    pub fn cleanup(&self) -> Result<()> {
        if self.state.transaction_dir.try_exists()? {
            fs::remove_dir_all(&self.state.transaction_dir).with_context(|| {
                format!("Failed to remove shared import transaction {}", self.state.transaction_dir.display())
            })?;
        }
        if self.state_path.try_exists()? {
            fs::remove_file(&self.state_path)
                .with_context(|| format!("Failed to remove shared import state {}", self.state_path.display()))?;
            sync_parent(&self.state_path)?;
        }
        Ok(())
    }

    pub fn recover(mut self, lifecycle: &mut dyn SiteLifecycle) -> Result<()> {
        if matches!(self.state.phase, Phase::Committed | Phase::Staging | Phase::Aborted) {
            return self.cleanup();
        }

        let exchanged = self.exchange_has_occurred()?;
        if self.state.phase == Phase::Exchanged && exchanged {
            if let Err(error) = lifecycle.start_and_verify() {
                return self.rollback(lifecycle, error.context("Interrupted imported services failed verification"));
            }
            self.state.phase = Phase::Committed;
            self.persist()?;
            return self.cleanup();
        }

        if exchanged {
            exchange_directories(&self.state.live, &self.state.replacement)
                .context("Failed to restore the previous shared directory during import recovery")?;
            sync_parent(&self.state.live)?;
        }
        lifecycle.start_and_verify().with_context(|| {
            format!(
                "Previous shared directory was restored but services could not be recovered; transaction retained at {}",
                self.state.transaction_dir.display()
            )
        })?;
        self.cleanup()
    }

    fn persist(&self) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(&self.state).context("Failed to serialize shared import state")?;
        state::atomic_write(&self.state_path, &bytes)
            .with_context(|| format!("Failed to persist shared import state for {}", self.state.site))
    }
}

pub fn run(mutation: &SiteMutation, reader: &mut dyn Read) -> Result<()> {
    let mut lifecycle = SystemdLifecycle { mutation };
    recover_existing(mutation, &mut lifecycle)?;
    ensure_site_idle(mutation)?;

    let project_root = PathBuf::from(&mutation.config().project_root);
    real_directory_metadata(&project_root)?;
    let live = mutation.shared_dir();
    real_directory_metadata(&live)?;

    let runtime_group = config::runtime_group_for(mutation.site());
    let runtime_gid = ownership::site_group_gid(&runtime_group)?;
    let environment = live.join(paths::DOT_ENV);
    validate_live_environment(&environment, runtime_gid)?;

    let transaction_dir = create_transaction_directory(&project_root)?;
    let replacement = transaction_dir.join("replacement");
    fs::create_dir(&replacement)
        .with_context(|| format!("Failed to create shared import staging directory {}", replacement.display()))?;
    fs::set_permissions(&replacement, fs::Permissions::from_mode(0o700))?;
    let state_path = transaction_state_path(mutation.site());
    let mut transaction = ImportTransaction::begin(mutation.site(), live, replacement.clone(), state_path)?;

    let staging_result = stage_archive(reader, &transaction_dir, &replacement, mutation.site());
    if let Err(error) = staging_result {
        transaction.cleanup()?;
        return Err(error);
    }
    verify_exchange_support(&transaction_dir)?;
    transaction.mark_prepared()?;

    transaction.cutover(&mut lifecycle, || {
        copy_live_environment(&environment, &replacement.join(paths::DOT_ENV), runtime_gid)
    })?;
    println!("Imported shared data for {} and restarted its services", mutation.site());
    Ok(())
}

pub fn recover(mutation: &SiteMutation) -> Result<()> {
    let mut lifecycle = SystemdLifecycle { mutation };
    recover_existing(mutation, &mut lifecycle)
}

fn stage_archive(reader: &mut dyn Read, transaction_dir: &Path, replacement: &Path, site: &str) -> Result<()> {
    let archive_path = transaction_dir.join("archive.zip");
    receive_archive(reader, &archive_path, transaction_dir)?;
    let archive = File::open(&archive_path).context("Failed to open received shared import archive")?;
    extract_archive(archive, replacement, ArchiveLimits::default())?;
    let user = config::runtime_user_for(site);
    let group = config::runtime_group_for(site);
    let uid = ownership::user_uid(&user)?;
    let gid = ownership::site_group_gid(&group)?;
    chown_tree_without_following(replacement, uid, gid)
}

fn receive_archive(reader: &mut dyn Read, archive_path: &Path, filesystem_path: &Path) -> Result<()> {
    let mut archive = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(archive_path)
        .with_context(|| format!("Failed to create shared import archive {}", archive_path.display()))?;
    fs::set_permissions(archive_path, fs::Permissions::from_mode(0o600))?;
    let mut received = 0_u64;
    let mut buffer = [0_u8; COPY_BUFFER_BYTES];
    loop {
        let read = reader.read(&mut buffer).context("Failed to receive shared import archive")?;
        if read == 0 {
            break;
        }
        received = received.checked_add(read as u64).context("Shared import archive size overflow")?;
        if received > MAX_ARCHIVE_BYTES {
            bail!("Shared import archive exceeds {MAX_ARCHIVE_BYTES} bytes");
        }
        let available = available_bytes(filesystem_path)?;
        if available < SPACE_RESERVE_BYTES + read as u64 {
            bail!("Shared import stopped to preserve its {SPACE_RESERVE_BYTES}-byte filesystem safety reserve");
        }
        archive.write_all(&buffer[..read]).context("Failed to store shared import archive")?;
    }
    archive.flush().context("Failed to flush shared import archive")?;
    archive.sync_all().context("Failed to sync shared import archive")
}

fn recover_existing(mutation: &SiteMutation, lifecycle: &mut dyn SiteLifecycle) -> Result<()> {
    let state_path = transaction_state_path(mutation.site());
    if !state_path.try_exists()? {
        return Ok(());
    }
    ImportTransaction::load(state_path, mutation.site(), &mutation.shared_dir())?.recover(lifecycle)
}

impl ImportTransaction {
    fn exchange_has_occurred(&self) -> Result<bool> {
        let live = real_directory_metadata(&self.state.live)?;
        let replacement = real_directory_metadata(&self.state.replacement)?;
        if live.dev() == self.state.replacement_device
            && live.ino() == self.state.replacement_inode
            && replacement.dev() == self.state.live_device
            && replacement.ino() == self.state.live_inode
        {
            return Ok(true);
        }
        if live.dev() == self.state.live_device
            && live.ino() == self.state.live_inode
            && replacement.dev() == self.state.replacement_device
            && replacement.ino() == self.state.replacement_inode
        {
            return Ok(false);
        }
        bail!("Shared import paths no longer match the durable transaction state; refusing automatic recovery")
    }
}

fn transaction_state_path(site: &str) -> PathBuf {
    state::resolved_site_root(site).join(TRANSACTION_STATE_FILE)
}
