def artifacts(ctx):
    paths = ctx.paths
    version = str(ctx.runtime.data.get("php_version", "8.5"))
    project = ctx.app.project_name
    return [
        ("PHP-FPM pool configuration", f"/etc/php/{version}/fpm/pool.d/{project}.conf", "file", "framework"),
        ("PHP-FPM socket", f"/run/php/php{version}-fpm-{project}.sock", "file", "framework"),
        ("PHP log directory", paths.site_log_dir, "directory", "framework"),
        ("current PHP web root", paths.current_web_root, "directory", "framework"),
        ("nginx site configuration", paths.site_nginx_config, "file", "runtime"),
        ("nginx site", paths.nginx_site_available, "file", "runtime"),
        ("enabled nginx site", paths.nginx_site_enabled, "link", "runtime"),
    ]


def services(_ctx):
    return [("site nginx", "{project}-nginx.service", "runtime")]


def mode(_ctx):
    return "php"
