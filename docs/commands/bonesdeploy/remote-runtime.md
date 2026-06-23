# bonesdeploy remote runtime

## Overview

Applies the selected runtime configuration through the hidden `bonesinfra` checkout managed by `bonesdeploy`.

## Command Signature

```bash
bonesdeploy remote runtime
```

## Detailed Execution Steps

### 1. Load Configuration and Choose Template

Loads `.bones/bones.toml` and `.bones/runtime.toml`. Prompts the user to select a framework from the available templates (`django`, `laravel`, `next`, `nuxt`, `rails`, `sveltekit`, `vue`).

### 2. Scaffold Runtime Assets

Deletes the existing `.bones/runtime/` directory (if any) and writes fresh scaffold:

- Writes runtime assets from the hidden `bonesinfra` checkout managed by `bonesdeploy` (shared runtime deploy script + Jinja2 templates)
- Writes `templates/<name>/runtime/` files to `.bones/runtime/` (template-specific `operations.py`)
- Saves the selected template name to `.bones/runtime.toml`

### 3. Apply Template Defaults

Parses the template's `bones.toml` to extract `web_root` and `permissions.paths` defaults. Updates the main `.bones/bones.toml` with these values only when the existing values still match the generic defaults — preserving any user overrides.

### 4. Confirm Remote Apply

Prompts `y/N` before running the remote deploy. On confirmation:

### 5. Run bonesinfra Runtime Apply

Runs the local bonesinfra CLI wrapper after confirmation:

```bash
python -m bonesinfra runtime apply --config .bones/bones.toml --runtime-config .bones/runtime.toml
```

The runtime apply uses the configured `ssh_user` from `bones.toml` and performs these operations in order:

1. **Framework packages** — installs template-defined apt packages and runs template-specific `operations.py` (e.g., Laravel installs PHP-FPM pool, Django installs Gunicorn)
2. **AppArmor** — ensures `apparmor.service` is running, deploys a per-project AppArmor profile from `assets/apparmor/project-nginx-profile.j2`, loads and enforces it
3. **Nginx** — creates runtime socket dir (`/run/<project>/`), deploys per-site nginx config from `site-nginx.conf.j2`, deploys per-site systemd service from `site-nginx.service.j2`, deploys router config from `router.conf.j2`, enables the site, validates and reloads
4. **Post-task** — runs `bonesremote doctor` as the deploy user to verify the setup

---

## What It Does NOT Do

- Does not handle SSL/TLS configuration (use `bonesdeploy remote ssl` for TLS)
- Does not run certbot or certificate challenges
- Does not pass any SSL-related variables to bonesinfra

---

## When to Run

1. After `bonesdeploy init` to choose a framework
2. When switching framework templates
3. After updating framework runtime assets in the repo
4. After editing the Jinja2 templates in the `src/assets/` directory of the `bonesinfra` repo

---

## Related Commands

- `bonesdeploy init` - Initialize project configuration
- `bonesdeploy remote setup` - Machine/bootstrap provisioning
- `bonesdeploy remote ssl` - Configure SSL certificates
- `bonesdeploy push` - Sync configuration to remote
