use std::{
    env, fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process,
};

use anyhow::Result;

use bonesremote::inspection::systemd::registered_services_in;

#[test]
fn reads_registered_site_services_from_target_requires_directory() -> Result<()> {
    let root = test_root("registered");
    let requires = root.join("shop.target.requires");
    fs::create_dir_all(&requires)?;
    for name in ["shop-nginx.service", "shop-worker.service"] {
        fs::write(root.join(name), "[Service]\n")?;
        symlink(root.join(name), requires.join(name))?;
    }

    assert_eq!(registered_services_in("shop.target", Path::new(&root))?, ["shop-nginx.service", "shop-worker.service"]);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rejects_non_site_service_registrations() -> Result<()> {
    let root = test_root("invalid-name");
    let requires = root.join("shop.target.requires");
    fs::create_dir_all(&requires)?;
    fs::write(root.join("shop-nginx.service"), "[Service]\n")?;
    symlink(root.join("shop-nginx.service"), requires.join("other-nginx.service"))?;

    assert!(registered_services_in("shop.target", Path::new(&root)).is_err());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rejects_non_symlink_service_registrations() -> Result<()> {
    let root = test_root("invalid-link");
    let requires = root.join("shop.target.requires");
    fs::create_dir_all(&requires)?;
    fs::write(requires.join("shop-nginx.service"), "not a link\n")?;

    assert!(registered_services_in("shop.target", Path::new(&root)).is_err());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rejects_service_links_outside_systemd_root() -> Result<()> {
    let root = test_root("invalid-target");
    let requires = root.join("shop.target.requires");
    let outside = root.join("outside.service");
    fs::create_dir_all(&requires)?;
    fs::write(&outside, "[Service]\n")?;
    symlink(&outside, requires.join("shop-nginx.service"))?;

    assert!(registered_services_in("shop.target", Path::new(&root)).is_err());
    fs::remove_dir_all(root)?;
    Ok(())
}

fn test_root(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!("bonesremote-systemd-{name}-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    root
}
