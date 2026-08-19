use bonesdeploy_core::{config, paths};

use crate::inspection::systemd;
use crate::release::state as release_state;

pub(crate) fn check_target(cfg: &config::Bones, issues: &mut Vec<String>) {
    let target_name = paths::site_target_name(&cfg.project_name);
    match systemd::property(&target_name, "LoadState") {
        Ok(state) if service_exists(&state) => {
            check_target_membership(&target_name, cfg, issues);
        }
        Ok(_) => issues.push(format!("site target is missing: {target_name}")),
        Err(error) => issues.push(format!("could not inspect site target {target_name} ({error})")),
    }
}

fn check_target_membership(target: &str, cfg: &config::Bones, issues: &mut Vec<String>) {
    let services = match systemd::required_services(target) {
        Ok(services) => services,
        Err(error) => {
            issues.push(format!("could not inspect required services for site target {target} ({error})"));
            return;
        }
    };
    if services.is_empty() {
        issues.push(format!("site target has no registered services: {target}"));
        return;
    }

    for service in services {
        check_required_service_active(target, &service, cfg, issues);
    }
}

fn check_required_service_active(target: &str, service: &str, cfg: &config::Bones, issues: &mut Vec<String>) {
    match systemd::active_status(service) {
        Ok(true) => {}
        Ok(false) if is_deferred_laravel_worker(cfg, service) => {}
        Ok(false) => issues.push(inactive_service_issue(target, service)),
        Err(error) => issues.push(format!("could not inspect required service {service} for {target} ({error})")),
    }
}

fn is_deferred_laravel_worker(cfg: &config::Bones, service: &str) -> bool {
    is_configured_laravel_worker(
        &cfg.runtime.template,
        cfg.runtime.extra.get(config::LARAVEL_INSTALL_QUEUE_WORKER).and_then(|value| value.as_bool()) == Some(true),
        &cfg.project_name,
        service,
    ) && current_is_placeholder(&cfg.project_root)
        && condition_failed(service)
}

fn is_configured_laravel_worker(template: &str, worker_enabled: bool, project_name: &str, service: &str) -> bool {
    template == config::LARAVEL_TEMPLATE && worker_enabled && service == format!("{project_name}-worker.service")
}

fn current_is_placeholder(project_root: &str) -> bool {
    release_state::current_release_name(project_root).is_ok_and(|release| release == paths::PLACEHOLDER_RELEASE_NAME)
}

fn condition_failed(service: &str) -> bool {
    systemd::property(service, "ConditionResult").is_ok_and(|value| value == "no")
}

fn inactive_service_issue(target: &str, service: &str) -> String {
    format!("required service {service} for site target {target} is not active")
}

fn service_exists(load_state: &str) -> bool {
    load_state.trim() == "loaded"
}

#[cfg(test)]
mod tests {
    use std::{env, fs, io::Result, os::unix::fs::symlink, process};

    use bonesdeploy_core::paths;

    use super::{current_is_placeholder, is_configured_laravel_worker, service_exists};

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
}
