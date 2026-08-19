use std::{env, fs, io::Result, os::unix::fs::symlink, process};

use bonesdeploy_core::paths;

use bonesremote::commands::doctor::services::{current_is_placeholder, is_configured_laravel_worker, service_exists};

#[test]
fn only_the_first_laravel_release_can_defer_its_configured_worker() {
    assert!(is_configured_laravel_worker("laravel", true, "shop", "shop-worker.service"));
    assert!(!is_configured_laravel_worker("laravel", true, "shop", "shop-nginx.service"));
    assert!(!is_configured_laravel_worker("next", true, "shop", "shop-worker.service"));
    assert!(!is_configured_laravel_worker("laravel", false, "shop", "shop-worker.service"));
}

#[test]
fn recognizes_only_the_canonical_placeholder_release() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-doctor-placeholder-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    let placeholder = root.join(paths::RELEASES_DIR).join(paths::PLACEHOLDER_RELEASE_NAME);
    let deployed = root.join(paths::RELEASES_DIR).join("deployed");
    fs::create_dir_all(&placeholder)?;
    fs::create_dir_all(&deployed)?;
    symlink(&placeholder, root.join(paths::CURRENT_LINK))?;

    assert!(current_is_placeholder(root.to_str().unwrap_or_default()));

    fs::remove_file(root.join(paths::CURRENT_LINK))?;
    symlink(&deployed, root.join(paths::CURRENT_LINK))?;
    assert!(!current_is_placeholder(root.to_str().unwrap_or_default()));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn service_exists_accepts_loaded_unit() {
    assert!(service_exists("loaded\n"));
    assert!(!service_exists("not-found\n"));
}
