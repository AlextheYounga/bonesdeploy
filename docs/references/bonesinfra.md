Project Path: bonesinfra

Source Tree:

```txt
bonesinfra
├── AGENTS.md
├── README.md
├── build.sh
├── docs
├── pyproject.toml
├── ruff.toml
├── src
│   └── bonesinfra
│       ├── __main__.py
│       ├── app
│       │   ├── apply.py
│       │   ├── runtime_apply.py
│       │   ├── runtime_catalog.py
│       │   ├── setup_apply.py
│       │   └── ssl_apply.py
│       ├── assets
│       │   ├── apparmor
│       │   │   └── project-nginx-profile.j2
│       │   └── nginx
│       │       ├── index.html.j2
│       │       ├── router.conf.j2
│       │       ├── site-nginx.conf.j2
│       │       └── site-nginx.service.j2
│       ├── cli
│       │   └── app.py
│       ├── deploys
│       │   ├── runtime
│       │   │   ├── apparmor.py
│       │   │   ├── doctor.py
│       │   │   ├── nginx.py
│       │   │   ├── packages.py
│       │   │   ├── plan.py
│       │   │   └── template_runtime.py
│       │   ├── setup
│       │   │   ├── bonesremote.py
│       │   │   ├── directories.py
│       │   │   ├── firewall.py
│       │   │   ├── packages.py
│       │   │   ├── placeholder.py
│       │   │   ├── plan.py
│       │   │   └── users.py
│       │   └── ssl
│       │       └── plan.py
│       ├── domain
│       │   ├── context.py
│       │   └── paths.py
│       ├── infra
│       │   ├── deploy_helpers.py
│       │   ├── output.py
│       │   ├── pyinfra_runner.py
│       │   ├── toml_store.py
│       │   └── utils.py
│       └── runtimes
│           ├── __init__.py
│           ├── common
│           │   ├── __init__.py
│           │   ├── apparmor.py
│           │   ├── assets
│           │   │   ├── app-profile.j2
│           │   │   ├── app-site-nginx.conf.j2
│           │   │   ├── app.service.j2
│           │   │   └── static-site-nginx.conf.j2
│           │   ├── logs.py
│           │   ├── nginx.py
│           │   ├── node.py
│           │   ├── paths.py
│           │   ├── php_fpm_pool.py
│           │   ├── python.py
│           │   ├── ruby.py
│           │   ├── service.py
│           │   └── validation.py
│           ├── django
│           │   ├── deployment
│           │   │   ├── 01_install_build_deps.sh
│           │   │   └── 02_run_build.sh
│           │   ├── django.py
│           │   └── runtime.toml
│           ├── laravel
│           │   ├── __init__.py
│           │   ├── assets
│           │   │   ├── nginx
│           │   │   │   └── laravel-site-nginx.conf.j2
│           │   │   └── php
│           │   │       └── php-fpm-pool.conf.j2
│           │   ├── deploy.py
│           │   ├── deployment
│           │   │   ├── 01_install_build_deps.sh
│           │   │   └── 02_run_build.sh
│           │   ├── metadata.py
│           │   ├── nginx.py
│           │   ├── php_fpm.py
│           │   ├── php_packages.py
│           │   ├── php_repo.py
│           │   └── runtime.toml
│           ├── next
│           │   ├── next.py
│           │   └── runtime.toml
│           ├── nuxt
│           │   ├── deployment
│           │   │   ├── 01_install_build_deps.sh
│           │   │   └── 02_run_build.sh
│           │   ├── nuxt.py
│           │   └── runtime.toml
│           ├── rails
│           │   ├── deployment
│           │   │   ├── 01_install_build_deps.sh
│           │   │   └── 02_run_build.sh
│           │   ├── rails.py
│           │   └── runtime.toml
│           ├── sveltekit
│           │   ├── deployment
│           │   │   ├── 01_install_build_deps.sh
│           │   │   └── 02_run_build.sh
│           │   ├── runtime.toml
│           │   └── svelte.py
│           └── vue
│               ├── deployment
│               │   ├── 01_install_build_deps.sh
│               │   └── 02_run_build.sh
│               ├── runtime.toml
│               └── vue.py
└── tests
    ├── __init__.py
    ├── __main__.py
    ├── cleancode
    │   ├── test_no_provably_unnecessary_fallback.py
    │   └── test_no_suspicious_fallback.py
    ├── helpers.py
    ├── test_cli.py
    ├── test_context.py
    ├── test_deploy_structure.py
    ├── test_paths.py
    ├── test_pyinfra_runner.py
    ├── test_runtime_nginx.py
    ├── test_runtimes.py
    ├── test_syntax.py
    ├── test_templates_j2.py
    └── test_tripwires.py

```

`AGENTS.md`:

```md
You are a lazy senior developer. Lazy means efficient, not careless. The best code is the code never written.

Before writing any code, stop at the first rung that holds:

1. Does this need to be built at all? (YAGNI)
2. Does the standard library already do this? Use it.
3. Does a native platform feature cover it? Use it.
4. Does an already-installed dependency solve it? Use it.
5. Can this be one line? Make it one line.
6. Only then: write the minimum code that works.

When you are done, please run `ruff check .`. Do not ignore errors or warnings.

Rules:

- No abstractions that weren't explicitly requested.
- No new dependency if it can be avoided.
- No boilerplate nobody asked for.
- Deletion over addition. Boring over clever. Fewest files possible.
- Question complex requests: "Do you actually need X, or does Y cover it?"
- Pick the edge-case-correct option when two stdlib approaches are the same size, lazy means less code, not the flimsier algorithm.
- Mark intentional simplifications with a `ponytail:` comment. If the shortcut has a known ceiling (global lock, O(n²) scan, naive heuristic), the comment names the ceiling and the upgrade path.

Not lazy about: input validation at trust boundaries, error handling that prevents data loss, security, accessibility, the calibration real hardware needs (the platform is never the spec ideal, a clock drifts, a sensor reads off), anything explicitly requested. Lazy code without its check is unfinished: non-trivial logic leaves ONE runnable check behind, the smallest thing that fails if the logic breaks (an assert-based demo/self-check or one small test file; no frameworks, no fixtures). Trivial one-liners need no test.

(Yes, this file also applies to agents working on the ponytail repo itself. Especially to them.)
```

`README.md`:

```md
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

All deploy scripts receive data via pyinfra `--data key=value` CLI flags. Nested objects (like `DeploymentPaths`) are flattened to dotted keys (e.g. `--data paths.repo=/home/git/myapp.git`). Each script calls `_unflatten(host.data.dict())` to reconstruct nested dicts for template rendering. Direct access uses `DEPLOY_DATA["key"]` or `DEPLOY_DATA.get("key")`.

## Python Dependencies

Defined in `pyproject.toml`. Not embedded — the user's local `pyinfra` installation handles dependency resolution. The `.venv/` and `__pycache__/` directories are excluded from embedding.

```

`build.sh`:

```sh
#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON="${PYTHON:-python3}"

if command -v uv >/dev/null 2>&1; then
  exec uv run --with pyinstaller --project "$ROOT" python -m PyInstaller \
    --clean \
    --noconfirm \
    --onefile \
    --name bonesinfra \
    --paths "$ROOT/src" \
    --collect-data bonesinfra \
    --collect-submodules pyinfra \
    "$ROOT/src/bonesinfra/__main__.py"
fi

"$PYTHON" -m pip install --upgrade pyinstaller

exec "$PYTHON" -m PyInstaller \
  --clean \
  --noconfirm \
  --onefile \
  --name bonesinfra \
  --paths "$ROOT/src" \
  --collect-data bonesinfra \
  --collect-submodules pyinfra \
  "$ROOT/src/bonesinfra/__main__.py"

```

`pyproject.toml`:

```toml
[build-system]
requires = ["setuptools>=75"]
build-backend = "setuptools.build_meta"

[project]
name = "bonesinfra"
version = "0.1.0"
description = "Deployment automation for BonesDeploy"
readme = "README.md"
requires-python = ">=3.12"
dependencies = [
    "pyinfra>=3.8.0",
    "rich>=15.0.0",
    "typer>=0.26.7",
]

[project.scripts]
bonesinfra = "bonesinfra.__main__:main"

[tool.setuptools.package-dir]
"" = "src"

[tool.setuptools.package-data]
bonesinfra = [
    "assets/**/*.j2",
    "runtimes/**/assets/**/*.j2",
    "runtimes/**/deployment/*.sh",
    "runtimes/**/runtime.toml",
]

[dependency-groups]
dev = [
    "pytest>=9.0.3",
]

```

`ruff.toml`:

```toml
target-version = "py312"
line-length = 120

[lint]
select = [
  "E",    # pycodestyle
  "F",    # pyflakes
  "B",    # bugbear
  "I",    # import sorting
  "SIM",  # simplify
  "UP",   # pyupgrade
  "N",    # naming
  "ARG",  # unused args
  "C90",  # complexity
  "BLE",  # blind except
  "FBT",  # boolean trap
  "TRY",  # exception handling quality
  "PTH",  # pathlib preference
  "RUF",  # Ruff-specific rules
  "PL",   # pylint-derived rules
  "S",    # security-ish checks
  "D",    # docstring presence/style
  "DOC",  # docstring consistency
  "A",    # shadowing builtins
  "RET",  # return statement cleanliness
  "T20",  # print statement detection (like dbg_macro in Rust)
  "ERA",  # commented-out code detection
  "ICN",  # import conventions (numpy as np, etc.)
  "PERF", # performance anti-patterns
  "FURB", # modernize / refurbish
  "Q",    # quote consistency
  "PIE",  # misc. lints (unnecessary pass, duplicate class field keys)
  "C4",   # comprehension style
  "RSE",  # unnecessary exception parentheses
  "FA",   # future annotations
]
ignore = [
  "D",      # docstring rules (disabled by project convention)
  "TRY003", # long exception messages (sometimes necessary in CLI tools)
  "FBT001", # boolean positional arg (sometimes necessary for flags)
  "RET504", # unnecessary assignment before return (can aid readability)
  "ERA001", # commented-out code (enable once codebase is clean)
]

fixable = ["ALL"]
dummy-variable-rgx = "^_$"

[lint.mccabe]
max-complexity = 8

[lint.pylint]
max-args = 5
max-branches = 10
max-returns = 6
max-statements = 40

[lint.isort]
force-single-line = false
known-first-party = []
combine-as-imports = true

[lint.per-file-ignores]
"tests/**/*.py" = [
  "S101",   # allow assert in tests
  "D100",   # no module docstring requirement in tests
  "D101",
  "D102",
  "D103",
  "D104",
  "ARG001", # unused function args (fixtures, parametrize)
  "ARG002", # unused method args
  "PLR2004", # magic values in comparisons (fine in tests)
  "T20",    # allow print in tests
  "S102",    # exec() used in test helpers
  "S603",    # subprocess with test harness
]
"__init__.py" = ["F401", "D104"]
"conftest.py" = ["D100", "D103"]
"scripts/**/*.py" = ["T20"]
"src/bonesinfra/cli/*.py" = [
  "T20",     # CLI tool uses print for output/errors
  "PLC0415", # intentional lazy imports for CLI commands
]
"src/bonesinfra/app/*.py" = [
  "T20",     # app services print errors to stderr
]
"src/bonesinfra/infra/pyinfra_runner.py" = [
  "T20",     # print to stderr on failure
  "PLR0913", # run() takes 6 keyword args naturally
]
"src/bonesinfra/infra/deploy_helpers.py" = [
  "PLR0913", # render() wraps pyinfra template with all its args
]
"src/bonesinfra/deploys/runtime/template_runtime.py" = [
  "PLC0415", # lazy import to avoid circular dependency
  "BLE001",  # blind except on ImportError/KeyError
]
"src/bonesinfra/runtimes/__init__.py" = ["T20"]
"src/bonesinfra/runtimes/**/*.py" = [
  "PLC0415", # pyinfra imports are deferred to deploy time
]
"src/bonesinfra/deploys/setup/bonesremote.py" = [
  "S604",    # pyinfra.server.user with shell=True is standard
]
"src/bonesinfra/deploys/setup/users.py" = [
  "S604",    # pyinfra.server.user with shell=True is standard
]
"src/bonesinfra/deploys/ssl/plan.py" = [
  "A005",    # intentional module name matching project domain
  "T20",     # print to stderr on error
]

[format]
quote-style = "double"
indent-style = "space"
skip-magic-trailing-comma = false
line-ending = "auto"
docstring-code-format = true
docstring-code-line-length = 80

```

`src/bonesinfra/__main__.py`:

```py
from bonesinfra.cli.app import app


def main():
    app()


if __name__ == "__main__":
    main()

```

`src/bonesinfra/app/apply.py`:

```py
import sys

from bonesinfra.infra.pyinfra_runner import run as run_deploy


def run_plan(deploy, ctx):
    if not ctx.host:
        print("Error: missing host in bones.toml", file=sys.stderr)
        sys.exit(3)
    run_deploy(ctx=ctx, deploy=deploy)

```

`src/bonesinfra/app/runtime_apply.py`:

```py
from bonesinfra.app.apply import run_plan
from bonesinfra.deploys.runtime.plan import deploy_runtime
from bonesinfra.domain.context import DeployContext


def apply(config_path: str, runtime_config_path: str) -> None:
    ctx = DeployContext.from_files(config_path, runtime_config_path)
    run_plan(deploy_runtime, ctx)

```

`src/bonesinfra/app/runtime_catalog.py`:

```py
from bonesinfra.runtimes import get_runtime, list_runtimes


def list_all() -> list[str]:
    return list_runtimes()


def get_questions(runtime_name: str) -> list[dict]:
    module = get_runtime(runtime_name)
    return module.questions()

```

`src/bonesinfra/app/setup_apply.py`:

```py
from bonesinfra.app.apply import run_plan
from bonesinfra.deploys.setup.plan import deploy_setup
from bonesinfra.domain.context import DeployContext


def apply(config_path: str) -> None:
    ctx = DeployContext.from_files(config_path)
    run_plan(deploy_setup, ctx)

```

`src/bonesinfra/app/ssl_apply.py`:

```py
import sys

from bonesinfra.app.apply import run_plan
from bonesinfra.deploys.ssl.plan import deploy_ssl
from bonesinfra.domain.context import DeployContext


def apply(config_path: str) -> None:
    ctx = DeployContext.from_files(config_path)
    if not ctx.host:
        print("Error: missing host in bones.toml", file=sys.stderr)
        sys.exit(3)
    if not ctx.config.domain or not ctx.config.email:
        print("Error: ssl.domain and ssl.email are required in bones.toml", file=sys.stderr)
        sys.exit(3)
    run_plan(deploy_ssl, ctx)

```

`src/bonesinfra/assets/apparmor/project-nginx-profile.j2`:

```j2
#include <tunables/global>

profile {{ apparmor_profile_name | default("bonesdeploy-" ~ project_name ~ "-nginx") }} flags=(attach_disconnected,mediate_deleted) {
  # Base runtime abstractions and libc/loader paths.
  #include <abstractions/base>

  network unix stream,

  {{ paths.bonesremote_global_link }} mr,
  {{ paths.bonesremote_global_link }} ix,
  /usr/sbin/nginx mr,
  /usr/sbin/nginx ix,

  /usr/** r,
  /bin/** r,
  /sbin/** r,
  /lib/** r,
  /lib64/** r,
  /etc/nginx/** r,
  /etc/ssl/** r,
  /etc/hosts r,
  /etc/resolv.conf r,
  /etc/nsswitch.conf r,
  /etc/passwd r,
  /etc/group r,
  /proc/** r,
  /sys/devices/system/cpu/online r,

  {{ paths.current_web_root }}/** r,
  # current is a symlink, so allow the resolved release path too.
  {{ paths.releases }}/*/{{ web_root }}/** r,
  {{ paths.repo_bones_toml }} r,
  {{ paths.site_nginx_config }} r,

  {{ paths.runtime_nginx_dir }}/ rw,
  {{ paths.runtime_nginx_dir }}/** rwk,

  # ponytail: glob grant on app sockets — per-app profiles are leaf-scoped,
  # but the per-site nginx must reach every app socket under /run/<project>/.
  # Upgrade path: enumerate explicit socket paths per runtime when runtimes
  # become dynamic.
  {{ paths.runtime_socket_dir }}/ r,
  {{ paths.runtime_socket_dir }}/*/ r,
  {{ paths.runtime_socket_dir }}/*/*.sock rw,

  # repo_path defaults to /home/{{ deploy_user }}/<project>.git, so global /home denies
  # would block bonesremote reading config and nginx config from repo-local bones files.
  deny /root/** r,
  deny /etc/ssh/** r,
}

```

`src/bonesinfra/assets/nginx/index.html.j2`:

```j2
<!doctype html>
<html lang="en">

<head>
	<meta charset="utf-8">
	<meta name="viewport" content="width=device-width, initial-scale=1">
	<link rel="preconnect" href="https://fonts.googleapis.com">
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
	<link href="https://fonts.googleapis.com/css2?family=Pirata+One&display=swap" rel="stylesheet">
	<title>{{ project_name }}</title>

	<style>
		:root {
			color-scheme: light;
		}

		* {
			box-sizing: border-box;
		}

		body {
			margin: 0;
			min-height: 100vh;
			display: grid;
			place-items: center;
			background: #010302;
			font-family: "Trebuchet MS", "Segoe UI", sans-serif;
		}

		.logo {
			font-size: clamp(96px, 22vw, 200px);
			line-height: 1;
			user-select: none;
		}

		h1 {
			font-family: "Pirata One", cursive;
			font-size: 5rem;
			letter-spacing: 2px;
			color: #E8E7E2;
			text-shadow: 0 2px 5px rgba(0, 0, 0, 0.25);
			margin: 0;
			margin-top: 3rem;
		}
	</style>
</head>

<body>
	<h1>It's Working!</h1>
	<div class="logo">
		<img class="logo" style="width: 35vw;"
			src="data:image/jpg;base64,/9j//gAQTGF2YzYyLjExLjEwMAD/2wBDAAgSEhUSFRgYGBgYGB0bHR4eHh0dHR0eHh4gICAmJiYgICAeHiAgJCQmJikqKScnJicqKi0tLTY2MzM/P0FNTV3/xACYAAEAAQUBAQAAAAAAAAAAAAAABwgBBgIFBAMBAQEBAQEAAAAAAAAAAAAAAAABAgMEEAEAAgECAwIICggEBAUFAQEAAQIDBBEFITESQVFhcQaBIpETMrFScqFCssFzNWLRFIIjMzSSU/DCQ6LSFeFUJJNjgxazRPGjJeIRAQEBAAMBAQEBAQAAAAAAAAABETECEiFBUSIy/8AAEQgE5gTmAwESAAISAAMSAP/aAAwDAQACEQMRAD8AgEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA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AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAf//Z"
			alt="BonesDeploy">
		
	</div>
</body>

</html>
```

`src/bonesinfra/assets/nginx/router.conf.j2`:

```j2
server {
    listen 80;
    listen [::]:80;
    server_name {{ nginx_server_name }};

    location / {
        proxy_pass http://unix:{{ paths.runtime_nginx_socket }};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location ^~ /.well-known/acme-challenge/ {
        root {{ paths.current_web_root }};
        try_files $uri =404;
    }
}

{% if nginx_ssl_enabled %}
server {
    listen 443 ssl;
    listen [::]:443 ssl;
    server_name {{ nginx_server_name }};

    ssl_certificate {{ nginx_ssl_certificate_path }};
    ssl_certificate_key {{ nginx_ssl_certificate_key_path }};

    location / {
        proxy_pass http://unix:{{ paths.runtime_nginx_socket }};
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
{% endif %}

```

`src/bonesinfra/assets/nginx/site-nginx.conf.j2`:

```j2
worker_processes 1;
pid {{ paths.runtime_nginx_pid }};
error_log {{ paths.runtime_nginx_dir }}/error.log notice;

events {
    worker_connections 1024;
}

http {
    access_log {{ paths.runtime_nginx_dir }}/access.log;
    client_body_temp_path {{ paths.runtime_nginx_dir }}/client_body;
    proxy_temp_path {{ paths.runtime_nginx_dir }}/proxy;
    fastcgi_temp_path {{ paths.runtime_nginx_dir }}/fastcgi;
    uwsgi_temp_path {{ paths.runtime_nginx_dir }}/uwsgi;
    scgi_temp_path {{ paths.runtime_nginx_dir }}/scgi;

    server {
        # ponytail: socket inherits umask, typically 0666 — any local user
        # can connect directly, bypassing the system nginx router. Acceptable
        # on single-tenant deploy servers. Upgrade path: set the per-site
        # nginx group to www-data and use a 0660 socket mode directive.
        listen unix:{{ paths.runtime_nginx_socket }};
        root {{ paths.current_web_root }};
        index index.html;

        location / {
            try_files $uri $uri/ /index.html;
        }

        location ^~ /.well-known/acme-challenge/ {
            try_files $uri =404;
        }
    }
}

```

`src/bonesinfra/assets/nginx/site-nginx.service.j2`:

```j2
[Unit]
Description=Per-site nginx for {{ project_name }}
After=network.target apparmor.service
Requires=apparmor.service

[Service]
Type=simple
User={{ runtime_user }}
Group={{ runtime_group }}
SupplementaryGroups={{ release_group }}
WorkingDirectory={{ paths.current }}
# 0711: system nginx (www-data) must traverse this dir to reach the socket.
# 0750 causes 502 — www-data is not in the runtime group.
RuntimeDirectory={{ project_name }}/nginx
RuntimeDirectoryMode=0711
StandardOutput=journal
StandardError=journal

ExecStart=/usr/sbin/nginx -c {{ paths.site_nginx_config }} -g 'daemon off;'

AppArmorProfile={{ apparmor_profile_name | default("bonesdeploy-" ~ project_name ~ "-nginx") }}

NoNewPrivileges=yes
RestrictNamespaces=yes
LockPersonality=yes
RestrictRealtime=yes
SystemCallArchitectures=native
CapabilityBoundingSet=
AmbientCapabilities=
PrivateDevices=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictAddressFamilies=AF_UNIX

PrivateTmp=yes
ProtectHome=yes
ProtectSystem=strict

ReadWritePaths={{ paths.runtime_nginx_dir }}
ReadOnlyPaths={{ paths.current }} {{ paths.site_nginx_config }}

Restart=always
RestartSec=2

[Install]
WantedBy=multi-user.target

```

`src/bonesinfra/cli/app.py`:

```py
import json

import typer

from bonesinfra.app import runtime_apply, runtime_catalog, setup_apply, ssl_apply

app = typer.Typer()
runtime_app = typer.Typer()
setup_app = typer.Typer()
ssl_app = typer.Typer()
app.add_typer(runtime_app, name="runtime", help="Runtime operations")
app.add_typer(setup_app, name="setup", help="Setup operations")
app.add_typer(ssl_app, name="ssl", help="SSL operations")


@runtime_app.command("list")
def runtime_list():
    print(json.dumps(runtime_catalog.list_all()))


@runtime_app.command("questions")
def runtime_questions(
    runtime: str = typer.Argument(help="Runtime name"),
):
    print(json.dumps(runtime_catalog.get_questions(runtime)))


@runtime_app.command("apply")
def runtime_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
    runtime_config: str = typer.Option(..., "--runtime-config", help="Path to runtime.toml"),
):
    runtime_apply.apply(config, runtime_config)


@setup_app.command("apply")
def setup_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
):
    setup_apply.apply(config)


@ssl_app.command("apply")
def ssl_apply_cmd(
    config: str = typer.Option(..., "--config", help="Path to bones.toml"),
):
    ssl_apply.apply(config)

```

`src/bonesinfra/deploys/runtime/apparmor.py`:

```py
from pyinfra.operations import server, systemd

from bonesinfra.domain.context import template_data
from bonesinfra.infra.deploy_helpers import render


def setup(ctx, paths, here):
    systemd.service(
        name="Ensure apparmor service is enabled and started",
        service="apparmor",
        enabled=True,
        running=True,
        _sudo=True,
    )

    server.shell(
        name="Verify apparmor kernel enabled",
        commands=[f"cat {paths['apparmor_enabled_param']}"],
        _sudo=True,
    )

    apparmor_profile_name = f"bonesdeploy-{ctx.config.project_name}-nginx"
    apparmor_profile_path = f"/etc/apparmor.d/{apparmor_profile_name}"

    render(
        "Deploy per-project apparmor profile",
        here / "assets/apparmor/project-nginx-profile.j2",
        apparmor_profile_path,
        mode="0644",
        apparmor_profile_name=apparmor_profile_name,
        **template_data(ctx, paths=paths),
    )

    server.shell(
        name="Load updated apparmor profile",
        commands=[f"apparmor_parser -r {apparmor_profile_path}"],
        _sudo=True,
    )

    server.shell(
        name="Ensure project profile is in enforce mode",
        commands=[f"aa-enforce {apparmor_profile_path}"],
        _sudo=True,
    )

```

`src/bonesinfra/deploys/runtime/doctor.py`:

```py
from shlex import quote

from pyinfra.operations import server


def _user_env_command(user, command):
    q_user = quote(user)
    home = f"$(getent passwd {q_user} | cut -d: -f6)"
    return f"HOME={home} XDG_CONFIG_HOME={home}/.config {command}"


def run(ctx):
    server.shell(
        name="Run bonesremote doctor as deploy user",
        commands=[_user_env_command(ctx.config.deploy_user, "/usr/local/bin/bonesremote doctor")],
        _sudo=True,
        _sudo_user=ctx.config.deploy_user,
    )

```

`src/bonesinfra/deploys/runtime/nginx.py`:

```py
from pathlib import Path

from pyinfra.operations import files, server, systemd

from bonesinfra.domain.context import template_data
from bonesinfra.infra.deploy_helpers import mkdir, render


def setup(ctx, paths, here):
    # 0711: system nginx (www-data) needs traversal to reach the per-site
    # nginx socket at /run/<project>/nginx/nginx.sock. 0750 would block it.
    mkdir(
        name="Ensure socket directory exists",
        path=paths["runtime_socket_dir"],
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0711",
    )

    mkdir(
        name="Ensure nginx runtime directory exists",
        path=paths["runtime_nginx_dir"],
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0711",
    )

    mkdir(
        name="Ensure conf directory exists",
        path=paths["conf_root"],
        group=ctx.runtime.runtime_group,
        mode="0750",
    )

    render(
        "Deploy per-site nginx config",
        here / "assets/nginx/site-nginx.conf.j2",
        paths["site_nginx_config"],
        group=ctx.runtime.runtime_group,
        mode="0640",
        **template_data(ctx, paths=paths),
    )

    render(
        "Deploy per-site nginx systemd service",
        here / "assets/nginx/site-nginx.service.j2",
        paths["systemd_site_nginx_service"],
        mode="0644",
        **template_data(ctx, paths=paths),
    )

    systemd.daemon_reload(
        name="Reload systemd after site-nginx service change",
        _sudo=True,
    )

    nginx_server_name = ctx.config.domain or ctx.config.preview_domain
    if not nginx_server_name:
        raise ValueError("domain or preview_domain is required for nginx config")
    nginx_ssl_enabled = bool(
        ctx.runtime.runtime_data.get("ssl_cert_path")
        and ctx.runtime.runtime_data.get("ssl_key_path")
    )

    render(
        "Deploy router nginx config",
        here / "assets/nginx/router.conf.j2",
        paths["nginx_site_available"],
        mode="0644",
        nginx_server_name=nginx_server_name,
        nginx_ssl_enabled=nginx_ssl_enabled,
        nginx_ssl_certificate_path=ctx.runtime.runtime_data.get("ssl_cert_path", ""),
        nginx_ssl_certificate_key_path=ctx.runtime.runtime_data.get("ssl_key_path", ""),
        **template_data(ctx, paths=paths),
    )

    files.link(
        name="Enable router nginx site",
        path=paths["nginx_site_enabled"],
        target=paths["nginx_site_available"],
        force=True,
        _sudo=True,
    )

    files.link(
        name="Disable default nginx site",
        path=paths["nginx_default_site_enabled"],
        present=False,
        _sudo=True,
    )

    server.shell(
        name="Validate nginx configuration",
        commands=["nginx -t"],
        _sudo=True,
    )


def start_services(paths):
    systemd.service(
        name="Ensure nginx service is enabled and started",
        service="nginx",
        enabled=True,
        running=True,
        _sudo=True,
    )

    site_name = Path(paths["systemd_site_nginx_service"]).stem
    systemd.service(
        name="Ensure per-site nginx service is enabled and started",
        service=site_name,
        enabled=True,
        running=True,
        daemon_reload=True,
        _sudo=True,
    )

```

`src/bonesinfra/deploys/runtime/packages.py`:

```py
from pyinfra.operations import apt


def install_apt(ctx):
    pkgs = ctx.runtime.runtime_data.get("runtime_apt_packages", [])
    if not pkgs:
        return
    apt.packages(
        name="Install runtime apt packages",
        packages=pkgs,
        present=True,
        update=True,
        cache_time=3600,
        _sudo=True,
    )

```

`src/bonesinfra/deploys/runtime/plan.py`:

```py
from pathlib import Path

from bonesinfra.deploys.runtime import apparmor, doctor, nginx, packages, template_runtime
from bonesinfra.domain.paths import DeploymentPaths


def deploy_runtime(ctx):
    paths = DeploymentPaths.new(
        ctx.config.project_name,
        ctx.config.repo_path,
        ctx.config.project_root,
        ctx.runtime.web_root,
    ).__dict__
    here = Path(__file__).parent.parent.parent

    packages.install_apt(ctx)
    apparmor.setup(ctx, paths, here)
    nginx.setup(ctx, paths, here)
    template_runtime.load(ctx)
    nginx.start_services(paths)
    doctor.run(ctx)

```

`src/bonesinfra/deploys/runtime/template_runtime.py`:

```py
def load(ctx):
    template = ctx.runtime.runtime_data.get("template")
    if not template:
        return

    from bonesinfra.runtimes import get_runtime

    runtime = get_runtime(template)
    if not hasattr(runtime, "deploy"):
        raise RuntimeError(f"Runtime {template} does not expose deploy(ctx)")

    runtime.deploy(ctx)

```

`src/bonesinfra/deploys/setup/bonesremote.py`:

```py
from pyinfra.operations import server

from bonesinfra.domain.paths import BONESDEPLOY_REPO


def install():
    cargo_bin = "/root/.cargo/bin/cargo"
    server.shell(
        name="Install bonesremote binary",
        commands=[f"{cargo_bin} install --root /usr/local --git {BONESDEPLOY_REPO} bonesremote"],
        _sudo=True,
    )

    server.shell(
        name="Run bonesremote init",
        commands=["/usr/local/bin/bonesremote init"],
        _sudo=True,
    )


def install_authorized_key(ctx):
    if not ctx.runtime.runtime_data.get("deploy_authorized_key"):
        return
    server.user(
        name="Ensure deploy user authorized key is installed",
        user=ctx.config.deploy_user,
        public_keys=[ctx.runtime.runtime_data["deploy_authorized_key"]],
        _sudo=True,
    )

```

`src/bonesinfra/deploys/setup/directories.py`:

```py
from pathlib import Path
from shlex import quote

from pyinfra.operations import server

from bonesinfra.infra.deploy_helpers import mkdir


def _user_env_command(user, command):
    q_user = quote(user)
    home = f"$(getent passwd {q_user} | cut -d: -f6)"
    return f"HOME={home} XDG_CONFIG_HOME={home}/.config {command}"


def setup_repo_and_project(ctx, paths):
    mkdir(
        name="Ensure bare repo parent directory exists",
        path=paths["repo_parent"],
        user=ctx.config.deploy_user,
        group=ctx.config.deploy_user,
    )

    server.shell(
        name="Initialize bare git repo",
        commands=[_user_env_command(ctx.config.deploy_user, f"git init --bare {quote(paths['repo'])}")],
        _sudo=True,
        _sudo_user=ctx.config.deploy_user,
    )

    mkdir(
        name="Ensure bare repo bones directory exists",
        path=paths["repo_bones"],
        user=ctx.config.deploy_user,
        group=ctx.config.deploy_user,
    )

    mkdir(
        name="Ensure project root parent directory is traversable",
        path=paths["project_root_parent"],
        mode="0711",
    )

    mkdir(
        name="Ensure project root with setgid for release group",
        path=ctx.config.project_root,
        user=ctx.config.deploy_user,
        group=ctx.runtime.release_group,
        mode="2751",
    )

    mkdir(
        name="Ensure releases directory with setgid",
        path=paths["releases"],
        user=ctx.config.deploy_user,
        group=ctx.runtime.release_group,
        mode="2750",
    )

    mkdir(
        name="Ensure build directory (private to deploy user)",
        path=str(Path(ctx.config.project_root) / "build"),
        user=ctx.config.deploy_user,
        group=ctx.config.deploy_user,
        mode="0700",
    )

    mkdir(
        name="Ensure shared directory (owned by runtime user)",
        path=paths["shared"],
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0711",
    )

    mkdir(
        name="Ensure placeholder release directory exists",
        path=paths["placeholder_web_root"],
        user=ctx.config.deploy_user,
        group=ctx.runtime.release_group,
        mode="0750",
    )

```

`src/bonesinfra/deploys/setup/firewall.py`:

```py
from pyinfra.operations import server


def configure(ctx):
    if not ctx.runtime.runtime_data.get("firewall_enabled", True):
        return

    ssh_port = int(ctx.runtime.runtime_data.get("ssh_port", int(ctx.config.port)))
    allowed_ports = ctx.runtime.runtime_data.get("firewall_allowed_ports", ["http", "https"])
    port_aliases = ctx.runtime.runtime_data.get("firewall_port_aliases", {"http": 80, "https": 443})
    rate_limit = ctx.runtime.runtime_data.get("firewall_ssh_rate_limit", False)
    ssh_cidrs = ctx.runtime.runtime_data.get("firewall_ssh_allowed_cidrs", [])
    manage_ssh = ctx.runtime.runtime_data.get("firewall_manage_ssh", True)

    cmds = []

    if manage_ssh:
        rule = "limit" if rate_limit else "allow"
        if not ssh_cidrs:
            cmds.append(f"ufw {rule} {ssh_port}/tcp")
        else:
            cmds.extend(f"ufw {rule} from {cidr} to any port {ssh_port} proto tcp" for cidr in ssh_cidrs)

    for port in allowed_ports:
        if port == "ssh":
            continue
        port_num = port_aliases.get(port, port)
        cmds.append(f"ufw allow {port_num}/tcp")

    incoming = ctx.runtime.runtime_data.get("firewall_default_incoming_policy", "deny")
    outgoing = ctx.runtime.runtime_data.get("firewall_default_outgoing_policy", "allow")
    cmds.append(f"ufw --force default {incoming} incoming")
    cmds.append(f"ufw --force default {outgoing} outgoing")
    cmds.append("ufw --force enable")

    server.shell(
        name="Apply UFW configuration",
        commands=cmds,
        _sudo=True,
    )

    if ctx.runtime.runtime_data.get("firewall_show_status", True):
        server.shell(
            name="Display UFW status",
            commands=["ufw status verbose"],
            _sudo=True,
        )

```

`src/bonesinfra/deploys/setup/packages.py`:

```py
from pyinfra.operations import apt

BASE_SYSTEM_PACKAGES: list[str] = [
    "build-essential",
    "ca-certificates",
    "curl",
    "git",
    "rsync",
    "sudo",
    "nginx",
    "apparmor",
    "apparmor-utils",
    "certbot",
    "ufw",
]

SUPPLEMENTARY_PACKAGES: list[str] = [
    "acl",
    "age",
    "apt-listchanges",
    "apt-transport-https",
    "automysqlbackup",
    "autossh",
    "btop",
    "borgbackup",
    "fail2ban",
    "fastfetch",
    "gnupg",
    "htop",
    "iftop",
    "inotify-tools",
    "iotop",
    "jdupes",
    "jq",
    "lsb-release",
    "lsof",
    "moreutils",
    "mutt",
    "nano",
    "neovim",
    "ncdu",
    "powerstat",
    "powertop",
    "rename",
    "sqlite3",
    "smartmontools",
    "sysbench",
    "sysstat",
    "telnet",
    "tmux",
    "tree",
    "unattended-upgrades",
    "unzip",
    "zip",
    "zsh",
]


def install_system(packages):
    apt.packages(
        name="Install setup apt packages",
        packages=packages,
        present=True,
        update=True,
        cache_time=3600,
        _sudo=True,
    )

```

`src/bonesinfra/deploys/setup/placeholder.py`:

```py
from pyinfra.operations import files

from bonesinfra.domain.context import template_data
from bonesinfra.infra.deploy_helpers import render


def seed(ctx, paths, here):
    render(
        "Seed placeholder index page",
        here / "assets/nginx/index.html.j2",
        paths["placeholder_index"],
        user=ctx.config.deploy_user,
        group=ctx.runtime.release_group,
        mode="0640",
        **template_data(ctx, paths=paths),
    )

    files.link(
        name="Point current symlink at placeholder release",
        path=paths["current"],
        target=paths["placeholder_release"],
        force=True,
        _sudo=True,
    )

```

`src/bonesinfra/deploys/setup/plan.py`:

```py
from pathlib import Path

from bonesinfra.deploys.setup import bonesremote, directories, firewall, packages, placeholder, users
from bonesinfra.deploys.setup.packages import BASE_SYSTEM_PACKAGES, SUPPLEMENTARY_PACKAGES
from bonesinfra.domain.paths import DeploymentPaths


def deploy_setup(ctx):
    paths = DeploymentPaths.new(
        ctx.config.project_name,
        ctx.config.repo_path,
        ctx.config.project_root,
        ctx.runtime.web_root,
    ).__dict__
    here = Path(__file__).parent.parent.parent
    all_pkgs = BASE_SYSTEM_PACKAGES + SUPPLEMENTARY_PACKAGES

    packages.install_system(all_pkgs)
    users.install_rust()
    users.ensure_users_and_groups(ctx)
    directories.setup_repo_and_project(ctx, paths)
    placeholder.seed(ctx, paths, here)
    firewall.configure(ctx)
    bonesremote.install_authorized_key(ctx)
    bonesremote.install()

```

`src/bonesinfra/deploys/setup/users.py`:

```py
from shlex import quote

from pyinfra import host
from pyinfra.facts.server import Users
from pyinfra.operations import server


def install_rust():
    server.shell(
        name="Install rustup and cargo",
        commands=["curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal"],
        _sudo=True,
    )


def _ensure_group_membership(user, group):
    q_user = quote(user)
    q_group = quote(group)
    server.shell(
        name=f"Ensure {user} is a member of {group}",
        commands=[f"id -nG {q_user} | tr ' ' '\\n' | grep -Fxq {q_group} || gpasswd -a {q_user} {q_group}"],
        _sudo=True,
    )


def ensure_users_and_groups(ctx):
    server.user(
        name="Ensure deploy user exists",
        user=ctx.config.deploy_user,
        shell="/bin/bash",
        ensure_home=True,
        _sudo=True,
    )

    server.group(
        name="Ensure runtime group exists",
        group=ctx.runtime.runtime_group,
        _sudo=True,
    )

    server.group(
        name="Ensure release-read group exists",
        group=ctx.runtime.release_group,
        _sudo=True,
    )

    existing_user = host.get_fact(Users).get(ctx.runtime.runtime_user)

    if existing_user is None:
        server.user(
            name="Ensure runtime user exists with groups",
            user=ctx.runtime.runtime_user,
            system=True,
            home="/nonexistent",
            shell="/usr/sbin/nologin",
            create_home=False,
            groups=[ctx.runtime.runtime_group, ctx.runtime.release_group],
            _sudo=True,
        )
        return

    required_groups = []
    for group in (ctx.runtime.runtime_group, ctx.runtime.release_group):
        if group not in required_groups:
            required_groups.append(group)

    for group in required_groups:
        if group != existing_user["group"] and group not in existing_user["groups"]:
            _ensure_group_membership(ctx.runtime.runtime_user, group)

```

`src/bonesinfra/deploys/ssl/plan.py`:

```py
import sys
from pathlib import Path

from pyinfra.operations import server, systemd

from bonesinfra.domain.context import template_data
from bonesinfra.domain.paths import DeploymentPaths
from bonesinfra.infra.deploy_helpers import render


def deploy_ssl(ctx):
    paths = DeploymentPaths.new(
        ctx.config.project_name,
        ctx.config.repo_path,
        ctx.config.project_root,
        ctx.runtime.web_root,
    ).__dict__
    here = Path(__file__).parent.parent.parent

    if not ctx.config.domain or not ctx.config.email:
        print("Error: ssl_domain and ssl_email are required", file=sys.stderr)
        sys.exit(1)

    _render_router_config(ctx, paths, here, ssl_enabled=False, stage="certbot challenge")
    obtain_certificate(ctx, paths)
    _render_router_config(ctx, paths, here, ssl_enabled=True, stage="SSL enable")


def _render_router_config(ctx, paths, here, ssl_enabled, stage):
    nginx_server_name = ctx.config.domain
    if not nginx_server_name:
        raise ValueError("domain is required for ssl nginx config")

    render(
        f"Render nginx config ({stage})",
        here / "assets/nginx/router.conf.j2",
        paths["nginx_site_available"],
        mode="0644",
        nginx_server_name=nginx_server_name,
        nginx_ssl_enabled=ssl_enabled,
        **template_data(ctx, paths=paths),
    )

    server.shell(
        name=f"Validate nginx configuration ({stage})",
        commands=["nginx -t"],
        _sudo=True,
    )

    systemd.service(
        name=f"Reload nginx ({stage})",
        service="nginx",
        reloaded=True,
        _sudo=True,
    )


def obtain_certificate(ctx, paths):
    server.shell(
        name="Obtain or renew certificate",
        commands=[
            "certbot certonly --non-interactive --agree-tos "
            f"--email {ctx.config.email} "
            "--webroot "
            f"-w {paths['current_web_root']} "
            f"-d {ctx.config.domain} "
            "--keep-until-expiring"
        ],
        _sudo=True,
    )

```

`src/bonesinfra/domain/context.py`:

```py
from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from bonesinfra.domain.paths import DeploymentPaths
from bonesinfra.infra.toml_store import load_toml

DEPLOY_USER = "git"

DEFAULT_SSH_USER = "root"
DEFAULT_SSH_PORT = "22"
DEFAULT_WEB_ROOT = "public"


@dataclass
class DeployContext:
    config: BonesConfig
    runtime: RuntimeConfig

    @classmethod
    def from_files(
        cls,
        config_path: str,
        runtime_config_path: str | None = None,
    ) -> DeployContext:
        bones_cfg = load_toml(config_path)
        project_name = bones_cfg.get("project_name", "")
        repo_path = bones_cfg.get("repo_path", "")
        project_root = bones_cfg.get("project_root", "")
        host = bones_cfg.get("host", "")
        port = int(bones_cfg.get("port", DEFAULT_SSH_PORT))

        runtime_cfg = {}
        if runtime_config_path:
            rpath = Path(runtime_config_path)
            if rpath.exists():
                runtime_cfg = load_toml(str(rpath))

        config = BonesConfig(
            remote_name=bones_cfg.get("remote_name", ""),
            project_name=project_name,
            ssh_user=bones_cfg.get("ssh_user", DEFAULT_SSH_USER),
            host=host,
            port=str(port),
            repo_path=repo_path,
            project_root=project_root,
            branch=bones_cfg.get("branch", ""),
            preview_domain=bones_cfg.get("preview_domain", ""),
            releases_keep=int(bones_cfg.get("releases_keep", 5)),
            ssl_enabled=bones_cfg.get("ssl_enabled", False),
            domain=bones_cfg.get("domain", ""),
            email=bones_cfg.get("email", ""),
            deploy_user=DEPLOY_USER,
        )

        runtime = RuntimeConfig(
            web_root=runtime_cfg.get("web_root", DEFAULT_WEB_ROOT),
            runtime_user=runtime_cfg.get("runtime_user", project_name),
            runtime_group=runtime_cfg.get("runtime_group", project_name),
            release_group=runtime_cfg.get("release_group", f"{project_name}-release"),
            runtime_data=runtime_cfg,
        )

        return cls(config=config, runtime=runtime)

    @property
    def host(self) -> str:
        return self.config.host

    @property
    def ssh_port(self) -> int:
        return int(self.config.port)


def template_data(ctx: DeployContext, *, paths: dict[str, Any] | None = None, **extra: Any) -> dict[str, Any]:
    """Build flat template context from DeployContext for Jinja2 template rendering."""
    if paths is None:
        p = DeploymentPaths.new(
            ctx.config.project_name, ctx.config.repo_path, ctx.config.project_root, ctx.runtime.web_root
        )
        paths = p.__dict__

    data: dict[str, Any] = {}
    data["project_name"] = ctx.config.project_name
    data["project_root"] = ctx.config.project_root
    data["web_root"] = ctx.runtime.web_root
    data["repo_path"] = ctx.config.repo_path
    data["deploy_user"] = ctx.config.deploy_user
    data["runtime_user"] = ctx.runtime.runtime_user
    data["runtime_group"] = ctx.runtime.runtime_group
    data["release_group"] = ctx.runtime.release_group
    data["project_root_parent"] = paths.get("project_root_parent", "")
    data["ssh_port"] = int(ctx.config.port)
    data["paths"] = paths
    data["ssl_domain"] = ctx.config.domain
    data["ssl_email"] = ctx.config.email
    data["preview_domain"] = ctx.config.preview_domain

    for key, value in ctx.runtime.runtime_data.items():
        if key not in data:
            data[key] = value

    data.update(extra)
    return data


@dataclass
class BonesConfig:
    remote_name: str
    project_name: str
    ssh_user: str
    host: str
    port: str
    repo_path: str
    project_root: str
    branch: str
    preview_domain: str
    releases_keep: int
    ssl_enabled: bool
    domain: str
    email: str
    deploy_user: str


@dataclass
class RuntimeConfig:
    web_root: str
    runtime_user: str
    runtime_group: str
    release_group: str
    runtime_data: dict[str, Any] = field(default_factory=dict)

```

`src/bonesinfra/domain/paths.py`:

```py
from dataclasses import dataclass
from pathlib import Path

DEFAULT_REPO_PARENT = "/home/git"
DEFAULT_PROJECT_ROOT_PARENT = "/srv/sites"
DEFAULT_CONF_ROOT_PARENT = "/srv/conf"
DEFAULT_WEB_ROOT = "public"

ETC_NGINX_SITES_AVAILABLE = "/etc/nginx/sites-available"
ETC_NGINX_SITES_ENABLED = "/etc/nginx/sites-enabled"
ETC_SYSTEMD_SYSTEM = "/etc/systemd/system"
ETC_APPARMOR_D = "/etc/apparmor.d"
ETC_SUDOERS_D = "/etc/sudoers.d"

RUNTIME_SOCKET_PARENT = "/run"
BONES_DIR = "bones"
BONES_TOML = "bones.toml"
NGINX_CONF = "nginx.conf"
INDEX_HTML = "index.html"
GIT_HEAD = "HEAD"
DEPLOYMENT_DIR = "deployment"
RELEASES_DIR = "releases"
SHARED_DIR = "shared"
BUILD_DIR = "build"
WORKSPACE_DIR = "workspace"
LOGS_DIR = "logs"
CURRENT_LINK = "current"
PLACEHOLDER_RELEASE_NAME = "19700101_000000"

NGINX_SOCKET = "nginx.sock"
NGINX_PID = "nginx.pid"
PHP_FPM_SOCKET = "php-fpm.sock"
DEFAULT_NGINX_SITE = "default"

BONESDEPLOY_BINARY = "bonesdeploy"
BONESREMOTE_BINARY = "bonesremote"

USR_LOCAL_BIN = "/usr/local/bin"
APPARMOR_ENABLED_PARAM = "/sys/module/apparmor/parameters/enabled"
APPARMOR_PROFILES = "/sys/kernel/security/apparmor/profiles"

BONESDEPLOY_REPO = "https://github.com/AlextheYounga/bonesdeploy.git"


def _parent_or_default(path: str, fallback: str) -> str:
    parent = Path(path).parent
    if parent and str(parent) != ".":
        return str(parent)
    return fallback


@dataclass
class DeploymentPaths:
    repo: str
    repo_parent: str
    repo_head: str
    repo_bones: str
    repo_bones_toml: str
    repo_deployment: str
    site_nginx_config: str
    conf_root: str
    project_root: str
    project_root_parent: str
    releases: str
    shared: str
    build_root: str
    build_logs: str
    current: str
    current_web_root: str
    placeholder_release: str
    placeholder_web_root: str
    placeholder_index: str
    nginx_site_available: str
    nginx_site_enabled: str
    nginx_default_site_enabled: str
    systemd_site_nginx_service: str
    apparmor_profile_path: str
    runtime_socket_dir: str
    runtime_nginx_dir: str
    runtime_nginx_socket: str
    runtime_nginx_pid: str
    runtime_php_fpm_socket: str
    sudoers_path: str
    usr_local_bin: str
    bonesremote_global_link: str
    apparmor_enabled_param: str
    apparmor_profiles: str

    @classmethod
    def new(
        cls,
        project_name: str,
        repo_path: str,
        project_root: str,
        web_root: str | None = None,
    ) -> "DeploymentPaths":
        if web_root is None:
            web_root = DEFAULT_WEB_ROOT

        placeholder_release = Path(project_root) / RELEASES_DIR / PLACEHOLDER_RELEASE_NAME
        current = Path(project_root) / CURRENT_LINK
        runtime_socket_dir = Path(RUNTIME_SOCKET_PARENT) / project_name
        runtime_nginx_dir = runtime_socket_dir / "nginx"
        repo_bones = Path(repo_path) / BONES_DIR
        conf_root = Path(DEFAULT_CONF_ROOT_PARENT) / project_name

        return cls(
            repo=repo_path,
            repo_parent=_parent_or_default(repo_path, DEFAULT_REPO_PARENT),
            repo_head=str(Path(repo_path) / GIT_HEAD),
            repo_bones=str(repo_bones),
            repo_bones_toml=str(repo_bones / BONES_TOML),
            site_nginx_config=str(conf_root / NGINX_CONF),
            repo_deployment=str(repo_bones / DEPLOYMENT_DIR),
            conf_root=str(conf_root),
            project_root=project_root,
            project_root_parent=_parent_or_default(project_root, DEFAULT_PROJECT_ROOT_PARENT),
            releases=str(Path(project_root) / RELEASES_DIR),
            shared=str(Path(project_root) / SHARED_DIR),
            build_root=str(Path(project_root) / BUILD_DIR / WORKSPACE_DIR),
            build_logs=str(Path(project_root) / BUILD_DIR / LOGS_DIR),
            current=str(current),
            current_web_root=str(current / web_root),
            placeholder_release=str(placeholder_release),
            placeholder_web_root=str(placeholder_release / web_root),
            placeholder_index=str(placeholder_release / web_root / INDEX_HTML),
            nginx_site_available=str(Path(ETC_NGINX_SITES_AVAILABLE) / f"{project_name}.conf"),
            nginx_site_enabled=str(Path(ETC_NGINX_SITES_ENABLED) / f"{project_name}.conf"),
            nginx_default_site_enabled=str(Path(ETC_NGINX_SITES_ENABLED) / DEFAULT_NGINX_SITE),
            systemd_site_nginx_service=str(Path(ETC_SYSTEMD_SYSTEM) / f"{project_name}-nginx.service"),
            apparmor_profile_path=str(Path(ETC_APPARMOR_D) / f"bonesdeploy-{project_name}-nginx"),
            runtime_socket_dir=str(runtime_socket_dir),
            runtime_nginx_dir=str(runtime_nginx_dir),
            runtime_nginx_socket=str(runtime_nginx_dir / NGINX_SOCKET),
            runtime_nginx_pid=str(runtime_nginx_dir / NGINX_PID),
            runtime_php_fpm_socket=str(runtime_socket_dir / PHP_FPM_SOCKET),
            sudoers_path=str(Path(ETC_SUDOERS_D) / "bonesdeploy"),
            usr_local_bin=USR_LOCAL_BIN,
            bonesremote_global_link=str(Path(USR_LOCAL_BIN) / BONESREMOTE_BINARY),
            apparmor_enabled_param=APPARMOR_ENABLED_PARAM,
            apparmor_profiles=APPARMOR_PROFILES,
        )

```

`src/bonesinfra/infra/deploy_helpers.py`:

```py
from pyinfra.operations import files


def mkdir(name, path, user="root", group="root", mode="0755"):
    files.directory(
        name=name,
        path=path,
        user=user,
        group=group,
        mode=mode,
        _sudo=True,
    )


def render(name, src, dest, user="root", group="root", mode="0644", **data):
    files.template(
        name=name,
        src=str(src),
        dest=dest,
        user=user,
        group=group,
        mode=mode,
        **data,
        _sudo=True,
    )

```

`src/bonesinfra/infra/output.py`:

```py
from __future__ import annotations

import logging
import os
from collections.abc import Iterator
from contextlib import contextmanager

from pyinfra.api.output import set_echo, set_formatter
from pyinfra.api.state import BaseStateCallback, State
from rich.console import Console
from rich.markup import escape
from rich.panel import Panel
from rich.status import Status
from rich.table import Table
from rich.text import Text

console = Console(stderr=True)
_err = Console(stderr=True)

_STATUS_STYLES = {
    "Success": "bold green",
    "No changes": "cyan",
    "Failure": "bold red",
}


class _PyinfraLogHandler(logging.Handler):
    def emit(self, record):
        console.print(self.format(record), markup=True, highlight=False)


class BonesDeployCallback(BaseStateCallback):
    _status: Status | None = None

    @classmethod
    def _stop_status(cls) -> None:
        if cls._status is not None:
            cls._status.stop()
            cls._status = None

    @staticmethod
    def operation_start(state: State, op_hash: str) -> None:
        BonesDeployCallback._stop_status()
        op_meta = state.get_op_meta(op_hash)
        op_name = ", ".join(op_meta.names) or "Operation"
        BonesDeployCallback._status = console.status(
            f"[bold cyan]☠[/]  Running operation: [bold]{escape(op_name)}[/]",
            spinner="dots",
        )
        BonesDeployCallback._status.start()

    @staticmethod
    def operation_end(state: State, op_hash: str) -> None:
        BonesDeployCallback._stop_status()
        op_meta = state.get_op_meta(op_hash)
        op_name = ", ".join(op_meta.names) or "Operation"
        status = "No changes"

        for host in state.inventory:
            try:
                op_data = state.get_op_data_for_host(host, op_hash)
            except KeyError:
                continue

            operation_meta = op_data.operation_meta
            if not operation_meta.is_complete() or operation_meta.did_error():
                status = "Failure"
                break

            if operation_meta.did_change():
                status = "Success"

        status_style = _STATUS_STYLES.get(status, "dim")
        console.print(f"☠  {op_name}", end=" ")
        console.print(f"[{status_style}]{status}[/{status_style}]")


def setup_output() -> None:
    os.environ["PYINFRA_PROGRESS"] = "off"

    def bones_echo(message=None, **kwargs):
        del kwargs
        if message is not None:
            _err.print(message, markup=True, highlight=False)

    def bones_format(text, *args, **kwargs):
        fg = args[0] if args else kwargs.get("fg")
        styles = []
        if kwargs.get("bold"):
            styles.append("bold")
        if fg:
            styles.append(str(fg))

        text = escape(str(text))
        if not styles:
            return text
        return f"[{' '.join(styles)}]{text}[/]"

    set_echo(bones_echo)
    set_formatter(bones_format)

    pyinfra_logger = logging.getLogger("pyinfra")
    pyinfra_logger.handlers.clear()
    handler = _PyinfraLogHandler()
    handler.setFormatter(logging.Formatter("%(message)s"))
    handler.setLevel(logging.WARNING)
    pyinfra_logger.addHandler(handler)
    pyinfra_logger.setLevel(logging.WARNING)
    pyinfra_logger.propagate = False


def print_banner() -> None:
    console.print()
    title = Text("☠  bonesdeploy", style="bold cyan")
    console.print(Panel(title, border_style="cyan"))


def print_target(hostname: str, user: str) -> None:
    info = Table.grid(padding=(0, 1))
    info.add_column(style="dim")
    info.add_column(style="bold yellow")
    info.add_row("target:", f"{user}@{hostname}")
    console.print(info)
    console.print()


def print_connected() -> None:
    console.print("☠  [bold cyan]connected[/]")
    console.print()


def stop_live_output() -> None:
    BonesDeployCallback._stop_status()


def print_done(success: bool) -> None:
    console.print()
    if success:
        console.print("☠  [bold green]deploy complete[/]")
    else:
        console.print("☠  [bold red]deploy failed[/]")
    console.print()


@contextmanager
def activity(message: str) -> Iterator[None]:
    with console.status(f"[bold cyan]☠ bonesdeploy[/] {message}", spinner="dots"):
        yield

```

`src/bonesinfra/infra/pyinfra_runner.py`:

```py
from __future__ import annotations

import sys
from collections.abc import Callable

from pyinfra.api import Config, Inventory, State
from pyinfra.api.connect import connect_all
from pyinfra.api.exceptions import PyinfraError
from pyinfra.api.operations import run_ops
from pyinfra.context import ctx_config, ctx_host, ctx_inventory, ctx_state

from bonesinfra.domain.context import DeployContext
from bonesinfra.infra.output import (
    BonesDeployCallback,
    activity,
    print_banner,
    print_connected,
    print_done,
    print_target,
    setup_output,
    stop_live_output,
)


def run(
    *,
    ctx: DeployContext,
    ssh_key: str | None = None,
    deploy: Callable[[DeployContext], None],
) -> None:
    setup_output()

    hostname = ctx.config.host
    ssh_user = ctx.config.ssh_user
    ssh_port = int(ctx.config.port)

    host_data: dict[str, object] = {
        "ssh_user": ssh_user,
        "ssh_port": ssh_port,
    }
    if ssh_key:
        host_data["ssh_key"] = ssh_key

    config = Config()

    inventory = Inventory(([(hostname, host_data)], {}))
    state = State(inventory, config)
    target_host = next(iter(inventory))

    print_banner()
    print_target(hostname, ssh_user)

    try:
        with activity("connecting"):
            connect_all(state)
    except PyinfraError:
        print_done(success=False)
        sys.exit(1)

    print_connected()

    with (
        ctx_state.use(state),
        ctx_config.use(config),
        ctx_inventory.use(inventory),
        ctx_host.use(target_host),
        activity("planning deploy operations"),
    ):
        deploy(ctx)

    state.add_callback_handler(BonesDeployCallback())

    try:
        run_ops(state)
    except PyinfraError:
        stop_live_output()
        print_done(success=False)
        sys.exit(1)

    if state.failed_hosts:
        stop_live_output()
        print_done(success=False)
        sys.exit(1)

    stop_live_output()
    print_done(success=True)

```

`src/bonesinfra/infra/toml_store.py`:

```py
import tomllib
from pathlib import Path
from typing import Any


def load_toml(path: str) -> dict[str, Any]:
    with Path(path).open("rb") as file:
        return tomllib.load(file)


def load_runtime_config(deploy_file: str) -> dict[str, Any]:
    return load_toml(str(Path(deploy_file).parent / "runtime.toml"))

```

`src/bonesinfra/infra/utils.py`:

```py
from collections.abc import Mapping
from typing import Any


def unflatten(data_dict: Mapping[str, Any]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in data_dict.items():
        parts = key.split(".")
        node = result
        for part in parts[:-1]:
            if part not in node:
                node[part] = {}
            node = node[part]
        node[parts[-1]] = value
    return result

```

`src/bonesinfra/runtimes/__init__.py`:

```py
import sys

from bonesinfra.runtimes import laravel
from bonesinfra.runtimes.django import django
from bonesinfra.runtimes.next import next as next_runtime
from bonesinfra.runtimes.nuxt import nuxt
from bonesinfra.runtimes.rails import rails
from bonesinfra.runtimes.sveltekit import svelte
from bonesinfra.runtimes.vue import vue

RUNTIMES = {
    "laravel": laravel,
    "django": django,
    "next": next_runtime,
    "nuxt": nuxt,
    "rails": rails,
    "sveltekit": svelte,
    "vue": vue,
}


def list_runtimes():
    return sorted(RUNTIMES.keys())


def get_runtime(name):
    module = RUNTIMES.get(name)
    if module is None:
        print(f"Unknown runtime: {name}. Available: {', '.join(list_runtimes())}", file=sys.stderr)
        sys.exit(1)
    return module

```

`src/bonesinfra/runtimes/common/__init__.py`:

```py
from bonesinfra.runtimes.common import apparmor, logs, nginx, node, paths, python, ruby, service, validation

__all__ = ["apparmor", "logs", "nginx", "node", "paths", "python", "ruby", "service", "validation"]

```

`src/bonesinfra/runtimes/common/apparmor.py`:

```py
from pathlib import Path

from pyinfra.operations import files, server

from bonesinfra.domain.context import template_data


def render_app_profile(  # noqa: PLR0913
    ctx,
    *,
    paths,
    runtime,
    apparmor_exec_paths,
    apparmor_writable_paths,
    apparmor_network="network unix stream,",
):
    here = Path(__file__).parent
    profile_name = f"bonesdeploy-{ctx.config.project_name}-{runtime}"
    profile_path = f"/etc/apparmor.d/{profile_name}"
    files.template(
        name=f"Deploy {runtime} AppArmor profile",
        src=str(here / "assets/app-profile.j2"),
        dest=profile_path,
        user="root",
        group="root",
        mode="0644",
        apparmor_profile_name=profile_name,
        apparmor_runtime=runtime,
        apparmor_exec_paths=apparmor_exec_paths,
        apparmor_writable_paths=apparmor_writable_paths,
        apparmor_network=apparmor_network,
        **template_data(ctx, paths=paths),
        _sudo=True,
    )
    server.shell(
        name=f"Load {runtime} AppArmor profile",
        commands=[f"apparmor_parser -r -T -W {profile_path}"],
        _sudo=True,
    )
    server.shell(
        name=f"Enforce {runtime} AppArmor profile",
        commands=[f"aa-enforce {profile_name}"],
        _sudo=True,
    )
    return profile_name

```

`src/bonesinfra/runtimes/common/assets/app-profile.j2`:

```j2
#include <tunables/global>

profile {{ apparmor_profile_name }} flags=(attach_disconnected,mediate_deleted) {
  #include <abstractions/base>

  {{ apparmor_network | default("network unix stream,") }}

{% for exec_path in apparmor_exec_paths %}
  {{ exec_path }} mrix,
{% endfor %}

  /usr/** r,
  /bin/** r,
  /sbin/** r,
  /lib/** r,
  /lib64/** r,
  /etc/** r,
  /proc/** r,
  /sys/devices/system/cpu/online r,

  {{ paths.current }}/** r,
{% for write_path in apparmor_writable_paths %}
  {{ write_path }}/ rw,
  {{ write_path }}/** rwk,
{% endfor %}

  {{ paths.runtime_socket_dir }}/{{ apparmor_runtime }}/ rw,
  {{ paths.runtime_socket_dir }}/{{ apparmor_runtime }}/** rwk,

  /var/log/bonesdeploy/{{ project_name }}/ rw,
  /var/log/bonesdeploy/{{ project_name }}/** rwk,

  deny /root/** r,
  deny /etc/ssh/** r,
}

```

`src/bonesinfra/runtimes/common/assets/app-site-nginx.conf.j2`:

```j2
worker_processes 1;
pid {{ paths.runtime_nginx_pid }};
error_log {{ paths.runtime_nginx_dir }}/error.log notice;

events {
    worker_connections 1024;
}

http {
    access_log {{ paths.runtime_nginx_dir }}/access.log;
    client_body_temp_path {{ paths.runtime_nginx_dir }}/client_body;
    proxy_temp_path {{ paths.runtime_nginx_dir }}/proxy;
    fastcgi_temp_path {{ paths.runtime_nginx_dir }}/fastcgi;
    uwsgi_temp_path {{ paths.runtime_nginx_dir }}/uwsgi;
    scgi_temp_path {{ paths.runtime_nginx_dir }}/scgi;

    server {
        listen unix:{{ paths.runtime_nginx_socket }};
        root {{ paths.current_web_root }};

        location / {
            proxy_pass {{ app_proxy_target }};
            proxy_set_header Host $host;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_set_header X-Forwarded-Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_http_version 1.1;
            proxy_set_header Connection "";
        }

        location ^~ /.well-known/acme-challenge/ {
            try_files $uri =404;
        }
    }
}

```

`src/bonesinfra/runtimes/common/assets/app.service.j2`:

```j2
[Unit]
Description={{ runtime_label }} for {{ project_name }}
After=network.target apparmor.service
Requires=apparmor.service

[Service]
Type=simple
User={{ runtime_user }}
Group={{ runtime_group }}
SupplementaryGroups={{ release_group }}
WorkingDirectory={{ paths.current }}
RuntimeDirectory={{ project_name }}/{{ runtime_name }}
RuntimeDirectoryMode=0750
EnvironmentFile=-{{ paths.conf_root }}/runtime.env
ExecStart={{ runtime_exec }}

AppArmorProfile={{ apparmor_profile_name }}

NoNewPrivileges=yes
PrivateTmp=yes
ProtectHome=yes
ProtectSystem=strict
RestrictNamespaces=yes
LockPersonality=yes
RestrictRealtime=yes
SystemCallArchitectures=native
CapabilityBoundingSet=
AmbientCapabilities=
PrivateDevices=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictAddressFamilies={{ runtime_address_families | default("AF_UNIX") }}

ReadOnlyPaths={{ paths.current }}
ReadWritePaths={{ paths.runtime_socket_dir }}/{{ runtime_name }} {{ runtime_write_paths }} /var/log/bonesdeploy/{{ project_name }}

StandardOutput=journal
StandardError=journal
Restart=always
RestartSec=2

[Install]
WantedBy=multi-user.target

```

`src/bonesinfra/runtimes/common/assets/static-site-nginx.conf.j2`:

```j2
worker_processes 1;
pid {{ paths.runtime_nginx_pid }};
error_log {{ paths.runtime_nginx_dir }}/error.log notice;

events {
    worker_connections 1024;
}

http {
    access_log {{ paths.runtime_nginx_dir }}/access.log;
    client_body_temp_path {{ paths.runtime_nginx_dir }}/client_body;
    proxy_temp_path {{ paths.runtime_nginx_dir }}/proxy;
    fastcgi_temp_path {{ paths.runtime_nginx_dir }}/fastcgi;
    uwsgi_temp_path {{ paths.runtime_nginx_dir }}/uwsgi;
    scgi_temp_path {{ paths.runtime_nginx_dir }}/scgi;

    server {
        listen unix:{{ paths.runtime_nginx_socket }};
        root {{ paths.current }}/dist;
        index index.html;

        location / {
            try_files $uri $uri/ /index.html;
        }

        location ^~ /.well-known/acme-challenge/ {
            try_files $uri =404;
        }
    }
}

```

`src/bonesinfra/runtimes/common/logs.py`:

```py
from pyinfra.operations import files

BONESDEPLOY_LOG_ROOT = "/var/log/bonesdeploy"


def ensure(ctx):
    """Provision /var/log/bonesdeploy and /var/log/bonesdeploy/<project>.

    The root is root-owned; the per-project dir is owned by the runtime user
    so the service (and validation) can write logs without root.
    """
    files.directory(
        name="Ensure BonesDeploy log root exists",
        path=BONESDEPLOY_LOG_ROOT,
        user="root",
        group="root",
        mode="0755",
        _sudo=True,
    )

    files.directory(
        name="Ensure per-project log directory exists",
        path=f"{BONESDEPLOY_LOG_ROOT}/{ctx.config.project_name}",
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0750",
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/common/nginx.py`:

```py
from pathlib import Path

from pyinfra.operations import files, server

from bonesinfra.domain.context import template_data


def _ensure_runtime_socket_dir(ctx, paths):
    # 0711: system nginx (www-data) must traverse to reach per-site sockets.
    files.directory(
        name="Ensure runtime socket directory exists before nginx validation",
        path=paths["runtime_socket_dir"],
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0711",
        _sudo=True,
    )
    files.directory(
        name="Ensure nginx runtime directory exists before nginx validation",
        path=paths["runtime_nginx_dir"],
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0711",
        _sudo=True,
    )


def render_proxy(ctx, *, paths, socket_path=None, port=None):
    here = Path(__file__).parent
    app_proxy_target = f"http://unix:{socket_path}:" if socket_path else f"http://127.0.0.1:{port}"
    files.template(
        name="Deploy per-site app nginx config",
        src=str(here / "assets/app-site-nginx.conf.j2"),
        dest=paths["site_nginx_config"],
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0640",
        app_proxy_target=app_proxy_target,
        **template_data(ctx, paths=paths),
        _sudo=True,
    )
    _ensure_runtime_socket_dir(ctx, paths)
    server.shell(
        name="Validate nginx configuration with app config",
        commands=[f"nginx -t -c {paths['site_nginx_config']}"],
        _sudo=True,
    )


def render_static(ctx, *, paths):
    here = Path(__file__).parent
    files.template(
        name="Deploy per-site static nginx config",
        src=str(here / "assets/static-site-nginx.conf.j2"),
        dest=paths["site_nginx_config"],
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0640",
        **template_data(ctx, paths=paths),
        _sudo=True,
    )
    _ensure_runtime_socket_dir(ctx, paths)
    server.shell(
        name="Validate nginx configuration with static config",
        commands=[f"nginx -t -c {paths['site_nginx_config']}"],
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/common/node.py`:

```py
from pyinfra.operations import apt

NODE_PACKAGES = ["nodejs", "npm"]


def install_packages():
    apt.packages(
        name="Install Node.js runtime packages",
        packages=NODE_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/common/paths.py`:

```py
from pyinfra.operations import files

from bonesinfra.domain.paths import RUNTIME_SOCKET_PARENT


def ensure_runtime_dirs(ctx):
    """Create /run/<project> as the runtime user before validation or service start.

    RuntimeDirectory= in the systemd unit also does this at start time, but
    validation runs before start, so we provision it here too.
    """
    # 0711: system nginx (www-data) must traverse /run/<project>/ to reach
    # the per-site nginx socket. 0750 would cause 502 on every public request.
    project = ctx.config.project_name
    files.directory(
        name="Ensure runtime socket directory exists",
        path=f"{RUNTIME_SOCKET_PARENT}/{project}",
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0711",
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/common/php_fpm_pool.py`:

```py
from pyinfra.operations import files, server, systemd

from bonesinfra.domain.context import template_data
from bonesinfra.runtimes.common import logs

PHP_FPM_SOCKET_PARENT = "/run/php"


def socket_path(project, php_version):
    return f"{PHP_FPM_SOCKET_PARENT}/php{php_version}-fpm-{project}.sock"


def pool_config_path(project, php_version):
    return f"/etc/php/{php_version}/fpm/pool.d/{project}.conf"


def ensure_log_dir(ctx):
    logs.ensure(ctx)


def render_pool(ctx, *, here, paths, php_version):
    project = ctx.config.project_name
    files.template(
        name="Deploy Laravel PHP-FPM pool config",
        src=str(here / "assets/php/php-fpm-pool.conf.j2"),
        dest=pool_config_path(project, php_version),
        user="root",
        group="root",
        mode="0644",
        laravel_php_fpm_pool_name=project,
        laravel_php_fpm_socket_path=socket_path(project, php_version),
        **template_data(ctx, paths=paths),
        _sudo=True,
    )


def validate_php_fpm(php_version):
    server.shell(
        name="Validate PHP-FPM configuration",
        commands=[f"php-fpm{php_version} --test"],
        _sudo=True,
    )


def reload_php_fpm(php_version):
    # ponytail: reload may fail on a fresh install where the unit is inactive,
    # so we restart (always valid) rather than `systemctl reload`. Upgrade path
    # is a dedicated "ensure active then reload" sequence once first-boot is
    # handled out of band.
    systemd.service(
        name="Enable and restart PHP-FPM service",
        service=f"php{php_version}-fpm",
        enabled=True,
        running=True,
        restarted=True,
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/common/python.py`:

```py
from pyinfra.operations import apt

PYTHON_PACKAGES = [
    "python3",
    "python3-dev",
    "python3-pip",
    "python3-venv",
    "libpq-dev",
]


def install_packages():
    apt.packages(
        name="Install Python runtime packages",
        packages=PYTHON_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/common/ruby.py`:

```py
from pyinfra.operations import apt

RUBY_PACKAGES = [
    "ruby-full",
    "ruby-bundler",
    "libffi-dev",
    "libpq-dev",
    "libyaml-dev",
    "shared-mime-info",
    "zlib1g-dev",
]


def install_packages():
    apt.packages(
        name="Install Ruby runtime packages",
        packages=RUBY_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/common/service.py`:

```py
from pathlib import Path

from pyinfra.operations import files, systemd

from bonesinfra.domain.context import template_data
from bonesinfra.domain.paths import DeploymentPaths
from bonesinfra.runtimes.common import validation


def runtime_paths(ctx):
    return DeploymentPaths.new(
        ctx.config.project_name,
        ctx.config.repo_path,
        ctx.config.project_root,
        ctx.runtime.web_root,
    ).__dict__


def render_app_service(  # noqa: PLR0913
    ctx,
    *,
    paths,
    name,
    runtime_label,
    runtime_exec,
    apparmor_profile_name,
    runtime_write_paths,
    runtime_address_families="AF_UNIX",
):
    here = Path(__file__).parent
    project = ctx.config.project_name
    files.template(
        name=f"Deploy {name} systemd service",
        src=str(here / "assets/app.service.j2"),
        dest=f"/etc/systemd/system/{project}-{name}.service",
        user="root",
        group="root",
        mode="0644",
        runtime_name=name,
        runtime_label=runtime_label,
        runtime_exec=runtime_exec,
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=" ".join(runtime_write_paths),
        runtime_address_families=runtime_address_families,
        **template_data(ctx, paths=paths),
        _sudo=True,
    )


def enable_and_start(ctx, name, *, apparmor_profile_name=None):
    service = f"{ctx.config.project_name}-{name}.service"
    systemd.service(
        name=f"Enable and start {name} service",
        service=service,
        enabled=True,
        running=True,
        daemon_reload=True,
        _sudo=True,
    )
    if apparmor_profile_name:
        validation.verify_profile_attached(service, apparmor_profile_name)

```

`src/bonesinfra/runtimes/common/validation.py`:

```py
from shlex import quote

from pyinfra.operations import server


def run_as_runtime_user(ctx, name, command):
    """Validate a runtime command as the runtime user, never as root.

    Root provisioning must not also be the thing that proves the service can
    start: a root-owned validation artifact (e.g. a log file) would later be
    unwritable by the service. Runs the command with HOME/XDG_CONFIG_HOME set
    so user-scoped config resolution matches the real service environment.
    """
    user = ctx.runtime.runtime_user
    q_user = quote(user)
    home = f"$(getent passwd {q_user} | cut -d: -f6)"
    wrapped = f"HOME={home} XDG_CONFIG_HOME={home}/.config {command}"
    server.shell(
        name=name,
        commands=[wrapped],
        _sudo=True,
        _sudo_user=user,
    )


def verify_profile_attached(service_name, profile_name, *, name=None):
    """Verify the service's main PID is confined to the expected AppArmor profile.

    A loaded-but-unattached profile is a silent isolation failure: the service
    runs unrestricted. Reads /proc/<MainPID>/attr/current and fails if the
    profile name is not present. Deliberately run as root (needs /proc access
    to other users' attr and systemctl show).
    """
    q_service = quote(service_name)
    q_profile = quote(profile_name)
    cmd = (
        f"pid=$(systemctl show -p MainPID --value {q_service}); "
        f'[ "$pid" != "0" ] && [ -n "$pid" ] && '
        f'grep -q "{q_profile}" /proc/$pid/attr/current'
    )
    server.shell(
        name=name or f"Verify {service_name} attached to {profile_name}",
        commands=[cmd],
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/django/deployment/01_install_build_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
NVM_INSTALL_URL="https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.5/install.sh"

if [ -s "$NVM_DIR/nvm.sh" ]; then
  exit 0
fi

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$NVM_INSTALL_URL" | PROFILE=/dev/null NVM_DIR="$NVM_DIR" bash
else
  echo "curl or wget is required to install nvm."
  exit 1
fi
```

`src/bonesinfra/runtimes/django/deployment/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

command -v python3 >/dev/null 2>&1 || { echo "python3 not found"; exit 1; }

# Activate virtualenv
VENV_DIR="${VENV_DIR:-venv}"
if [ -d "./$VENV_DIR" ]; then
  # shellcheck disable=SC1090
  source "./$VENV_DIR/bin/activate"
else
  echo "Virtual environment not found at ./$VENV_DIR"
  echo "Create one on the server: python3 -m venv $VENV_DIR"
  exit 1
fi

# Install dependencies
if [ -f "./requirements.txt" ]; then
  pip install -r requirements.txt --quiet
elif [ -f "./pyproject.toml" ]; then
  pip install . --quiet
fi

# Run migrations
python3 manage.py migrate --noinput

# Collect static files
python3 manage.py collectstatic --noinput

# Restart gunicorn via systemd
SERVICE_NAME="$PROJECT_NAME"
if ! command -v systemctl >/dev/null 2>&1; then
  echo "systemctl not found. Restart your app server manually."
elif systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
  systemctl restart "$SERVICE_NAME"
elif systemctl list-unit-files | grep -q "$SERVICE_NAME"; then
  systemctl start "$SERVICE_NAME"
else
  echo "No systemd service found for $SERVICE_NAME. Restart your app server manually."
fi

```

`src/bonesinfra/runtimes/django/django.py`:

```py
from bonesinfra.runtimes.common import apparmor, logs, nginx, paths as common_paths, python, service, validation


def questions():
    return [
        {
            "key": "wsgi_module",
            "type": "text",
            "label": "WSGI module",
            "default": "config.wsgi:application",
        },
        {
            "key": "python_version",
            "type": "choice",
            "label": "Python version",
            "choices": ["3.11", "3.12", "3.13"],
            "default": "3.12",
        },
        {
            "key": "install_postgres",
            "type": "bool",
            "label": "Install PostgreSQL client libraries?",
            "default": False,
        },
        {
            "key": "static_root",
            "type": "text",
            "label": "Static root",
            "default": "staticfiles",
        },
        {
            "key": "media_root",
            "type": "text",
            "label": "Media root",
            "default": "media",
        },
    ]


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    socket_path = f"{paths['runtime_socket_dir']}/gunicorn/gunicorn.sock"
    wsgi_module = ctx.runtime.runtime_data.get("wsgi_module", "config.wsgi:application")
    static_root = f"{paths['current']}/{ctx.runtime.runtime_data.get('static_root', 'staticfiles')}"
    media_root = f"{paths['current']}/{ctx.runtime.runtime_data.get('media_root', 'media')}"
    writable = [static_root, media_root]
    gunicorn_bin = f"{paths['current']}/.venv/bin/gunicorn"
    python.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="gunicorn",
        apparmor_exec_paths=[gunicorn_bin],
        apparmor_writable_paths=writable,
    )
    validation.run_as_runtime_user(
        ctx,
        "Validate Gunicorn configuration as runtime user",
        f"{gunicorn_bin} --check-config {wsgi_module}",
    )
    service.render_app_service(
        ctx,
        paths=paths,
        name="gunicorn",
        runtime_label="Gunicorn",
        runtime_exec=f"{gunicorn_bin} {wsgi_module} --bind unix:{socket_path}",
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=writable,
    )
    nginx.render_proxy(ctx, paths=paths, socket_path=socket_path)
    service.enable_and_start(ctx, "gunicorn", apparmor_profile_name=apparmor_profile_name)

```

`src/bonesinfra/runtimes/django/runtime.toml`:

```toml
template = "django"
web_root = "public"

[permissions]
paths = [
    { path = "*", type = "dir", mode = "750" },
    { path = "*", type = "file", mode = "640" },
    { path = "static", type = "dir", mode = "750", recursive = true },
    { path = "media", type = "dir", mode = "770", recursive = true },
]

[shared]
paths = [
    { path = ".env", type = "file" },
    { path = "storage", type = "dir" },
]

```

`src/bonesinfra/runtimes/laravel/__init__.py`:

```py
from bonesinfra.runtimes.laravel.deploy import deploy
from bonesinfra.runtimes.laravel.metadata import questions

```

`src/bonesinfra/runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2`:

```j2
worker_processes 1;
pid {{ paths.runtime_nginx_pid }};
error_log {{ paths.runtime_nginx_dir }}/error.log notice;

events {
    worker_connections 1024;
}

http {
    access_log {{ paths.runtime_nginx_dir }}/access.log;
    client_body_temp_path {{ paths.runtime_nginx_dir }}/client_body;
    proxy_temp_path {{ paths.runtime_nginx_dir }}/proxy;
    fastcgi_temp_path {{ paths.runtime_nginx_dir }}/fastcgi;
    uwsgi_temp_path {{ paths.runtime_nginx_dir }}/uwsgi;
    scgi_temp_path {{ paths.runtime_nginx_dir }}/scgi;

    server {
        listen unix:{{ paths.runtime_nginx_socket }};
        root {{ paths.current_web_root }};
        index index.php index.html;

        location / {
            try_files $uri $uri/ /index.php?$query_string;
        }

        location ~ ^/index\.php(/|$) {
            fastcgi_pass unix:{{ laravel_php_fpm_socket_path }};
            fastcgi_param SCRIPT_FILENAME $realpath_root$fastcgi_script_name;
            include /etc/nginx/fastcgi_params;
            fastcgi_param DOCUMENT_ROOT $realpath_root;
            internal;
        }

        location ~ \.php$ {
            return 404;
        }

        location ^~ /.well-known/acme-challenge/ {
            try_files $uri =404;
        }
    }
}

```

`src/bonesinfra/runtimes/laravel/assets/php/php-fpm-pool.conf.j2`:

```j2
[{{ laravel_php_fpm_pool_name }}]
user = {{ runtime_user }}
group = {{ runtime_group }}

listen = {{ laravel_php_fpm_socket_path }}
listen.owner = www-data
listen.group = www-data
listen.mode = 0660

pm = dynamic
pm.max_children = 10
pm.start_servers = 2
pm.min_spare_servers = 1
pm.max_spare_servers = 3

chdir = {{ paths.current }}
clear_env = no

catch_workers_output = yes
decorate_workers_output = no

php_admin_flag[log_errors] = on
php_admin_value[error_log] = /var/log/bonesdeploy/{{ project_name }}/php-worker-error.log

access.log = /var/log/bonesdeploy/{{ project_name }}/php-fpm-access.log
slowlog = /var/log/bonesdeploy/{{ project_name }}/php-fpm-slow.log

```

`src/bonesinfra/runtimes/laravel/deploy.py`:

```py
from pathlib import Path

from bonesinfra.domain.paths import DeploymentPaths
from bonesinfra.runtimes.laravel import nginx, php_fpm, php_packages, php_repo


def deploy(ctx):
    here = Path(__file__).parent
    php_version = ctx.runtime.runtime_data.get("php_version", "8.3")
    paths = DeploymentPaths.new(
        ctx.config.project_name,
        ctx.config.repo_path,
        ctx.config.project_root,
        ctx.runtime.web_root,
    ).__dict__

    php_repo.add_php_apt_source()
    php_packages.install_php(php_version)

    php_fpm.setup_storage_directories(paths, ctx)
    php_fpm.setup_pool(here, ctx, paths, php_version)
    nginx.setup(here, ctx, paths, php_version)

```

`src/bonesinfra/runtimes/laravel/deployment/01_install_build_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
NVM_INSTALL_URL="https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.5/install.sh"

if [ -s "$NVM_DIR/nvm.sh" ]; then
  exit 0
fi

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$NVM_INSTALL_URL" | PROFILE=/dev/null NVM_DIR="$NVM_DIR" bash
else
  echo "curl or wget is required to install nvm."
  exit 1
fi
```

`src/bonesinfra/runtimes/laravel/deployment/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

trap 'status=$?; echo "[bonesdeploy] Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2; exit "$status"' ERR

[ -f artisan ] || { echo "artisan not found"; exit 1; }
command -v php >/dev/null 2>&1 || { echo "php not found"; exit 1; }
command -v composer >/dev/null 2>&1 || { echo "composer not found"; exit 1; }

# Install PHP dependencies first — artisan requires vendor/autoload.php
echo "[bonesdeploy] Installing Composer dependencies..."
composer install --no-dev --prefer-dist --no-interaction --optimize-autoloader

# Maintenance mode once the app can boot
echo "[bonesdeploy] Entering Laravel maintenance mode..."
php artisan down --render="errors::503"
trap 'php artisan up || true' EXIT

# Frontend build
if [ -f "./.nvmrc" ]; then
  export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
  if [ -s "$NVM_DIR/nvm.sh" ]; then
    # shellcheck disable=SC1090
    source "$NVM_DIR/nvm.sh"
  elif [ -s "$HOME/.config/nvm/nvm.sh" ]; then
    # shellcheck disable=SC1090
    source "$HOME/.config/nvm/nvm.sh"
  fi
  nvm install
fi

command -v pnpm >/dev/null 2>&1 || {
  echo "pnpm not found. Install it globally or enable it via corepack before deploy."
  exit 1
}

echo "[bonesdeploy] Installing frontend dependencies..."
pnpm install --frozen-lockfile
echo "[bonesdeploy] Building frontend assets..."
pnpm run build

if php artisan list | grep -q 'wayfinder:generate'; then
  php artisan wayfinder:generate
fi

if [ ! -f .env ] || ! grep -Eq '^APP_KEY=base64:' .env; then
  php artisan key:generate --force
fi

echo "[bonesdeploy] Running migrations..."
php artisan migrate --force

# Clear old caches and rebuild them back-to-back
echo "[bonesdeploy] Rebuilding Laravel caches..."
php artisan optimize:clear
php artisan config:cache
php artisan route:cache
php artisan view:cache
php artisan event:cache || true
php artisan queue:restart || true

php artisan up
trap - EXIT

```

`src/bonesinfra/runtimes/laravel/metadata.py`:

```py
def questions():
    return [
        {
            "key": "php_version",
            "type": "choice",
            "label": "PHP version",
            "choices": ["8.2", "8.3", "8.4", "8.5"],
            "default": "8.5",
        },
        {
            "key": "install_queue_worker",
            "type": "bool",
            "label": "Install Laravel queue worker?",
            "default": False,
        },
    ]

```

`src/bonesinfra/runtimes/laravel/nginx.py`:

```py
from pyinfra.operations import files, server

from bonesinfra.domain.context import template_data
from bonesinfra.runtimes.common import php_fpm_pool


def setup(here, ctx, paths, php_version):
    runtime_group = ctx.runtime.runtime_group
    php_fpm_socket_path = php_fpm_pool.socket_path(ctx.config.project_name, php_version)

    files.template(
        name="Deploy Laravel-specific per-site nginx config",
        src=str(here / "assets/nginx/laravel-site-nginx.conf.j2"),
        dest=paths["site_nginx_config"],
        user="root",
        group=runtime_group,
        mode="0640",
        laravel_php_fpm_socket_path=php_fpm_socket_path,
        **template_data(ctx, paths=paths),
        _sudo=True,
    )

    server.shell(
        name="Validate nginx configuration with Laravel config",
        commands=[f"nginx -t -c {paths['site_nginx_config']}"],
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/laravel/php_fpm.py`:

```py
from pyinfra.operations import files

from bonesinfra.runtimes.common import php_fpm_pool


def setup_storage_directories(paths, ctx):
    runtime_user = ctx.runtime.runtime_user
    runtime_group = ctx.runtime.runtime_group
    subdirs = ["logs", "framework/cache", "framework/sessions", "framework/views"]
    for subdir in subdirs:
        files.directory(
            name=f"Ensure storage/{subdir} exists",
            path=f"{paths['current']}/storage/{subdir}",
            user=runtime_user,
            group=runtime_group,
            mode="0775",
            _sudo=True,
        )


def setup_pool(here, ctx, paths, php_version):
    php_fpm_pool.ensure_log_dir(ctx)
    php_fpm_pool.render_pool(ctx, here=here, paths=paths, php_version=php_version)
    php_fpm_pool.validate_php_fpm(php_version)
    php_fpm_pool.reload_php_fpm(php_version)

```

`src/bonesinfra/runtimes/laravel/php_packages.py`:

```py
from pyinfra.operations import apt


def install_php(php_version):
    packages = [
        f"php{php_version}",
        f"php{php_version}-cli",
        f"php{php_version}-fpm",
        f"php{php_version}-bcmath",
        f"php{php_version}-curl",
        f"php{php_version}-gd",
        f"php{php_version}-intl",
        f"php{php_version}-mbstring",
        f"php{php_version}-mysql",
        f"php{php_version}-sqlite3",
        f"php{php_version}-xml",
        f"php{php_version}-zip",
        "composer",
    ]

    apt.packages(
        name="Install Laravel PHP packages",
        packages=packages,
        present=True,
        update=True,
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/laravel/php_repo.py`:

```py
from tempfile import NamedTemporaryFile

PHP_SURY_KEYRING_URL = "https://packages.sury.org/debsuryorg-archive-keyring.deb"
PHP_SURY_KEYRING_DEST = "/usr/share/keyrings/deb.sury.org-php.gpg"
PHP_SURY_PREREQUISITES = [
    "apt-transport-https",
    "ca-certificates",
    "curl",
    "lsb-release",
]


def _resolve_codename():
    from pyinfra import host
    from pyinfra.facts.server import LinuxDistribution

    deb = host.get_fact(LinuxDistribution)
    release_meta = deb.get("release_meta", {}) if deb else {}
    return (
        release_meta.get("VERSION_CODENAME")
        or release_meta.get("CODENAME")
        or release_meta.get("DISTRIB_CODENAME")
        or "noble"
    )


def add_php_apt_source():
    from pyinfra.operations import apt, server

    apt.packages(
        name="Install PHP repo prerequisites",
        packages=PHP_SURY_PREREQUISITES,
        present=True,
        update=True,
        _sudo=True,
    )

    with NamedTemporaryFile(delete=False, suffix=".deb") as f:
        keyring_path = f.name

    server.shell(
        name="Download PHP repo keyring package",
        commands=[f"curl -sSLo {keyring_path} {PHP_SURY_KEYRING_URL}"],
        _sudo=True,
    )

    apt.deb(
        name="Install PHP repo keyring package",
        src=keyring_path,
        _sudo=True,
    )

    server.shell(
        name="Remove stale PHP apt source file",
        commands=["rm -f /etc/apt/sources.list.d/php.list"],
        _sudo=True,
    )

    codename = _resolve_codename()
    apt.repo(
        name="Add Laravel PHP apt repository",
        src=f"deb [signed-by={PHP_SURY_KEYRING_DEST}] https://packages.sury.org/php {codename} main",
        filename="php",
        _sudo=True,
    )

```

`src/bonesinfra/runtimes/laravel/runtime.toml`:

```toml
template = "laravel"
web_root = "public"
php_version = "8.5"

[permissions]
paths = [
    { path = "*", type = "dir", mode = "750" },
    { path = "*", type = "file", mode = "640" },
    { path = "storage", type = "dir", mode = "770", recursive = true },
    { path = "bootstrap/cache", type = "dir", mode = "770", recursive = true },
    { path = "database", type = "dir", mode = "770", recursive = false },
    { path = "database/database.sqlite", type = "file", mode = "660", recursive = false },
]

[shared]
paths = [
    { path = "storage", type = "dir" },
    { path = ".env", type = "file" },
]

```

`src/bonesinfra/runtimes/next/next.py`:

```py
from bonesinfra.runtimes.common import apparmor, logs, nginx, node, paths as common_paths, service, validation


def questions():
    return []


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    port = ctx.runtime.runtime_data.get("internal_port", 3100)
    node.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="next",
        apparmor_exec_paths=["/usr/bin/node"],
        apparmor_writable_paths=[],
        apparmor_network="network inet stream,",
    )
    validation.run_as_runtime_user(
        ctx,
        "Validate Next.js standalone server exists as runtime user",
        "test -f .next/standalone/server.js",
    )
    service.render_app_service(
        ctx,
        paths=paths,
        name="next",
        runtime_label="Next.js app server",
        runtime_exec=(
            f"/usr/bin/env NODE_ENV=production PORT={port} HOSTNAME=127.0.0.1 "
            "node .next/standalone/server.js"
        ),
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=[],
        runtime_address_families="AF_UNIX AF_INET",
    )
    nginx.render_proxy(ctx, paths=paths, port=port)
    service.enable_and_start(ctx, "next", apparmor_profile_name=apparmor_profile_name)

```

`src/bonesinfra/runtimes/next/runtime.toml`:

```toml
template = "next"
web_root = "public"

[permissions]
paths = [
    { path = "*", type = "dir", mode = "750" },
    { path = "*", type = "file", mode = "640" },
    { path = ".next", type = "dir", mode = "770", recursive = true },
]

[shared]
paths = [
    { path = ".env", type = "file" },
    { path = "storage", type = "dir" },
]

```

`src/bonesinfra/runtimes/nuxt/deployment/01_install_build_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
NVM_INSTALL_URL="https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.5/install.sh"

if [ -s "$NVM_DIR/nvm.sh" ]; then
  exit 0
fi

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$NVM_INSTALL_URL" | PROFILE=/dev/null NVM_DIR="$NVM_DIR" bash
else
  echo "curl or wget is required to install nvm."
  exit 1
fi
```

`src/bonesinfra/runtimes/nuxt/deployment/02_run_build.sh`:

```sh
  #!/usr/bin/env bash
  set -Eeuo pipefail

  if [ -f "./.nvmrc" ]; then
    export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
    if [ -s "$NVM_DIR/nvm.sh" ]; then
      # shellcheck disable=SC1090
      source "$NVM_DIR/nvm.sh"
    elif [ -s "$HOME/.config/nvm/nvm.sh" ]; then
      # shellcheck disable=SC1090
      source "$HOME/.config/nvm/nvm.sh"
    fi
    nvm install
  fi

  if [ -f "./pnpm-lock.yaml" ]; then
    npm install -g pnpm
    pnpm install --frozen-lockfile
    pnpm generate
  elif [ -f "./yarn.lock" ]; then
    command -v corepack >/dev/null 2>&1 && corepack enable || true
    yarn install --frozen-lockfile
    yarn generate
  elif [ -f "./package-lock.json" ]; then
    npm ci --include=optional
    npm run generate
  else
    echo "No lockfile found. Run your package manager locally first."
    exit 1
  fi

```

`src/bonesinfra/runtimes/nuxt/nuxt.py`:

```py
from bonesinfra.runtimes.common import apparmor, logs, nginx, node, paths as common_paths, service, validation


def questions():
    return []


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    socket_path = f"{paths['runtime_socket_dir']}/nuxt/nuxt.sock"
    node.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="nuxt",
        apparmor_exec_paths=["/usr/bin/node"],
        apparmor_writable_paths=[],
    )
    validation.run_as_runtime_user(
        ctx,
        "Validate Nuxt server entrypoint exists as runtime user",
        "test -f .output/server/index.mjs",
    )
    service.render_app_service(
        ctx,
        paths=paths,
        name="nuxt",
        runtime_label="Nuxt app server",
        runtime_exec=(
            f"/usr/bin/env NODE_ENV=production NITRO_UNIX_SOCKET={socket_path} "
            "node .output/server/index.mjs"
        ),
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=[],
    )
    nginx.render_proxy(ctx, paths=paths, socket_path=socket_path)
    service.enable_and_start(ctx, "nuxt", apparmor_profile_name=apparmor_profile_name)

```

`src/bonesinfra/runtimes/nuxt/runtime.toml`:

```toml
template = "nuxt"
web_root = ".output/public"

[permissions]
paths = [
    { path = "*", type = "dir", mode = "750" },
    { path = "*", type = "file", mode = "640" },
    { path = ".output", type = "dir", mode = "770", recursive = true },
    { path = ".nuxt", type = "dir", mode = "770", recursive = true },
]

[shared]
paths = [
    { path = ".env", type = "file" },
    { path = "storage", type = "dir" },
]

```

`src/bonesinfra/runtimes/rails/deployment/01_install_build_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
NVM_INSTALL_URL="https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.5/install.sh"

if [ -s "$NVM_DIR/nvm.sh" ]; then
  exit 0
fi

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$NVM_INSTALL_URL" | PROFILE=/dev/null NVM_DIR="$NVM_DIR" bash
else
  echo "curl or wget is required to install nvm."
  exit 1
fi
```

`src/bonesinfra/runtimes/rails/deployment/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

command -v ruby >/dev/null 2>&1 || { echo "ruby not found"; exit 1; }
command -v bundle >/dev/null 2>&1 || { echo "bundler not found"; exit 1; }

# Load rbenv if available
if [ -d "$HOME/.rbenv" ]; then
  export PATH="$HOME/.rbenv/bin:$PATH"
  eval "$(rbenv init -)"
fi

# Install Ruby version from .ruby-version if rbenv is available
if [ -f "./.ruby-version" ] && command -v rbenv >/dev/null 2>&1; then
  rbenv install --skip-existing
fi

# Install dependencies
bundle install --deployment --without development test

# Precompile assets
if bundle exec rails assets:precompile 2>/dev/null; then
  echo "Assets precompiled."
fi

# Run database migrations
RAILS_ENV=production bundle exec rails db:migrate

# Restart puma via systemd
SERVICE_NAME="$PROJECT_NAME"
if ! command -v systemctl >/dev/null 2>&1; then
  echo "systemctl not found. Restart your app server manually."
elif systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
  systemctl restart "$SERVICE_NAME"
elif systemctl list-unit-files | grep -q "$SERVICE_NAME"; then
  systemctl start "$SERVICE_NAME"
else
  echo "No systemd service found for $SERVICE_NAME. Restart your app server manually."
fi

```

`src/bonesinfra/runtimes/rails/rails.py`:

```py
from bonesinfra.runtimes.common import apparmor, logs, nginx, paths as common_paths, ruby, service, validation


def questions():
    return [
        {
            "key": "ruby_version",
            "type": "choice",
            "label": "Ruby version",
            "choices": ["3.2", "3.3", "3.4"],
            "default": "3.3",
        },
        {
            "key": "install_postgres",
            "type": "bool",
            "label": "Install PostgreSQL client libraries?",
            "default": False,
        },
        {
            "key": "install_redis",
            "type": "bool",
            "label": "Install Redis?",
            "default": False,
        },
        {
            "key": "rails_env",
            "type": "text",
            "label": "Rails environment",
            "default": "production",
        },
    ]


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    socket_path = f"{paths['runtime_socket_dir']}/puma/puma.sock"
    runtime_write_paths = [
        f"{paths['current']}/tmp",  # noqa: S108
        f"{paths['current']}/log",
        f"{paths['current']}/storage",
    ]
    rails_env = ctx.runtime.runtime_data.get("rails_env", "production")
    ruby.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="puma",
        apparmor_exec_paths=["/usr/bin/ruby*", "/usr/bin/bundle*"],
        apparmor_writable_paths=runtime_write_paths,
    )
    validation.run_as_runtime_user(
        ctx,
        "Validate Puma availability as runtime user",
        "bundle exec puma --help >/dev/null",
    )
    service.render_app_service(
        ctx,
        paths=paths,
        name="puma",
        runtime_label="Puma",
        runtime_exec=f"/usr/bin/env RAILS_ENV={rails_env} bundle exec puma -e {rails_env} -b unix://{socket_path}",
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=runtime_write_paths,
    )
    nginx.render_proxy(ctx, paths=paths, socket_path=socket_path)
    service.enable_and_start(ctx, "puma", apparmor_profile_name=apparmor_profile_name)

```

`src/bonesinfra/runtimes/rails/runtime.toml`:

```toml
template = "rails"
web_root = "public"

[permissions]
paths = [
    { path = "*", type = "dir", mode = "750" },
    { path = "*", type = "file", mode = "640" },
    { path = "tmp", type = "dir", mode = "770", recursive = true },
    { path = "log", type = "dir", mode = "770", recursive = true },
    { path = "storage", type = "dir", mode = "770", recursive = true },
    { path = "public/assets", type = "dir", mode = "750", recursive = true },
]

[shared]
paths = [
    { path = ".env", type = "file" },
    { path = "storage", type = "dir" },
]

```

`src/bonesinfra/runtimes/sveltekit/deployment/01_install_build_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
NVM_INSTALL_URL="https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.5/install.sh"

if [ -s "$NVM_DIR/nvm.sh" ]; then
  exit 0
fi

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$NVM_INSTALL_URL" | PROFILE=/dev/null NVM_DIR="$NVM_DIR" bash
else
  echo "curl or wget is required to install nvm."
  exit 1
fi
```

`src/bonesinfra/runtimes/sveltekit/deployment/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

# Load nvm if .nvmrc is present
if [ -f "./.nvmrc" ]; then
  export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
  if [ -s "$NVM_DIR/nvm.sh" ]; then
    # shellcheck disable=SC1090
    source "$NVM_DIR/nvm.sh"
  elif [ -s "$HOME/.config/nvm/nvm.sh" ]; then
    # shellcheck disable=SC1090
    source "$HOME/.config/nvm/nvm.sh"
  fi
  nvm install
fi

# Clean install and build
rm -rf node_modules

if [ -f "./pnpm-lock.yaml" ]; then
  npm install -g pnpm
  pnpm install --frozen-lockfile
  pnpm build
elif [ -f "./yarn.lock" ]; then
  command -v corepack >/dev/null 2>&1 && corepack enable || true
  yarn install --frozen-lockfile
  yarn build
elif [ -f "./package-lock.json" ]; then
  npm install --include=optional
  npm run build
else
  echo "No lockfile found. Run your package manager locally first."
  exit 1
fi

```

`src/bonesinfra/runtimes/sveltekit/runtime.toml`:

```toml
template = "sveltekit"
web_root = "build"

[permissions]
paths = [
    { path = "*", type = "dir", mode = "750" },
    { path = "*", type = "file", mode = "640" },
    { path = "build", type = "dir", mode = "770", recursive = true },
]

[shared]
paths = [
    { path = ".env", type = "file" },
    { path = "storage", type = "dir" },
]

```

`src/bonesinfra/runtimes/sveltekit/svelte.py`:

```py
from bonesinfra.runtimes.common import apparmor, logs, nginx, node, paths as common_paths, service, validation


def questions():
    return []


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    socket_path = f"{paths['runtime_socket_dir']}/sveltekit/sveltekit.sock"
    origin = f"https://{ctx.config.domain}" if ctx.config.domain else "https://localhost"
    node.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="sveltekit",
        apparmor_exec_paths=["/usr/bin/node"],
        apparmor_writable_paths=[],
    )
    validation.run_as_runtime_user(
        ctx,
        "Validate SvelteKit build entrypoint exists as runtime user",
        "test -e build",
    )
    service.render_app_service(
        ctx,
        paths=paths,
        name="sveltekit",
        runtime_label="SvelteKit app server",
        runtime_exec=(
            f"/usr/bin/env --chdir={paths['current']} NODE_ENV=production SOCKET_PATH={socket_path} "
            f"ORIGIN={origin} node --env-file=.env build"
        ),
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=[],
    )
    nginx.render_proxy(ctx, paths=paths, socket_path=socket_path)
    service.enable_and_start(ctx, "sveltekit", apparmor_profile_name=apparmor_profile_name)

```

`src/bonesinfra/runtimes/vue/deployment/01_install_build_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
NVM_INSTALL_URL="https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.5/install.sh"

if [ -s "$NVM_DIR/nvm.sh" ]; then
  exit 0
fi

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$NVM_INSTALL_URL" | PROFILE=/dev/null NVM_DIR="$NVM_DIR" bash
else
  echo "curl or wget is required to install nvm."
  exit 1
fi
```

`src/bonesinfra/runtimes/vue/deployment/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

# Load nvm if .nvmrc is present
if [ -f "./.nvmrc" ]; then
  export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
  if [ -s "$NVM_DIR/nvm.sh" ]; then
    # shellcheck disable=SC1090
    source "$NVM_DIR/nvm.sh"
  elif [ -s "$HOME/.config/nvm/nvm.sh" ]; then
    # shellcheck disable=SC1090
    source "$HOME/.config/nvm/nvm.sh"
  fi
  nvm install
fi

# Clean install and build
rm -rf node_modules

if [ -f "./pnpm-lock.yaml" ]; then
  npm install -g pnpm
  pnpm install --frozen-lockfile
  pnpm build
elif [ -f "./yarn.lock" ]; then
  command -v corepack >/dev/null 2>&1 && corepack enable || true
  yarn install --frozen-lockfile
  yarn build
elif [ -f "./package-lock.json" ]; then
  npm install --include=optional
  npm run build
else
  echo "No lockfile found. Run your package manager locally first."
  exit 1
fi

```

`src/bonesinfra/runtimes/vue/runtime.toml`:

```toml
template = "vue"
web_root = "dist/public"

[permissions]
paths = [
    { path = "*", type = "dir", mode = "750" },
    { path = "*", type = "file", mode = "640" },
    { path = "dist", type = "dir", mode = "755", recursive = true },
]

[shared]
paths = [
    { path = ".env", type = "file" },
    { path = "storage", type = "dir" },
]

```

`src/bonesinfra/runtimes/vue/vue.py`:

```py
from bonesinfra.runtimes.common import nginx, service


def questions():
    return []


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    nginx.render_static(ctx, paths=paths)

```

`tests/__main__.py`:

```py
"""Test discovery runner — no external deps required."""

import importlib
import traceback
from pathlib import Path


def discover_tests():
    root = Path(__file__).parent
    for file in sorted(root.glob("test_*.py")):
        mod = file.stem
        print(f"\n=== {mod} ===")
        m = importlib.import_module(f"tests.{mod}")
        for name in sorted(dir(m)):
            if not name.startswith("test_"):
                continue
            fn = getattr(m, name)
            if not callable(fn):
                continue
            yield mod, name, fn


def main():
    passed = 0
    failed = 0

    for _, test_name, fn in discover_tests():
        try:
            fn()
            print(f"  OK: {test_name}")
            passed += 1
        except AssertionError:
            print(f"  FAIL: {test_name}")
            for line in traceback.format_exc().splitlines()[-3:]:
                print(f"        {line}")
            failed += 1

    print(f"\n{passed} passed, {failed} failed")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())

```

`tests/cleancode/test_no_provably_unnecessary_fallback.py`:

```py
"""Test: no provably unnecessary fallback patterns.

Detects ``or`` expressions where the left side is a literal that is
always truthy, making the right-hand side (fallback) dead code.
"""

import ast
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
SRC_DIRS = ["src", "app", "lib"]
IGNORE_DIRS = {"venv", ".venv", ".env", "node_modules", "dist", "build", "coverage", "__pycache__"}


def _find_source_files() -> list[Path]:
    files = []
    for d in SRC_DIRS:
        src = PROJECT_ROOT / d
        if not src.is_dir():
            continue
        files.extend(
            path
            for path in src.rglob("*.py")
            if not any(part in IGNORE_DIRS for part in path.relative_to(PROJECT_ROOT).parts)
        )
    return files


def _has_items(node: ast.expr) -> bool:
    match node:
        case ast.List(elts=elts) | ast.Tuple(elts=elts) | ast.Set(elts=elts):
            return len(elts) > 0
        case ast.Dict(keys=keys):
            return len(keys) > 0
        case _:
            return False


def _is_definitely_truthy(node: ast.expr) -> bool:
    result = False
    match node:
        case ast.Constant(value=bool() as value):
            result = value
        case ast.Constant(value=str() as value):
            result = len(value) > 0
        case ast.Constant(value=int() as value):
            result = value != 0
        case ast.Constant(value=float() as value):
            result = value != 0.0
        case _:
            result = _has_items(node)
    return result


_SOURCE_FILES = _find_source_files()


@pytest.mark.parametrize("filepath", _SOURCE_FILES, ids=lambda p: str(p.relative_to(PROJECT_ROOT)))
def test_no_provably_unnecessary_fallback(filepath: Path) -> None:
    code = filepath.read_text(encoding="utf-8")
    try:
        tree = ast.parse(code)
    except SyntaxError:
        pytest.skip(f"Cannot parse {filepath}")

    violations: list[str] = []

    for node in ast.walk(tree):
        if not isinstance(node, ast.BoolOp) or node.op.__class__ is not ast.Or:
            continue
        if len(node.values) < 2:
            continue
        left = node.values[0]
        if _is_definitely_truthy(left):
            violations.append(f"  L{left.lineno}: left side of `or` is always truthy")

    assert not violations, f"Unnecessary fallback(s) in {filepath.relative_to(PROJECT_ROOT)}:\n" + "\n".join(violations)

```

`tests/cleancode/test_no_suspicious_fallback.py`:

```py
"""Test: no suspicious fallback patterns.

Detects ``try/except`` blocks where the except handler returns a
success value without re-raising — silently manufacturing success
from an error path.
"""

import ast
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[2]
SRC_DIRS = ["src", "app", "lib"]
IGNORE_DIRS = {"venv", ".venv", ".env", "node_modules", "dist", "build", "coverage", "__pycache__"}


def _find_source_files() -> list[Path]:
    files = []
    for d in SRC_DIRS:
        src = PROJECT_ROOT / d
        if not src.is_dir():
            continue
        files.extend(
            path
            for path in src.rglob("*.py")
            if not any(part in IGNORE_DIRS for part in path.relative_to(PROJECT_ROOT).parts)
        )
    return files


def _has_return(stmts: list[ast.stmt]) -> bool:
    stack: list[ast.AST] = list(stmts)
    while stack:
        node = stack.pop()
        if isinstance(node, ast.Return):
            return True
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                continue
            stack.append(child)
    return False


def _has_raise(stmts: list[ast.stmt]) -> bool:
    stack: list[ast.AST] = list(stmts)
    while stack:
        node = stack.pop()
        if isinstance(node, ast.Raise):
            return True
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                continue
            stack.append(child)
    return False


def _find_returns(stmts: list[ast.stmt]) -> list[ast.Return]:
    results: list[ast.Return] = []
    stack: list[ast.AST] = list(stmts)
    while stack:
        node = stack.pop()
        if isinstance(node, ast.Return):
            results.append(node)
        for child in ast.iter_child_nodes(node):
            if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                continue
            stack.append(child)
    return results


_SOURCE_FILES = _find_source_files()


@pytest.mark.parametrize("filepath", _SOURCE_FILES, ids=lambda p: str(p.relative_to(PROJECT_ROOT)))
def test_no_suspicious_fallback(filepath: Path) -> None:
    code = filepath.read_text(encoding="utf-8")
    try:
        tree = ast.parse(code)
    except SyntaxError:
        pytest.skip(f"Cannot parse {filepath}")

    violations: list[str] = []

    for node in ast.walk(tree):
        if not isinstance(node, ast.Try):
            continue
        if not _has_return(list(node.body)):
            continue
        for handler in node.handlers:
            if _has_raise(list(handler.body)):
                continue
            violations.extend(
                f"  L{return_node.lineno}: except handler returns success without re-raise"
                for return_node in _find_returns(list(handler.body))
            )

    assert not violations, f"Suspicious fallback(s) in {filepath.relative_to(PROJECT_ROOT)}:\n" + "\n".join(violations)

```

`tests/helpers.py`:

```py
import os
import subprocess
import sys
from functools import cache
from pathlib import Path

INFRA_DIR = Path(__file__).resolve().parent.parent
SRC_DIR = INFRA_DIR / "src"
REPO_ROOT = INFRA_DIR.parent
sys.path.insert(0, str(SRC_DIR))

PYTHON_BIN = sys.executable
PYTHON_ENV = {**os.environ, "PYTHONPATH": str(SRC_DIR)}


@cache
def read(path):
    return Path(path).read_text()


def assert_contains(text, pattern, msg=""):
    assert pattern in text, f"{msg}\n  missing: {pattern!r}"


def assert_not_contains(text, pattern, msg=""):
    assert pattern not in text, f"{msg}\n  unexpected: {pattern!r}"


def assert_ordering(text, *anchors):
    idx = -1
    for anchor in anchors:
        new_idx = text.find(anchor, idx + 1)
        assert new_idx > idx, f"Must appear in order, missing earlier: {anchor!r}"


def assert_file_exists(path, msg=""):
    assert Path(path).exists(), msg or f"Missing file: {path}"


def assert_file_not_exists(path, msg=""):
    assert not Path(path).exists(), msg or f"Unexpected file: {path}"


def compile_module(path):
    source = Path(path).read_text()
    return compile(source, str(path), "exec")


def exec_module(path):
    source = Path(path).read_text()
    ns = {}
    exec(source, ns)
    return ns


def run(*args):
    result = subprocess.run(
        [sys.executable, "-m", "bonesinfra", *args],
        capture_output=True,
        text=True,
        timeout=10,
        env=PYTHON_ENV,
        check=False,
    )
    assert result.returncode == 0, f"Failed: {' '.join(args)}\n{result.stderr}"
    return result.stdout

```

`tests/test_cli.py`:

```py
"""CLI commands must run without crashing."""

import subprocess

from . import helpers

PYTHON = helpers.PYTHON_BIN
PYTHON_ENV = helpers.PYTHON_ENV


def _run_no_input(*args):
    return subprocess.run(
        [PYTHON, "-m", "bonesinfra", *args],
        capture_output=True,
        text=True,
        timeout=10,
        env=PYTHON_ENV,
        check=False,
    )


def test_runtime_list():
    result = _run_no_input("runtime", "list")
    assert result.returncode == 0, result.stderr


def test_runtime_questions_all():
    for name in ["django", "laravel", "next", "rails", "sveltekit", "vue"]:
        result = _run_no_input("runtime", "questions", name)
        assert result.returncode == 0, f"{name}: {result.stderr}"


def test_setup_apply_rejects_missing_host():
    result = _run_no_input("setup", "apply", "--config", "/dev/null")
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()


def test_runtime_apply_rejects_missing_host():
    result = _run_no_input("runtime", "apply", "--config", "/dev/null", "--runtime-config", "/dev/null")
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()


def test_ssl_apply_rejects_missing_host():
    result = _run_no_input("ssl", "apply", "--config", "/dev/null")
    assert result.returncode == 3, f"Expected exit 3 for missing host, got {result.returncode}"
    assert "missing host" in result.stderr.lower()

```

`tests/test_context.py`:

```py
"""Deploy context defaults should preserve per-project identity."""

from pathlib import Path
from tempfile import TemporaryDirectory

from bonesinfra.domain.context import DeployContext, template_data


def test_runtime_identity_defaults_to_project_name():
    with TemporaryDirectory() as tmp:
        config_path = Path(tmp) / "bones.toml"
        config_path.write_text(
            """
project_name = "lawsnipe"
repo_path = "/home/git/lawsnipe.git"
project_root = "/srv/sites/lawsnipe"
host = "example.com"
""".lstrip()
        )

        ctx = DeployContext.from_files(str(config_path))

    assert ctx.config.project_name == "lawsnipe"
    assert ctx.config.host == "example.com"
    assert ctx.config.ssh_user == "root"
    assert ctx.runtime.web_root == "public"
    assert ctx.runtime.runtime_user == "lawsnipe"
    assert ctx.runtime.runtime_group == "lawsnipe"
    assert ctx.runtime.release_group == "lawsnipe-release"
    assert ctx.ssh_port == 22

    td = template_data(ctx)
    assert td["runtime_user"] == "lawsnipe"
    assert td["runtime_group"] == "lawsnipe"


def test_runtime_identity_respects_explicit_override():
    with TemporaryDirectory() as tmp:
        config_path = Path(tmp) / "bones.toml"
        config_path.write_text(
            """
project_name = "lawsnipe"
repo_path = "/home/git/lawsnipe.git"
project_root = "/srv/sites/lawsnipe"
host = "example.com"
""".lstrip()
        )
        runtime_config_path = Path(tmp) / "runtime.toml"
        runtime_config_path.write_text(
            """
runtime_user = "lawsnipe-web"
runtime_group = "lawsnipe-web"
""".lstrip()
        )

        ctx = DeployContext.from_files(str(config_path), str(runtime_config_path))

    assert ctx.config.project_name == "lawsnipe"
    assert ctx.runtime.web_root == "public"
    assert ctx.runtime.runtime_user == "lawsnipe-web"
    assert ctx.runtime.runtime_group == "lawsnipe-web"

```

`tests/test_deploy_structure.py`:

```py
"""Operation ordering and separation of concerns in deploy plans."""

from . import helpers

SETUP_PLAN = helpers.SRC_DIR / "bonesinfra/deploys/setup/plan.py"
SETUP_PACKAGES = helpers.SRC_DIR / "bonesinfra/deploys/setup/packages.py"
SETUP_USERS = helpers.SRC_DIR / "bonesinfra/deploys/setup/users.py"
SETUP_DIRECTORIES = helpers.SRC_DIR / "bonesinfra/deploys/setup/directories.py"
SETUP_PLACEHOLDER = helpers.SRC_DIR / "bonesinfra/deploys/setup/placeholder.py"
SETUP_FIREWALL = helpers.SRC_DIR / "bonesinfra/deploys/setup/firewall.py"
SETUP_BONESREMOTE = helpers.SRC_DIR / "bonesinfra/deploys/setup/bonesremote.py"
RUNTIME_PLAN = helpers.SRC_DIR / "bonesinfra/deploys/runtime/plan.py"
RUNTIME_PACKAGES = helpers.SRC_DIR / "bonesinfra/deploys/runtime/packages.py"
RUNTIME_APPARMOR = helpers.SRC_DIR / "bonesinfra/deploys/runtime/apparmor.py"
RUNTIME_NGINX = helpers.SRC_DIR / "bonesinfra/deploys/runtime/nginx.py"
RUNTIME_DOCTOR = helpers.SRC_DIR / "bonesinfra/deploys/runtime/doctor.py"
RUNTIME_TEMPLATE = helpers.SRC_DIR / "bonesinfra/deploys/runtime/template_runtime.py"
SSL_PLAN = helpers.SRC_DIR / "bonesinfra/deploys/ssl/plan.py"
LARAVEL_DEPLOY = helpers.SRC_DIR / "bonesinfra/runtimes/laravel/deploy.py"


# ---- setup plan ----


def test_setup_plan_calls_all_steps():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_contains(c, "packages.install_system")
    helpers.assert_contains(c, "users.install_rust")
    helpers.assert_contains(c, "users.ensure_users_and_groups")
    helpers.assert_contains(c, "directories.setup_repo_and_project")
    helpers.assert_contains(c, "placeholder.seed")
    helpers.assert_contains(c, "firewall.configure")
    helpers.assert_contains(c, "bonesremote.install_authorized_key")
    helpers.assert_contains(c, "bonesremote.install")


def test_setup_plan_uses_base_packages():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_contains(c, "BASE_SYSTEM_PACKAGES")


def test_setup_plan_ordering():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_ordering(
        c,
        "packages.install_system",
        "users.install_rust",
        "users.ensure_users_and_groups",
    )


def test_setup_excludes_apparmor():
    for f in [
        SETUP_PLAN,
        SETUP_PACKAGES,
        SETUP_USERS,
        SETUP_DIRECTORIES,
        SETUP_PLACEHOLDER,
        SETUP_FIREWALL,
        SETUP_BONESREMOTE,
    ]:
        c = helpers.read(f)
        helpers.assert_not_contains(c, "apparmor_parser")
        helpers.assert_not_contains(c, "apparmor_profile_name")


def test_setup_excludes_runtime_doctor():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_not_contains(c, "bonesremote doctor")


def test_setup_excludes_per_site_roles():
    c = helpers.read(SETUP_PLAN)
    helpers.assert_not_contains(c, "bonesdeploy-nginx")
    helpers.assert_not_contains(c, "per-site nginx")


def test_setup_uses_resolved_placeholder_paths():
    c1 = helpers.read(SETUP_DIRECTORIES)
    helpers.assert_contains(c1, "placeholder_web_root")
    c2 = helpers.read(SETUP_PLACEHOLDER)
    helpers.assert_contains(c2, "placeholder_index")


def test_setup_avoids_usermod_for_existing_runtime_user():
    c = helpers.read(SETUP_USERS)
    helpers.assert_contains(c, "host.get_fact(Users)")
    helpers.assert_contains(c, "gpasswd -a")


def test_setup_deploy_user_commands_set_user_home():
    c = helpers.read(SETUP_DIRECTORIES)
    helpers.assert_contains(c, "XDG_CONFIG_HOME={home}/.config")
    helpers.assert_contains(c, "getent passwd")


# ---- Firewall ----


def test_setup_firewall_handles_ssh_cidrs():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "ssh_cidrs")


def test_setup_firewall_filters_ssh_from_allowed_ports():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, 'port == "ssh"')
    helpers.assert_contains(c, "continue")


def test_setup_firewall_resolves_port_aliases():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "port_aliases.get(port, port)")


def test_setup_firewall_sets_default_policies():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "ufw --force default")
    helpers.assert_contains(c, "firewall_default_incoming_policy")
    helpers.assert_contains(c, "firewall_default_outgoing_policy")


def test_setup_firewall_gated_by_enabled_flag():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "firewall_enabled")


def test_setup_firewall_status_gated_by_show_status():
    c = helpers.read(SETUP_FIREWALL)
    helpers.assert_contains(c, "ufw status verbose")
    helpers.assert_contains(c, "firewall_show_status")


# ---- runtime plan ----


def test_runtime_plan_calls_all_steps():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_contains(c, "packages.install_apt")
    helpers.assert_contains(c, "apparmor.setup")
    helpers.assert_contains(c, "nginx.setup")
    helpers.assert_contains(c, "template_runtime.load")
    helpers.assert_contains(c, "nginx.start_services")
    helpers.assert_contains(c, "doctor.run")


def test_runtime_applies_apparmor_and_nginx():
    c = helpers.read(RUNTIME_APPARMOR)
    helpers.assert_contains(c, "apparmor_parser -r")
    helpers.assert_contains(c, "aa-enforce")
    helpers.assert_contains(c, "apparmor_enabled_param")
    c2 = helpers.read(RUNTIME_NGINX)
    helpers.assert_contains(c2, "per-site nginx")


def test_common_apparmor_enforces_after_load():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/apparmor.py")
    helpers.assert_ordering(
        c,
        "apparmor_parser -r",
        "aa-enforce",
    )


def test_common_service_verifies_profile_attached():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/service.py")
    helpers.assert_contains(c, "validation.verify_profile_attached")
    helpers.assert_contains(c, "apparmor_profile_name=None")


def test_common_validation_verifies_proc_attr_current():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/validation.py")
    helpers.assert_contains(c, "def verify_profile_attached")
    helpers.assert_contains(c, "attr/current")
    helpers.assert_contains(c, "MainPID")


def test_app_service_uses_per_service_runtime_directory_leaf():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "RuntimeDirectory={{ project_name }}/{{ runtime_name }}")


def test_site_nginx_service_uses_nginx_runtime_directory_leaf():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "RuntimeDirectory={{ project_name }}/nginx")
    helpers.assert_contains(c, "ReadWritePaths={{ paths.runtime_nginx_dir }}")


def test_project_nginx_profile_grants_nginx_dir_and_app_sockets():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "{{ paths.runtime_nginx_dir }}/ rw,")
    helpers.assert_contains(c, "{{ paths.runtime_nginx_dir }}/** rwk,")
    helpers.assert_contains(c, "{{ paths.runtime_socket_dir }}/*/*.sock rw,")
    helpers.assert_not_contains(c, "{{ paths.runtime_socket_dir }}/** rwk,")


def test_runtime_plan_ordering():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_ordering(
        c,
        "packages.install_apt",
        "nginx.setup",
        "template_runtime.load",
        "nginx.start_services",
    )


def test_runtime_excludes_ssl_logic():
    c = helpers.read(RUNTIME_PLAN)
    helpers.assert_not_contains(c, "certbot")


def test_runtime_socket_dir_runtime_user_owned():
    c = helpers.read(RUNTIME_NGINX)
    helpers.assert_contains(c, 'path=paths["runtime_socket_dir"]')
    helpers.assert_contains(c, 'path=paths["runtime_nginx_dir"]')
    helpers.assert_contains(c, "user=ctx.runtime.runtime_user")
    helpers.assert_contains(c, "group=ctx.runtime.runtime_group")
    # 0711: system nginx (www-data) must traverse /run/<project>/ and
    # /run/<project>/nginx/ to reach the per-site nginx socket. 0750 causes 502.
    helpers.assert_contains(c, 'mode="0711"')


def test_runtime_uses_template_runtime():
    c = helpers.read(RUNTIME_TEMPLATE)
    helpers.assert_contains(c, "get_runtime(template)")


def test_runtime_doctor_deploy_user_commands_set_user_home():
    c = helpers.read(RUNTIME_DOCTOR)
    helpers.assert_contains(c, "XDG_CONFIG_HOME={home}/.config")
    helpers.assert_contains(c, "getent passwd")


# ---- ssl plan ----


def test_ssl_uses_certbot():
    c = helpers.read(SSL_PLAN)
    helpers.assert_contains(c, "certbot certonly")
    helpers.assert_contains(c, "ssl_domain")


def test_ssl_excludes_apparmor():
    c = helpers.read(SSL_PLAN)
    helpers.assert_not_contains(c, "apparmor_parser")


def test_ssl_excludes_per_site_nginx():
    c = helpers.read(SSL_PLAN)
    helpers.assert_not_contains(c, '"per-site nginx"')


def test_ssl_excludes_runtimes():
    c = helpers.read(SSL_PLAN)
    helpers.assert_not_contains(c, "runtimes")


def test_ssl_defines_nginx_inline():
    c = helpers.read(SSL_PLAN)
    helpers.assert_contains(c, "nginx_server_name")
    helpers.assert_contains(c, "router.conf.j2")
    helpers.assert_contains(c, "nginx -t")


def test_runtime_nginx_falls_back_when_domain_empty():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/runtime/nginx.py")
    helpers.assert_contains(c, "ctx.config.domain or ctx.config.preview_domain")
    helpers.assert_contains(
        c,
        'raise ValueError("domain or preview_domain is required for nginx config")',
    )


def test_ssl_requires_real_domain_for_router_render():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/ssl/plan.py")
    helpers.assert_contains(c, 'raise ValueError("domain is required for ssl nginx config")')


def test_ssl_uses_current_web_root():
    c = helpers.read(SSL_PLAN)
    helpers.assert_contains(c, "current_web_root")


# ---- Laravel-specific ordering ----


def test_laravel_validates_php_fpm_before_start():
    c = helpers.read(LARAVEL_DEPLOY)
    helpers.assert_ordering(
        c,
        "php_repo.add_php_apt_source",
        "php_packages.install_php",
    )


def test_laravel_validates_php_fpm_before_reload():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/php_fpm.py")
    helpers.assert_ordering(
        c,
        "php_fpm_pool.render_pool",
        "php_fpm_pool.validate_php_fpm",
        "php_fpm_pool.reload_php_fpm",
    )


def test_laravel_deploy_does_not_setup_php_fpm_apparmor():
    c = helpers.read(LARAVEL_DEPLOY)
    helpers.assert_not_contains(c, "apparmor")
    helpers.assert_ordering(
        c,
        "php_fpm.setup_storage_directories",
        "php_fpm.setup_pool",
        "nginx.setup",
    )


def test_laravel_nginx_validates_without_creating_runtime_dir():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/nginx.py")
    helpers.assert_ordering(
        c,
        "laravel-site-nginx.conf.j2",
        "nginx -t",
    )
    helpers.assert_not_contains(c, "runtime_socket_dir")


def test_laravel_nginx_does_not_restart_site_service_early():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/nginx.py")
    helpers.assert_not_contains(c, "Restart per-site nginx with Laravel config")


# ---- Runtime directory traversal for system nginx (www-data) ----


def test_runtime_socket_dir_is_traversable_by_system_nginx():
    """Regression: /run/<project>/ must be 0711, not 0750, so system nginx
    (www-data) can traverse it to reach /run/<project>/nginx/nginx.sock.

    0750 caused 502 on every public request after the per-site nginx layout
    change moved the socket under /run/<project>/nginx/.
    """
    runtime_nginx = helpers.read(helpers.SRC_DIR / "bonesinfra/deploys/runtime/nginx.py")
    # Both runtime dir mkdirs must use 0711; the conf dir (0750) is unrelated.
    socket_dir_block = runtime_nginx.split('path=paths["runtime_socket_dir"]')[1].split(")")[0]
    helpers.assert_contains(socket_dir_block, 'mode="0711"')
    nginx_dir_block = runtime_nginx.split('path=paths["runtime_nginx_dir"]')[1].split(")")[0]
    helpers.assert_contains(nginx_dir_block, 'mode="0711"')

    common_paths = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/paths.py")
    helpers.assert_contains(common_paths, 'mode="0711"')
    helpers.assert_not_contains(common_paths, 'mode="0750"')

    common_nginx = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/nginx.py")
    helpers.assert_contains(common_nginx, 'mode="0711"')
    helpers.assert_not_contains(common_nginx, 'mode="0750"')


def test_site_nginx_service_runtime_dir_is_traversable():
    """The per-site nginx RuntimeDirectory must be 0711 so www-data can
    traverse into /run/<project>/nginx/ to reach the socket."""
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "RuntimeDirectoryMode=0711")
    helpers.assert_not_contains(c, "RuntimeDirectoryMode=0750")


def test_app_service_runtime_dir_stays_private():
    """App runtime dirs stay 0750 — only the per-site nginx (same runtime
    user) needs to reach app sockets, so no world traversal is required."""
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "RuntimeDirectoryMode=0750")

```

`tests/test_paths.py`:

```py
"""Paths manifest must define `build_logs` and `LOGS_DIR` for centralized log handling."""

from . import helpers

CRATES_PATHS = helpers.REPO_ROOT / "crates/shared/src/paths.rs"


def test_paths_has_build_logs_constant():
    if not CRATES_PATHS.exists():
        return
    c = helpers.read(CRATES_PATHS)
    helpers.assert_contains(c, 'pub const LOGS_DIR: &str = "logs";')
    helpers.assert_contains(c, "pub build_logs: String,")
    helpers.assert_contains(
        c,
        "build_logs: Path::new(&project_root).join(BUILD_DIR).join(LOGS_DIR).display().to_string()",
    )

```

`tests/test_pyinfra_runner.py`:

```py
from contextlib import contextmanager
from pathlib import Path
from tempfile import TemporaryDirectory

import pyinfra.connectors.ssh as pyinfra_ssh

from bonesinfra.domain.context import DeployContext
from bonesinfra.infra import pyinfra_runner

sentinel_key = object()


@contextmanager
def _noop_activity(_message):
    yield


def _noop_print_target(*args, **kwargs):
    del args, kwargs


def _noop_run_ops(state):
    del state


def _noop_get_private_key(*args, **kwargs):
    del args, kwargs
    return sentinel_key


def _noop_deploy(*args, **kwargs):
    del args, kwargs


def test_run_passes_ssh_auth_through_inventory(monkeypatch):
    with TemporaryDirectory() as tmp:
        config_path = Path(tmp) / "bones.toml"
        config_path.write_text(
            """
project_name = "lawsnipe"
repo_path = "/home/git/lawsnipe.git"
project_root = "/srv/sites/lawsnipe"
host = "example.com"
ssh_user = "root"
port = 2222
""".lstrip()
        )

        ctx = DeployContext.from_files(str(config_path))

    seen = {}

    monkeypatch.setattr(pyinfra_runner, "setup_output", lambda: None)
    monkeypatch.setattr(pyinfra_runner, "print_banner", lambda: None)
    monkeypatch.setattr(pyinfra_runner, "print_target", _noop_print_target)
    monkeypatch.setattr(pyinfra_runner, "print_connected", lambda: None)
    monkeypatch.setattr(pyinfra_runner, "print_done", lambda success: seen.setdefault("done", success))
    monkeypatch.setattr(pyinfra_runner, "stop_live_output", lambda: None)
    monkeypatch.setattr(pyinfra_runner, "activity", _noop_activity)
    monkeypatch.setattr(pyinfra_runner, "run_ops", _noop_run_ops)
    monkeypatch.setattr(pyinfra_ssh, "get_private_key", _noop_get_private_key)

    def fake_connect_all(state):
        host = next(iter(state.inventory))
        seen["kwargs"] = host.connector.make_paramiko_kwargs()

    monkeypatch.setattr(pyinfra_runner, "connect_all", fake_connect_all)

    pyinfra_runner.run(ctx=ctx, ssh_key="~/.ssh/id_ed25519", deploy=_noop_deploy)

    assert seen["kwargs"]["username"] == "root"
    assert seen["kwargs"]["port"] == 2222
    assert seen["kwargs"]["pkey"] is sentinel_key
    assert seen["kwargs"]["allow_agent"] is False
    assert seen["kwargs"]["look_for_keys"] is False
    assert seen["done"] is True

```

`tests/test_runtime_nginx.py`:

```py

import pytest

from bonesinfra.deploys.runtime import nginx as runtime_nginx
from bonesinfra.deploys.ssl import plan as ssl_plan
from bonesinfra.domain.context import DeployContext
from bonesinfra.domain.paths import DeploymentPaths


def _make_ctx(tmp_path, *, domain: str = "", preview_domain: str = "preview.example.com"):
    config_path = tmp_path / "bones.toml"
    config_path.write_text(
        f"""
project_name = "lawsnipe"
repo_path = "/home/git/lawsnipe.git"
project_root = "/srv/sites/lawsnipe"
host = "example.com"
domain = "{domain}"
preview_domain = "{preview_domain}"
email = "ops@example.com"
""".lstrip()
    )
    return DeployContext.from_files(str(config_path))


def _noop(*args, **kwargs):
    del args, kwargs


def test_runtime_nginx_uses_preview_domain_when_domain_is_empty(tmp_path, monkeypatch):
    ctx = _make_ctx(tmp_path, domain="", preview_domain="preview.example.com")
    paths = DeploymentPaths.new(
        ctx.config.project_name,
        ctx.config.repo_path,
        ctx.config.project_root,
        ctx.runtime.web_root,
    ).__dict__
    calls = []

    monkeypatch.setattr(runtime_nginx, "mkdir", _noop)
    monkeypatch.setattr(runtime_nginx.files, "link", _noop)
    monkeypatch.setattr(runtime_nginx.server, "shell", _noop)
    monkeypatch.setattr(runtime_nginx.systemd, "daemon_reload", _noop)

    def fake_render(*args, **kwargs):
        calls.append((args, kwargs))

    monkeypatch.setattr(runtime_nginx, "render", fake_render)

    runtime_nginx.setup(ctx, paths, tmp_path)

    router_call = next(call for _, call in calls if "nginx_server_name" in call)
    assert router_call["nginx_server_name"] == "preview.example.com"
    assert router_call["preview_domain"] == "preview.example.com"


def test_runtime_nginx_requires_a_real_name(tmp_path, monkeypatch):
    ctx = _make_ctx(tmp_path, domain="", preview_domain="")
    paths = DeploymentPaths.new(
        ctx.config.project_name,
        ctx.config.repo_path,
        ctx.config.project_root,
        ctx.runtime.web_root,
    ).__dict__

    monkeypatch.setattr(runtime_nginx, "mkdir", _noop)
    monkeypatch.setattr(runtime_nginx.files, "link", _noop)
    monkeypatch.setattr(runtime_nginx.server, "shell", _noop)
    monkeypatch.setattr(runtime_nginx.systemd, "daemon_reload", _noop)
    monkeypatch.setattr(runtime_nginx, "render", _noop)

    with pytest.raises(ValueError, match="domain or preview_domain"):
        runtime_nginx.setup(ctx, paths, tmp_path)


def test_ssl_setup_requires_a_real_domain(tmp_path):
    ctx = _make_ctx(tmp_path, domain="", preview_domain="preview.example.com")

    with pytest.raises(SystemExit, match="1"):
        ssl_plan.deploy_ssl(ctx)

```

`tests/test_runtimes.py`:

```py
import importlib

from bonesinfra.app import runtime_catalog
from bonesinfra.runtimes import list_runtimes

from . import helpers

RUNTIMES_MODULES = {
    "laravel": "bonesinfra.runtimes.laravel",
    "django": "bonesinfra.runtimes.django.django",
    "next": "bonesinfra.runtimes.next.next",
    "nuxt": "bonesinfra.runtimes.nuxt.nuxt",
    "rails": "bonesinfra.runtimes.rails.rails",
    "sveltekit": "bonesinfra.runtimes.sveltekit.svelte",
    "vue": "bonesinfra.runtimes.vue.vue",
}


def test_runtimes_have_questions_and_deploy():
    for name, module_path in RUNTIMES_MODULES.items():
        mod = importlib.import_module(module_path)
        assert callable(getattr(mod, "questions", None)), f"{name}: missing questions()"
        assert callable(getattr(mod, "deploy", None)), f"{name}: missing deploy()"


def test_runtime_registry_is_explicit():
    assert list_runtimes() == sorted(RUNTIMES_MODULES)


def test_laravel_questions_are_exposed():
    assert runtime_catalog.get_questions("laravel")


def test_laravel_deploy_accepts_ctx():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/deploy.py")
    helpers.assert_contains(content, "def deploy(ctx):", "laravel deploy must accept ctx")


def test_laravel_php_fpm_uses_distro_pool_model():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/php_fpm.py")
    helpers.assert_contains(content, "php_fpm_pool.render_pool")
    helpers.assert_contains(content, "php_fpm_pool.validate_php_fpm")
    helpers.assert_contains(content, "php_fpm_pool.reload_php_fpm")


def test_laravel_php_fpm_does_not_use_custom_service():
    """Regression guard: Laravel must not render or start a per-project
    php-fpm systemd service, nor validate via a custom --fpm-config."""
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/php_fpm.py")
    helpers.assert_not_contains(content, "site-php-fpm.service.j2")
    helpers.assert_not_contains(content, "-php-fpm.service")
    helpers.assert_not_contains(content, "--fpm-config")
    helpers.assert_not_contains(content, "apparmor")


def test_laravel_deploy_does_not_setup_php_fpm_apparmor():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/deploy.py")
    helpers.assert_not_contains(content, "apparmor.setup_php_fpm")
    helpers.assert_not_contains(content, "apparmor")


def test_common_php_fpm_pool_socket_path_is_distro_standard():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/php_fpm_pool.py")
    helpers.assert_contains(content, '"/run/php"')
    helpers.assert_contains(content, "fpm/pool.d/{project}.conf")
    helpers.assert_contains(content, "php-fpm{php_version} --test")
    helpers.assert_contains(content, "php{php_version}-fpm")


def test_common_php_fpm_pool_ensures_bonesdeploy_log_dir():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/php_fpm_pool.py")
    helpers.assert_contains(content, "logs.ensure(ctx)")


def test_laravel_nginx_uses_distro_php_socket_and_no_runtime_chown():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/laravel/nginx.py")
    helpers.assert_contains(content, "php_fpm_pool.socket_path")
    helpers.assert_not_contains(content, "runtime_socket_dir")
    helpers.assert_not_contains(content, "files.directory")


def test_common_validation_runs_as_runtime_user_not_root():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/validation.py")
    helpers.assert_contains(content, "_sudo_user=user")
    helpers.assert_contains(content, "def run_as_runtime_user(ctx, name, command):")


def test_common_logs_provisions_runtime_owned_dir():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/common/logs.py")
    helpers.assert_contains(content, "/var/log/bonesdeploy")
    helpers.assert_contains(content, "user=ctx.runtime.runtime_user")
    helpers.assert_contains(content, 'name="Ensure BonesDeploy log root exists"')


def test_template_runtime_load_fails_loudly_without_silent_swallow():
    content = helpers.read(
        helpers.SRC_DIR / "bonesinfra/deploys/runtime/template_runtime.py"
    )
    helpers.assert_not_contains(content, "except (ImportError, KeyError)")
    helpers.assert_not_contains(content, "    pass")
    helpers.assert_contains(content, 'raise RuntimeError(f"Runtime {template} does not expose deploy(ctx)")')


def test_template_runtime_load_requires_deploy_attribute():
    content = helpers.read(
        helpers.SRC_DIR / "bonesinfra/deploys/runtime/template_runtime.py"
    )
    helpers.assert_contains(content, 'if not hasattr(runtime, "deploy")')


def test_all_runtimes_use_common_service_layer():
    """Non-Laravel dynamic runtimes must wire through common.service, not pass."""
    runtime_to_module_file = {
        "django": "bonesinfra/runtimes/django/django.py",
        "next": "bonesinfra/runtimes/next/next.py",
        "nuxt": "bonesinfra/runtimes/nuxt/nuxt.py",
        "rails": "bonesinfra/runtimes/rails/rails.py",
        "sveltekit": "bonesinfra/runtimes/sveltekit/svelte.py",
        "vue": "bonesinfra/runtimes/vue/vue.py",
    }
    for name, rel in runtime_to_module_file.items():
        content = helpers.read(helpers.SRC_DIR / rel)
        helpers.assert_not_contains(content, "    pass", msg=name)
        helpers.assert_contains(content, "service.runtime_paths(ctx)", msg=name)


def test_vue_is_static_only():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/vue/vue.py")
    helpers.assert_contains(content, "nginx.render_static")
    helpers.assert_not_contains(content, "render_app_service")


def test_next_uses_tcp_localhost():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/next/next.py")
    helpers.assert_contains(content, "HOSTNAME=127.0.0.1")
    helpers.assert_contains(content, "port=port")
    helpers.assert_contains(content, 'runtime_address_families="AF_UNIX AF_INET"')


def test_nuxt_uses_nitro_unix_socket():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/nuxt/nuxt.py")
    helpers.assert_contains(content, "NITRO_UNIX_SOCKET=")
    helpers.assert_contains(content, "socket_path=socket_path")


def test_sveltekit_uses_socket_path_env():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/sveltekit/svelte.py")
    helpers.assert_contains(content, "SOCKET_PATH=")
    helpers.assert_contains(content, "ORIGIN=")


def test_django_uses_gunicorn_unix_socket():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/django/django.py")
    helpers.assert_contains(content, "gunicorn")
    helpers.assert_contains(content, "--bind unix:")
    helpers.assert_contains(content, "wsgi_module")
    helpers.assert_not_contains(content, "python3-gunicorn")


def test_app_runtimes_use_per_service_socket_leaf():
    """Each app runtime must place its socket in its own leaf dir under
    /run/<project>/<runtime>/, not in the shared /run/<project>/."""
    for slug, rel in [
        ("gunicorn", "bonesinfra/runtimes/django/django.py"),
        ("puma", "bonesinfra/runtimes/rails/rails.py"),
        ("nuxt", "bonesinfra/runtimes/nuxt/nuxt.py"),
        ("sveltekit", "bonesinfra/runtimes/sveltekit/svelte.py"),
    ]:
        content = helpers.read(helpers.SRC_DIR / rel)
        helpers.assert_contains(
            content,
            f"runtime_socket_dir']}}/{slug}/{slug}.sock",
            msg=slug,
        )


def test_rails_uses_puma_unix_socket():
    content = helpers.read(helpers.SRC_DIR / "bonesinfra/runtimes/rails/rails.py")
    helpers.assert_contains(content, "bundle exec puma")
    helpers.assert_contains(content, "-b unix://")

```

`tests/test_syntax.py`:

```py
"""All .py files under infra/ must parse without syntax errors."""

from . import helpers


def test_all_source_files_parse():
    for root in (helpers.INFRA_DIR, helpers.SRC_DIR):
        for file in sorted(root.rglob("*.py")):
            helpers.compile_module(file)

```

`tests/test_templates_j2.py`:

```py
"""J2 template file existence and content assertions."""

from . import helpers

N = helpers.SRC_DIR / "bonesinfra"


def _read(name):
    return helpers.read(N / name)


# ---- Base AppArmor profile ----


def test_apparmor_profile_allows_resolved_web_root():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "{{ paths.current_web_root }}/** r,")
    helpers.assert_contains(c, "{{ paths.releases }}/*/{{ web_root }}/** r,")


def test_apparmor_profile_allows_site_nginx_conf():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "{{ paths.site_nginx_config }} r,")


def test_apparmor_profile_allows_repo_bones_toml():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "{{ paths.repo_bones_toml }} r,")


def test_apparmor_profile_does_not_deny_home_globally():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_not_contains(c, "deny /home/** r,")
    helpers.assert_not_contains(c, "deny /home/{{ deploy_user }}/** r,")


def test_apparmor_profile_limits_network_to_unix_stream():
    c = _read("assets/apparmor/project-nginx-profile.j2")
    helpers.assert_contains(c, "network unix stream,")
    helpers.assert_not_contains(c, "network inet stream,")
    helpers.assert_not_contains(c, "network inet6 stream,")


# ---- Base nginx service template ----


def test_nginx_service_sets_apparmor_profile():
    c = _read("assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "AppArmorProfile=")


def test_nginx_service_waits_for_apparmor():
    c = _read("assets/nginx/site-nginx.service.j2")
    helpers.assert_contains(c, "After=network.target apparmor.service")
    helpers.assert_contains(c, "Requires=apparmor.service")


# ---- Base nginx config ----


def test_site_nginx_config_logs_under_runtime_nginx_dir():
    c = _read("assets/nginx/site-nginx.conf.j2")
    helpers.assert_contains(c, "error_log {{ paths.runtime_nginx_dir }}/error.log")
    helpers.assert_contains(c, "access_log {{ paths.runtime_nginx_dir }}/access.log")
    helpers.assert_not_contains(c, "access_log stderr")


# ---- Router nginx config ----


def test_router_config_uses_resolved_socket_path():
    c = _read("assets/nginx/router.conf.j2")
    helpers.assert_contains(c, "{{ paths.runtime_nginx_socket }}")
    helpers.assert_not_contains(c, "default_server")


# ---- Laravel PHP-FPM pool config ----


def test_laravel_php_fpm_pool_has_no_global_section():
    c = _read("runtimes/laravel/assets/php/php-fpm-pool.conf.j2")
    helpers.assert_not_contains(c, "[global]")
    helpers.assert_not_contains(c, "daemonize")
    helpers.assert_not_contains(c, "/var/log/php-fpm.log")


def test_laravel_php_fpm_pool_uses_distro_run_php_socket():
    c = _read("runtimes/laravel/assets/php/php-fpm-pool.conf.j2")
    helpers.assert_contains(c, "listen = {{ laravel_php_fpm_socket_path }}")
    helpers.assert_not_contains(c, "{{ paths.runtime_socket_dir }}")
    helpers.assert_not_contains(c, "/run/{{ project_name }}")


def test_laravel_php_fpm_pool_listens_as_www_data():
    c = _read("runtimes/laravel/assets/php/php-fpm-pool.conf.j2")
    helpers.assert_contains(c, "listen.owner = www-data")
    helpers.assert_contains(c, "listen.group = www-data")
    helpers.assert_contains(c, "listen.mode = 0660")


def test_laravel_php_fpm_pool_runs_as_runtime_user():
    c = _read("runtimes/laravel/assets/php/php-fpm-pool.conf.j2")
    helpers.assert_contains(c, "user = {{ runtime_user }}")
    helpers.assert_contains(c, "group = {{ runtime_group }}")


def test_laravel_php_fpm_pool_logs_under_bonesdeploy():
    c = _read("runtimes/laravel/assets/php/php-fpm-pool.conf.j2")
    helpers.assert_contains(c, "/var/log/bonesdeploy/{{ project_name }}/php-worker-error.log")
    helpers.assert_contains(c, "access.log = /var/log/bonesdeploy/{{ project_name }}/php-fpm-access.log")
    helpers.assert_contains(c, "slowlog = /var/log/bonesdeploy/{{ project_name }}/php-fpm-slow.log")
    helpers.assert_not_contains(c, "{{ paths.runtime_socket_dir }}")


def test_laravel_php_fpm_pool_uses_resolved_current_path():
    c = _read("runtimes/laravel/assets/php/php-fpm-pool.conf.j2")
    helpers.assert_contains(c, "chdir = {{ paths.current }}")
    helpers.assert_contains(c, "catch_workers_output = yes")
    helpers.assert_contains(c, "php_admin_flag[log_errors] = on")


# ---- Laravel nginx config ----


def test_laravel_nginx_prefers_php_over_html():
    c = _read("runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2")
    helpers.assert_contains(c, "index index.php index.html;")


def test_laravel_nginx_uses_absolute_fastcgi_params():
    c = _read("runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2")
    helpers.assert_contains(c, "include /etc/nginx/fastcgi_params;")
    helpers.assert_not_contains(c, "include fastcgi_params;")


def test_laravel_nginx_uses_resolved_path_manifest():
    c = _read("runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2")
    helpers.assert_contains(c, "pid {{ paths.runtime_nginx_pid }}")
    helpers.assert_contains(c, "listen unix:{{ paths.runtime_nginx_socket }}")
    helpers.assert_contains(c, "root {{ paths.current_web_root }}")
    helpers.assert_contains(c, "{{ paths.runtime_nginx_dir }}/")
    helpers.assert_not_contains(c, "/run/{{ project_name }}")
    helpers.assert_not_contains(c, "{{ project_root }}/current/{{ web_root }}")


def test_laravel_nginx_logs_under_runtime_nginx_dir():
    c = _read("runtimes/laravel/assets/nginx/laravel-site-nginx.conf.j2")
    helpers.assert_contains(c, "error_log {{ paths.runtime_nginx_dir }}/error.log")
    helpers.assert_contains(c, "access_log {{ paths.runtime_nginx_dir }}/access.log")
    helpers.assert_not_contains(c, "access_log stderr")


# ---- Laravel build script ----


def test_laravel_build_script_has_err_trap():
    c = _read("runtimes/laravel/deployment/02_run_build.sh")
    helpers.assert_contains(c, "trap '")
    helpers.assert_contains(c, "ERR")
    helpers.assert_contains(c, "$LINENO")
    helpers.assert_contains(c, "$BASH_COMMAND")


def test_laravel_build_script_labels_each_phase():
    c = _read("runtimes/laravel/deployment/02_run_build.sh")
    for label in [
        "Installing Composer dependencies",
        "Entering Laravel maintenance mode",
        "Installing frontend dependencies",
        "Building frontend assets",
        "Running migrations",
        "Rebuilding Laravel caches",
    ]:
        helpers.assert_contains(c, label)


# ---- Common app service template ----


def test_common_app_service_runs_as_runtime_user():
    c = _read("runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "User={{ runtime_user }}")
    helpers.assert_contains(c, "Group={{ runtime_group }}")
    helpers.assert_contains(c, "SupplementaryGroups={{ release_group }}")
    helpers.assert_contains(c, "WorkingDirectory={{ paths.current }}")
    helpers.assert_contains(c, "RuntimeDirectory={{ project_name }}/{{ runtime_name }}")
    helpers.assert_contains(c, "RuntimeDirectoryMode=0750")
    helpers.assert_contains(c, "EnvironmentFile=-{{ paths.conf_root }}/runtime.env")
    helpers.assert_contains(c, "ExecStart={{ runtime_exec }}")
    helpers.assert_contains(c, "AppArmorProfile={{ apparmor_profile_name }}")


def test_common_app_service_is_tight_sandbox():
    c = _read("runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "NoNewPrivileges=yes")
    helpers.assert_contains(c, "PrivateTmp=yes")
    helpers.assert_contains(c, "ProtectHome=yes")
    helpers.assert_contains(c, "ProtectSystem=strict")
    helpers.assert_contains(c, "RestrictNamespaces=yes")
    helpers.assert_contains(c, "LockPersonality=yes")
    helpers.assert_contains(c, "RestrictRealtime=yes")
    helpers.assert_contains(c, "SystemCallArchitectures=native")
    helpers.assert_contains(c, "CapabilityBoundingSet=")
    helpers.assert_contains(c, "AmbientCapabilities=")
    helpers.assert_contains(c, "PrivateDevices=yes")
    helpers.assert_contains(c, "ProtectKernelTunables=yes")
    helpers.assert_contains(c, "ProtectKernelModules=yes")
    helpers.assert_contains(c, "ProtectControlGroups=yes")
    helpers.assert_contains(c, 'RestrictAddressFamilies={{ runtime_address_families | default("AF_UNIX") }}')


def test_common_app_service_writes_to_runtime_and_logs_dirs():
    c = _read("runtimes/common/assets/app.service.j2")
    helpers.assert_contains(c, "ReadOnlyPaths={{ paths.current }}")
    helpers.assert_contains(
        c,
        "ReadWritePaths={{ paths.runtime_socket_dir }}/{{ runtime_name }} "
        "{{ runtime_write_paths }} /var/log/bonesdeploy/{{ project_name }}",
    )
    helpers.assert_contains(c, "StandardOutput=journal")
    helpers.assert_contains(c, "StandardError=journal")
    helpers.assert_contains(c, "Restart=always")
    helpers.assert_contains(c, "RestartSec=2")
    helpers.assert_contains(c, "WantedBy=multi-user.target")


# ---- Common app nginx proxy template ----


def test_common_app_nginx_proxies_to_socket():
    c = _read("runtimes/common/assets/app-site-nginx.conf.j2")
    helpers.assert_contains(c, "proxy_pass {{ app_proxy_target }}")
    helpers.assert_contains(c, "proxy_set_header Host $host")
    helpers.assert_contains(c, "proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for")
    helpers.assert_contains(c, "proxy_set_header X-Forwarded-Proto $scheme")
    helpers.assert_contains(c, "proxy_set_header X-Real-IP $remote_addr")
    helpers.assert_contains(c, "proxy_http_version 1.1")
    helpers.assert_contains(c, 'proxy_set_header Connection ""')


def test_common_app_nginx_logs_under_runtime_nginx_dir():
    c = _read("runtimes/common/assets/app-site-nginx.conf.j2")
    helpers.assert_contains(c, "error_log {{ paths.runtime_nginx_dir }}/error.log")
    helpers.assert_contains(c, "access_log {{ paths.runtime_nginx_dir }}/access.log")
    helpers.assert_not_contains(c, "access_log stderr")


# ---- Common static nginx template ----


def test_common_static_nginx_serves_dist():
    c = _read("runtimes/common/assets/static-site-nginx.conf.j2")
    helpers.assert_contains(c, "root {{ paths.current }}/dist")
    helpers.assert_contains(c, "index index.html")
    helpers.assert_contains(c, "try_files $uri $uri/ /index.html")


def test_common_static_nginx_has_no_proxy_pass():
    c = _read("runtimes/common/assets/static-site-nginx.conf.j2")
    helpers.assert_not_contains(c, "proxy_pass")


# ---- Common app AppArmor profile ----


def test_common_apparmor_profile_includes_exec_paths():
    c = _read("runtimes/common/assets/app-profile.j2")
    helpers.assert_contains(c, "{{ apparmor_profile_name }}")
    helpers.assert_contains(c, "{% for exec_path in apparmor_exec_paths %}")
    helpers.assert_contains(c, "mrix")


def test_common_apparmor_profile_allows_runtime_and_log_dirs():
    c = _read("runtimes/common/assets/app-profile.j2")
    helpers.assert_contains(c, "{{ paths.runtime_socket_dir }}/{{ apparmor_runtime }}/ rw,")
    helpers.assert_contains(c, "{{ paths.runtime_socket_dir }}/{{ apparmor_runtime }}/** rwk,")
    helpers.assert_contains(c, "/var/log/bonesdeploy/{{ project_name }}/ rw,")
    helpers.assert_contains(c, "/var/log/bonesdeploy/{{ project_name }}/** rwk,")


def test_common_apparmor_profile_uses_configurable_network():
    c = _read("runtimes/common/assets/app-profile.j2")
    helpers.assert_contains(c, '{{ apparmor_network | default("network unix stream,") }}')


def test_common_apparmor_profile_denies_root_and_ssh():
    c = _read("runtimes/common/assets/app-profile.j2")
    helpers.assert_contains(c, "deny /root/** r,")
    helpers.assert_contains(c, "deny /etc/ssh/** r,")

```

`tests/test_tripwires.py`:

```py
"""Files and directories that must NOT exist (removed in migrations)."""

from . import helpers

R = helpers.REPO_ROOT
SRC = R / "crates/bonesdeploy/src"
CRATES_EXIST = R.joinpath("crates").is_dir()


def test_old_embeds_runtimes_dir_is_removed():
    helpers.assert_file_not_exists(R / "crates/bonesdeploy/embeds/runtimes")


def test_old_embeds_kit_dir_is_removed():
    helpers.assert_file_not_exists(R / "crates/bonesdeploy/embeds/kit")


def test_old_operations_py_does_not_exist():
    for p in ("infra/src/operations.py", "infra/runtime/operations.py"):
        helpers.assert_file_not_exists(R / p)


def test_pyinfra_rs_is_deleted():
    helpers.assert_file_not_exists(SRC / "pyinfra.rs")


def test_main_rs_has_no_pyinfra_mod():
    if not CRATES_EXIST:
        return
    c = helpers.read(SRC / "main.rs")
    helpers.assert_not_contains(c, "mod pyinfra;")


def test_no_managed_pyinfra_in_shared_paths():
    if not CRATES_EXIST:
        return
    c = helpers.read(R / "crates/shared/src/paths.rs")
    helpers.assert_not_contains(c, "managed_pyinfra_venv_dir")
    helpers.assert_not_contains(c, "managed_pyinfra_binary")


def test_config_rs_no_deploy_constants():
    if not CRATES_EXIST:
        return
    c = helpers.read(R / "crates/bonesdeploy/src/config.rs")
    helpers.assert_not_contains(c, "BONES_REMOTE_SSL_DEPLOY")
    helpers.assert_not_contains(c, "BONES_REMOTE_SETUP_DEPLOY")


def test_embedded_rs_no_removed_functions():
    if not CRATES_EXIST:
        return
    c = helpers.read(R / "crates/bonesdeploy/src/embedded.rs")
    helpers.assert_not_contains(c, "struct Runtimes")
    helpers.assert_not_contains(c, "fn scaffold_runtime_template")
    helpers.assert_not_contains(c, "fn read_template_runtime_config")
    helpers.assert_not_contains(c, "fn available_templates")


def test_cli_has_apply_handlers():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/cli/app.py")
    helpers.assert_contains(c, "setup_apply_cmd")
    helpers.assert_contains(c, "runtime_apply_cmd")
    helpers.assert_contains(c, "ssl_apply_cmd")


def test_cli_has_no_unimplemented():
    c = helpers.read(helpers.SRC_DIR / "bonesinfra/cli/app.py")
    helpers.assert_not_contains(c, "UnimplementedError")


def test_infra_has_pyinfra_runner():
    helpers.assert_file_exists(helpers.SRC_DIR / "bonesinfra/infra/pyinfra_runner.py")


def test_infra_has_paths():
    helpers.assert_file_exists(helpers.SRC_DIR / "bonesinfra/domain/paths.py")

```