"""Resources owned by the shared per-site nginx runtime."""


def artifacts(ctx):
    if ctx.runtime.backend == "docker" and ctx.runtime.compose_port is None:
        return []
    paths = ctx.paths
    return [
        ("nginx site configuration", paths.site_nginx_config, "file", "runtime"),
        ("nginx site", paths.nginx_site_available, "file", "runtime"),
        ("enabled nginx site", paths.nginx_site_enabled, "link", "runtime"),
        ("site nginx systemd service", paths.systemd_site_nginx_service, "file", "runtime"),
        ("site nginx systemd requirement", paths.systemd_site_nginx_requirement, "link", "runtime"),
        ("nginx AppArmor profile", paths.nginx_apparmor_profile, "file", "runtime"),
        ("runtime nginx directory", paths.runtime_nginx_dir, "directory", "runtime"),
        ("runtime nginx socket", paths.runtime_nginx_socket, "socket", "runtime"),
        ("runtime nginx PID", paths.runtime_nginx_pid, "file", "runtime"),
    ]


def services(ctx):
    if ctx.runtime.backend == "docker" and ctx.runtime.compose_port is None:
        return []
    return [("site nginx", "{project}-nginx.service", "runtime")]
