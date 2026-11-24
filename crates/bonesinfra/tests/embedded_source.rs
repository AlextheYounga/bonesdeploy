use bonesdeploy_core::state;

use bonesinfra::embedded_template_paths;

#[test]
fn template_inventory_contains_shared_and_framework_assets() {
    let paths = embedded_template_paths().collect::<Vec<_>>();
    assert!(paths.iter().any(|path| path == state::INDEX_TEMPLATE));
    assert!(paths.iter().any(|path| path == "frameworks/angular/nginx/static-site-nginx.conf.j2"));
    assert!(paths.iter().any(|path| path == "frameworks/laravel/queue-worker.service.j2"));
    assert!(!paths.iter().any(|path| path.starts_with("tests/") || path.contains("__pycache__")));
}
