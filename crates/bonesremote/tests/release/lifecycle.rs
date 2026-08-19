use std::env;
use std::path::PathBuf;
use std::process;

use bonesdeploy_core::config;

use bonesremote::release::SiteMutation;
use bonesremote::release::lifecycle::DeploymentSnapshot;
use bonesremote::release::state::{DeploymentLock, override_sites_root};

#[test]
fn snapshot_uses_convention_paths_and_one_revision() -> anyhow::Result<()> {
    let root = env::temp_dir().join(format!("bonesremote_snapshot_{}", process::id()));
    let _root_guard = override_sites_root(root);
    let lock = DeploymentLock::acquire("demo")?;
    let mutation = SiteMutation::adopt("demo", config::Bones::for_site("demo"), lock);
    let snapshot = DeploymentSnapshot::new(&mutation, "deadbeef".to_string(), PathBuf::new());

    assert_eq!(snapshot.repo_path, PathBuf::from("/home/git/demo.git"));
    assert_eq!(snapshot.project_root, PathBuf::from("/srv/sites/demo"));
    assert_eq!(snapshot.revision, "deadbeef");
    assert_eq!(snapshot.site, "demo");
    Ok(())
}
