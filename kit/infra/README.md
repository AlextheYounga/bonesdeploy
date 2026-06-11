# kit/infra

This directory contains the three pyinfra deploy scripts that drive `bonesdeploy remote setup|runtime|ssl`, plus Jinja2 template assets. Every file is embedded into the `bonesdeploy` binary and written to `<project>/.bones/infra/` during `bonesdeploy init` and `bonesdeploy remote runtime`.

## Deploy Scripts

### `setup.py` — Machine Bootstrap
Runs once per project as root. Provisions the bare Git repo, placeholder release, system users (deploy + service), firewall (UFW), and builds/installs `bonesremote` from source.

### `runtime.py` — Per-Site Runtime
Runs as the deploy user. Installs template-specific packages (via loading `../runtime/operations.py`), deploys AppArmor profile, nginx router config, per-site nginx config, and per-site systemd service.

### `ssl.py` — TLS Certificates
Runs as root. Obtains certbot certificates via webroot challenge and re-renders the nginx router with TLS enabled.

## Jinja2 Templates

### `assets/apparmor/project-nginx-profile.j2`
Per-project AppArmor profile template. Variables: `socket_path`, `conf_root`, `runtime_binaries`, `project_root`, `current_web_root`.

### `assets/nginx/router.conf.j2`
Top-level nginx server block for the project domain. Two modes: HTTP-only (for certbot challenges) and HTTPS (post-SSL). Variables: `server_name`, `site_nginx_config`, `socket_path`, `ssl_enabled`, `ssl_cert_path`, `ssl_cert_key_path`.

### `assets/nginx/site-nginx.conf.j2`
Per-site upstream nginx config that proxies to the project's Unix socket. Included by `router.conf.j2`. Variables: `socket_path`, `nginx_runtime_group`.

### `assets/nginx/site-nginx.service.j2`
Per-project systemd service unit for the site-local nginx. Variables: `project_name`, `conf_root` (`/srv/conf/<project>/nginx.conf`), `apparmor_profile_path`.

## Data Format

All deploy scripts receive a flat JSON dictionary passed via pyinfra `--data`. The data is built in Rust from `bones.yaml` and computed `DeploymentPaths`. Scripts access values through: `host.data.<key>`.

## Python Dependencies

Defined in `pyproject.toml`. Not embedded — the user's local `pyinfra` installation handles dependency resolution. The `.venv/` and `__pycache__/` directories are excluded from embedding.
