use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use bonesdeploy_core::paths;

use bonesremote::commands::deploy::coordinator::restore_previous_release;

#[test]
fn failed_activation_restores_previous_release() -> Result<()> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = env::temp_dir().join(format!("bonesremote_restore_{}_{}", process::id(), nonce));
    let releases = root.join(paths::RELEASES_DIR);
    let previous = releases.join("previous");
    let failed = releases.join("failed");
    fs::create_dir_all(&previous)?;
    fs::create_dir(&failed)?;
    symlink(&failed, root.join(paths::CURRENT_LINK))?;

    restore_previous_release(Path::new(&root), &previous)?;

    assert_eq!(fs::read_link(root.join(paths::CURRENT_LINK))?, previous);
    fs::remove_dir_all(root)?;
    Ok(())
}
