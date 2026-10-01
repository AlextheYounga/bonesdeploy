use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Result, anyhow, bail};
use bonesremote::shared::{ImportTransaction, SiteLifecycle, exchange_directories};
use serde_json::Value;
use tempfile::tempdir;

struct FakeLifecycle {
    events: Vec<&'static str>,
    start_failures: usize,
    stopped: Rc<Cell<bool>>,
}

impl SiteLifecycle for FakeLifecycle {
    fn stop(&mut self) -> Result<()> {
        self.events.push("stop");
        self.stopped.set(true);
        Ok(())
    }

    fn start_and_verify(&mut self) -> Result<()> {
        self.events.push("start");
        if self.start_failures > 0 {
            self.start_failures -= 1;
            return Err(anyhow!("fake service failed to start"));
        }
        Ok(())
    }
}

fn transaction_fixture() -> Result<(tempfile::TempDir, PathBuf, PathBuf, PathBuf, ImportTransaction)> {
    let root = tempdir()?;
    let live = root.path().join("shared");
    let transaction_dir = root.path().join(".shared-import-test");
    let replacement = transaction_dir.join("replacement");
    let state = root.path().join("shared-import.json");
    fs::create_dir(&live)?;
    fs::create_dir_all(&replacement)?;
    fs::write(live.join("value"), "old")?;
    fs::write(replacement.join("value"), "new")?;
    let transaction = ImportTransaction::begin("site", live.clone(), replacement.clone(), state.clone())?;
    Ok((root, live, replacement, state, transaction))
}

fn mark_state_exchanged(state: &Path) -> Result<()> {
    let bytes = fs::read(state)?;
    let mut document: Value = serde_json::from_slice(&bytes)?;
    document["phase"] = Value::String("Exchanged".to_string());
    fs::write(state, serde_json::to_vec_pretty(&document)?)?;
    Ok(())
}

#[test]
fn successful_cutover_starts_after_exchange_and_cleans_state() -> Result<()> {
    let (_root, live, replacement, state, mut transaction) = transaction_fixture()?;
    transaction.mark_prepared()?;
    let mut lifecycle = FakeLifecycle { events: Vec::new(), start_failures: 0, stopped: Rc::new(Cell::new(false)) };

    transaction.cutover(&mut lifecycle, || Ok(()))?;

    assert_eq!(lifecycle.events, ["stop", "start"]);
    assert_eq!(fs::read_to_string(live.join("value"))?, "new");
    assert!(!replacement.exists());
    assert!(!state.exists());
    Ok(())
}

#[test]
fn prepare_exchange_runs_after_stop_before_exchange() -> Result<()> {
    let (_root, live, replacement, state, mut transaction) = transaction_fixture()?;
    transaction.mark_prepared()?;
    let mut lifecycle = FakeLifecycle { events: Vec::new(), start_failures: 0, stopped: Rc::new(Cell::new(false)) };
    let stopped = lifecycle.stopped.clone();

    transaction.cutover(&mut lifecycle, || {
        assert!(stopped.get());
        assert_eq!(fs::read_to_string(live.join("value"))?, "old");
        fs::write(replacement.join(".env"), "SECRET=preserved\n")?;
        Ok(())
    })?;

    assert_eq!(fs::read_to_string(live.join("value"))?, "new");
    assert_eq!(fs::read_to_string(live.join(".env"))?, "SECRET=preserved\n");
    assert!(!state.exists());
    Ok(())
}

#[test]
fn failed_start_rolls_back_and_cleans_state_after_service_recovery() -> Result<()> {
    let (_root, live, replacement, state, mut transaction) = transaction_fixture()?;
    transaction.mark_prepared()?;
    let mut lifecycle = FakeLifecycle { events: Vec::new(), start_failures: 1, stopped: Rc::new(Cell::new(false)) };

    match transaction.cutover(&mut lifecycle, || Ok(())) {
        Ok(()) => bail!("first start should fail"),
        Err(error) => assert!(error.to_string().contains("restored")),
    }
    assert_eq!(lifecycle.events, ["stop", "start", "start"]);
    assert_eq!(fs::read_to_string(live.join("value"))?, "old");
    assert!(!replacement.exists());
    assert!(!state.exists());
    Ok(())
}

#[test]
fn recovery_detects_post_exchange_by_inode_restores_live_and_cleans_state() -> Result<()> {
    let (_root, live, replacement, state, mut transaction) = transaction_fixture()?;
    transaction.mark_prepared()?;
    exchange_directories(&live, &replacement)?;

    let transaction = ImportTransaction::load(state.clone(), "site", &live)?;
    let mut lifecycle = FakeLifecycle { events: Vec::new(), start_failures: 0, stopped: Rc::new(Cell::new(false)) };
    transaction.recover(&mut lifecycle)?;

    assert_eq!(lifecycle.events, ["start"]);
    assert_eq!(fs::read_to_string(live.join("value"))?, "old");
    assert!(!state.exists());
    assert!(!replacement.parent().is_some_and(|path| path.exists()));
    Ok(())
}

#[test]
fn exchanged_recovery_verifies_imported_live_and_cleans_state() -> Result<()> {
    let (_root, live, replacement, state, mut transaction) = transaction_fixture()?;
    transaction.mark_prepared()?;
    exchange_directories(&live, &replacement)?;
    mark_state_exchanged(&state)?;

    let transaction = ImportTransaction::load(state.clone(), "site", &live)?;
    let mut lifecycle = FakeLifecycle { events: Vec::new(), start_failures: 0, stopped: Rc::new(Cell::new(false)) };
    transaction.recover(&mut lifecycle)?;

    assert_eq!(lifecycle.events, ["start"]);
    assert_eq!(fs::read_to_string(live.join("value"))?, "new");
    assert!(!state.exists());
    assert!(!replacement.parent().is_some_and(|path| path.exists()));
    Ok(())
}

#[test]
fn exchanged_recovery_failure_restores_previous_tree_and_cleans_state() -> Result<()> {
    let (_root, live, replacement, state, mut transaction) = transaction_fixture()?;
    transaction.mark_prepared()?;
    exchange_directories(&live, &replacement)?;
    mark_state_exchanged(&state)?;

    let transaction = ImportTransaction::load(state.clone(), "site", &live)?;
    let mut lifecycle = FakeLifecycle { events: Vec::new(), start_failures: 1, stopped: Rc::new(Cell::new(false)) };
    match transaction.recover(&mut lifecycle) {
        Ok(()) => bail!("service verification should fail"),
        Err(_) => {}
    }

    assert_eq!(lifecycle.events, ["start", "start"]);
    assert_eq!(fs::read_to_string(live.join("value"))?, "old");
    assert!(!state.exists());
    assert!(!replacement.parent().is_some_and(|path| path.exists()));
    Ok(())
}

#[test]
fn failed_import_and_failed_restore_retain_transaction_for_manual_recovery() -> Result<()> {
    let (_root, live, replacement, state, mut transaction) = transaction_fixture()?;
    transaction.mark_prepared()?;
    let mut lifecycle = FakeLifecycle { events: Vec::new(), start_failures: 2, stopped: Rc::new(Cell::new(false)) };

    match transaction.cutover(&mut lifecycle, || Ok(())) {
        Ok(()) => bail!("both starts should fail"),
        Err(error) => assert!(error.to_string().contains("transaction retained")),
    }
    assert_eq!(lifecycle.events, ["stop", "start", "start"]);
    assert_eq!(fs::read_to_string(live.join("value"))?, "old");
    assert!(state.exists());
    assert!(replacement.exists());
    assert_eq!(fs::read_to_string(replacement.join("value"))?, "new");
    Ok(())
}
