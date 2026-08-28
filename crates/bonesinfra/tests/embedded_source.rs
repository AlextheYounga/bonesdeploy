use anyhow::Result;
use bonesinfra::{embedded_template_paths, embedded_wheel};

#[test]
fn distribution_contains_a_portable_wheel() -> Result<()> {
    let wheel = embedded_wheel()?;
    assert!(wheel.starts_with(b"PK\x03\x04"));
    Ok(())
}

#[test]
fn template_inventory_contains_shared_and_framework_assets() {
    let paths = embedded_template_paths().collect::<Vec<_>>();
    assert!(paths.iter().any(|path| path == "assets/nginx/index.html.j2"));
    assert!(paths.iter().any(|path| path == "frameworks/laravel/templates/queue-worker.service.j2"));
    assert!(!paths.iter().any(|path| path.starts_with("tests/") || path.contains("__pycache__")));
}
