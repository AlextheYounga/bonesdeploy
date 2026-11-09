def artifacts(ctx):
    paths = ctx.paths
    placeholder = paths.placeholder_release
    return [
        ("application AppArmor profile", paths.apparmor_profile("gunicorn"), "file", "framework"),
        ("application systemd service", paths.systemd_service("gunicorn"), "file", "framework"),
        ("application systemd requirement", paths.systemd_service_requirement("gunicorn"), "link", "framework"),
        ("application runtime directory", paths.runtime_service_dir("gunicorn"), "directory", "framework"),
        ("application runtime socket", paths.runtime_service_socket("gunicorn"), "socket", "framework"),
        ("application log directory", paths.site_log_dir, "directory", "framework"),
        ("Django placeholder virtual environment", f"{placeholder}/.venv", "directory", "framework"),
        ("Django placeholder configuration", f"{placeholder}/config", "directory", "framework"),
        ("Django placeholder WSGI application", f"{placeholder}/config/wsgi.py", "file", "framework"),
    ]


def services(_ctx):
    return [
        ("application service", "{project}-gunicorn.service", "framework"),
    ]


def mode(_ctx):
    return "server"
