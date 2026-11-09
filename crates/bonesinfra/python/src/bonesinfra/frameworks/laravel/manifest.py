from bonesinfra.services.languages.php import php_fpm_pool_config_path, php_fpm_socket_path


def artifacts(ctx):
    paths = ctx.paths
    version = str(ctx.runtime.data.get("php_version", "8.5"))
    project = ctx.app.project_name
    entries = [
        (
            "PHP-FPM pool configuration",
            php_fpm_pool_config_path(version, project),
            "file",
            "framework",
        ),
        (
            "PHP-FPM socket",
            php_fpm_socket_path(version, project),
            "socket",
            "framework",
        ),
        ("PHP log directory", paths.site_log_dir, "directory", "framework"),
        ("current PHP web root", paths.current_web_root, "directory", "framework"),
    ]
    entries.append(("Laravel queue worker service", paths.systemd_service("worker"), "file", "framework"))
    return entries


def services(_ctx):
    return [("Laravel queue worker", "{project}-worker.service", "framework")]


def mode(_ctx):
    return "php"
