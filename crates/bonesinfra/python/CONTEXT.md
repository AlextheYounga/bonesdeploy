# BonesInfra Project Notes

BonesInfra is the Python provisioning engine embedded in the BonesDeploy
monorepo and materialized as the project-local managed framework.

It is not the public product interface. It is called by `bonesdeploy` to run pyinfra-based provisioning, runtime setup, SSL setup, and runtime-specific infrastructure tasks.

The user should normally never call `bonesinfra` directly, except for dev
testing. The Rust `bonesinfra` crate embeds this Python tree into the
`bonesdeploy` binary, materializes the complete distribution into
`infra/.framework/`, creates a project-scoped dependency virtualenv, and
invokes that project-local package with `python -m bonesinfra`.

______________________________________________________________________

# Role in the System

BonesDeploy is split into three cooperating systems:

```text
bonesdeploy
  Local Rust CLI.
  Public user interface.

bonesremote
  Remote Rust release lifecycle executor.

bonesinfra
  Hidden Python/pyinfra provisioning engine.
```

BonesInfra owns provisioning.

BonesRemote owns deployment.

BonesDeploy owns public UX.

______________________________________________________________________

# What BonesInfra Owns

BonesInfra owns:

- pyinfra API integration
- setup provisioning
- setup disables and unloads `algif_aead` to prevent Copy Fail (CVE-2026-31431) exploitation
- runtime provisioning
- SSL provisioning
- loading and running the project's `.bones/infra` infrastructure
- canonical framework infrastructure and scaffold resources
- Jinja2 templates used by provisioning
- runtime package installation
- Ruby runtime installation from pinned, checksum-verified official source archives
- runtime services
- nginx/AppArmor/systemd provisioning details

Setup provisioning gives each `<site>-build` user its own persistent home,
distribution-allocated subordinate UID/GID mappings, and a lingering systemd
user manager for rootless Podman. Runtime application users remain home-less
and non-login.

Repository and site paths are derived from `project_name`: `repo_path` defaults to `/home/git/<project>.git`, `bones_repo` defaults to `/root/.config/bonesremote/repos/<project>.bones.git`, and `project_root` defaults to `/srv/sites/<project>`.

Each build user's outer `user-<UID>.slice` is limited by root-owned systemd
resource control at 80% CPU quota, 80% memory high, and 80% memory max, plus
`MemorySwapMax=0` so a runaway build cannot thrash host swap.
CPUQuota is that percentage of each online CPU; MemoryHigh is the soft
reclaim/throttling threshold, while MemoryMax is the hard cgroup ceiling, so
exceeding it fails the build rather than starving the host. These are
host-level limits, not rootless Podman delegation.

BonesInfra does not own:

- public user UX
- local project initialization
- `.bones` scaffold ownership
- git hook lifecycle
- release activation
- release rollback
- release pruning
- source checkout
- build workspace lifecycle

Those belong to `bonesdeploy` and `bonesremote`.

______________________________________________________________________

# Public vs Private Interface

BonesInfra exposes a command-line interface because Rust needs a stable process boundary.

That CLI is private.

The Python package's development entrypoint is:

```text
bonesinfra = "bonesinfra.__main__:main"
```

The private command shapes currently used by the Rust CLI are:

```sh
bonesinfra helpers apply --env-file <.env>
bonesinfra setup apply --env-file <.env>
bonesinfra runtime apply --env-file <.env>
bonesinfra ssl apply --env-file <.env>
bonesinfra services apply --env-file <.env>
bonesinfra manifest show --env-file <.env>
```

`ssh_user` is read from the root `.env` (`SSH_USER` key, default `"root"`) instead of a CLI flag.

This command surface is an internal contract with `bonesdeploy`. Runtime
questions are owned by the Rust runtime definitions under
`crates/bonesdeploy/src/frameworks/`; BonesInfra receives the resulting
`bones.toml` and does not prompt for runtime settings.

Do not treat it as public user-facing API unless that decision is made deliberately later.

______________________________________________________________________

# Package Layout

The Python source is one part of the Rust `bonesinfra` crate, not a standalone
repository. It retains a normal `src/` Python package layout so it can be
tested and installed locally.

Expected structure:

```text
crates/bonesinfra/
├── Cargo.toml                 # Rust embedding/materialization crate
├── src/lib.rs                 # embeds and runs the Python package
├── tests/pytest.rs            # Rust-side Python runtime checks
└── python/
    ├── pyproject.toml         # Python package metadata and dev tooling
    ├── docs/PROJECT.md
    ├── tests/
    └── src/bonesinfra/        # importable Python package
        ├── __init__.py
        ├── __main__.py
        ├── cli/               # Typer command definitions
        ├── config/            # DeployContext, paths, template_data
        ├── manifest.py        # artifact/service inspection and reporting
        ├── project.py         # project `.bones/infra` loader
        ├── pyinfra/           # runner + small operation helpers
        └── services/          # languages, linux, runtime services
```

The monorepo root is the repository. `crates/bonesinfra/python` is the
Python package source tree, and `crates/bonesinfra` is the Rust crate that
embeds it for production use.

For Python-only development, run tools from `crates/bonesinfra/python` with
the checked-in `pyproject.toml` and `uv.lock`. Production execution goes
through the Rust crate's embedded copy, not the working tree or a checkout of
an external repository.

The package should run through:

```sh
python -m bonesinfra
```

or through the installed script:

```sh
bonesinfra
```

Avoid `sys.path` mutation.

Avoid root-level script architecture.

______________________________________________________________________

# Layer Responsibilities

## `cli/`

Owns Typer command definitions.

Allowed:

- declare commands
- declare arguments/options
- load the deploy context and select a deploy plan
- print JSON for query commands

Not allowed:

- pyinfra operations
- TOML parsing details
- path derivation
- direct deploy plan logic
- project infrastructure loading internals
- server provisioning logic

CLI should stay thin.

Example shape:

```python
def setup_apply_cmd(config: str):
    ctx = DeployContext.from_files(config)
    run(ctx=ctx, deploy=deploy_setup)
```

## `cli/` and `pyinfra/`

Command orchestration lives in `cli/app.py` and the pyinfra bridge in
`pyinfra/runner.py`.

The CLI commands load `DeployContext.from_files()`, validate command-specific
requirements, select a deploy plan, and pass it to `pyinfra.runner.run()`.
The runner owns pyinfra connection and execution concerns. CLI code should
not contain raw pyinfra operations, and the runner should not contain
framework-specific provisioning logic.

## `config/`

Owns stable data concepts.

Examples:

```text
config/context.py
config/paths.py
```

Responsibilities:

- represent deploy context
- represent derived deployment paths
- normalize config data
- provide `template_data()` helper for Jinja2 rendering
- keep data-shaping logic testable

Config code should not import pyinfra.

`config/context.py` mirrors the top-level `bones.toml` sections:

- **`AppConfig`**: the `[app]`, `[app.server]`, `[app.dns]`, and `[app.deploy]` tables
- **`RuntimeConfig`**: the typed `[runtime]` identity fields, plus dynamic runtime settings
- **`DeployContext`**: wraps `app`, `runtime`, and `services` and provides derived deployment paths

No flat dict. No `host.data` side-channel.

## `pyinfra/`

Owns external machinery.

Current examples:

```text
pyinfra/runner.py
pyinfra/operations.py
```

Responsibilities:

- run pyinfra programmatically
- load TOML files
- receive the explicit `.env` path from the Rust subprocess boundary
- bridge CLI-selected deploy plans to pyinfra

Pyinfra code may import pyinfra.

Config code should avoid direct pyinfra dependency; the runner and deploy
plans are the pyinfra boundary.

## `project.py` and `manifest.py`

`project.py` owns loading the project's own infrastructure: it resolves
`.bones/infra` relative to `bones.toml`, imports `runtime.py` / `manifest.py`
as a package (supporting relative imports such as `from . import custom`),
and validates entrypoints before SSH.

`manifest.py` combines core declarations and the loaded project manifest to
inspect artifacts and services, and renders reports for `manifest show`.
Framework-owned runtime artifacts are declared by the project manifest, not
embedded core constants.

## `services/`

Owns pyinfra deploy operations, grouped by concern:

```text
services/
├── languages/    # Python, Node, Ruby, PHP runtime installs
├── linux/        # systemd, nginx, apparmor, firewall, validation, application
└── runtime/      # database/data services (mysql, redis, valkey, ...)
```

Deploy plan files should read like stories.

Example:

```python
def deploy_setup(ctx):     # ctx: DeployContext
    install_system_packages(ctx)
    ensure_users_and_groups(ctx)
    setup_repo_and_project_dirs(ctx)
    seed_placeholder_release(ctx)
    setup_firewall(ctx)
    install_bonesremote(ctx)
```

Sub-modules receive `(ctx, paths)` — no flat dict.

Deploy plans use `ctx.paths_dict`, derived from `ctx.app` and `ctx.runtime.web_root`, and pass it to sub-modules along with `ctx`.

Raw pyinfra operations should live in focused modules.

## `frameworks/`

Framework packages are part of the complete distribution materialized at
`infra/provision/core/src/bonesinfra/frameworks/`. The project-local package
selects its framework from the root `.env` and composes it with optional
project-owned `infra/custom/` code. It never falls back to a cached
or globally installed framework implementation.

Each generated runtime must expose a consistent interface:

```python
def deploy(ctx) -> None: ...  # ctx: DeployContext
```

The project manifest must expose `artifacts(ctx)`, `services(ctx)`, and
`mode(ctx)`. User-facing Framework template questions are defined in the Rust
CLI under `crates/bonesdeploy/src/frameworks/`; the matching snapshot under
`crates/bonesdeploy/assets/frameworks/<fw>/` is scaffolded into the project.

______________________________________________________________________

# Deploy Context

`DeployContext` is the main object passed from CLI-selected deploy plans into
pyinfra operations.

It mirrors the top-level config sections:

```python
@dataclass
class DeployContext:
    app: AppConfig
    runtime: RuntimeConfig
    services: ServicesConfig
```

## AppConfig

Typed fields read from nested `bones.toml` tables:

```text
`app.project_name`, `app.repo_path`, `app.project_root`, `app.server.host`,
`app.server.ssh_user`, `app.server.port`, `app.deploy.branch`,
`app.dns.ssl_enabled`, `app.dns.domain`, and
`app.dns.email`. Deployment-owned fields such as `app.remote_name`,
`app.deploy.deploy_on_push`, and `app.deploy.releases` remain outside
`DeployContext`.
```

## RuntimeConfig

```text
runtime_user       # process user for nginx/php-fpm (always project_name)
runtime_group      # process group (always project_name)
web_root           # release directory served by nginx (default: public)
data               # dynamic runtime-specific settings from [runtime]
```

## template_data()

For Jinja2 template rendering, use `template_data(ctx, *, paths=None, **extra)`.
It assembles a flat dict from the typed fields (project_name, runtime_user, paths, etc.)
and merges `runtime.data` for dynamic keys.

No `flat_data` property on `DeployContext`. No `host.data` side-channel.
Plan files receive `ctx` directly as a function parameter.

# Runtime Catalog

The installed runtime catalog no longer exists.

Runtimes are selected by the user-facing Rust CLI during `bonesdeploy init`,
which writes the chosen values into `.bones/bones.toml` and scaffolds the
project's `.bones/infra/` snapshot. BonesInfra then loads that project source
at deploy time via `project.load_runtime(config)` / `load_manifest(config)`,
resolving `infra/runtime.py` and `infra/manifest.py` relative to `bones.toml`
as an importable package.

Broken project infrastructure surfaces before SSH: missing entrypoints raise
`FileNotFoundError`, import failures raise `ImportError` with the file path,
and missing callables raise `TypeError`.

Rust does not depend on a Python question endpoint. It owns the prompt schema
and writes the selected values into `bones.toml` before invoking BonesInfra.

______________________________________________________________________

# PyInfra Runner

The pyinfra runner is an infrastructure adapter.

It should:

- create pyinfra config
- create inventory
- connect
- execute deploy plan with `ctx: DeployContext`
- run operations
- return nonzero exit on pyinfra failure

It should not know about Laravel, SSL, nginx, config files, or project infrastructure selection.

The runner no longer attaches a flat data dict to `host.data`. It calls `deploy(ctx)` directly, passing the typed `DeployContext`. Plan files receive the context as a parameter and pass it to sub-modules.

______________________________________________________________________

# Setup Provisioning

Setup provisioning prepares the machine.

Responsibilities:

- install base packages
- ensure deploy user
- ensure runtime user
- ensure runtime group
- create bare repo parent
- initialize bare git repo
- set bare repo default branch (symbolic-ref HEAD refs/heads/<branch>)
- create project root
- create releases directory
- create shared directory
- create the `shared/` directory without creating `.env`; operators publish the encrypted environment explicitly with `bonesdeploy secrets push`
- create trusted site registry parent directory
- seed placeholder release (only repoints `current` at the placeholder when no release exists, so re-running setup never replaces the active release)
- install deploy authorized key
- install thin post-receive hook (delegates to `bonesremote hook post-receive`)
- initialize the root-owned `.bones` config repository and install its pre-receive import hook
- configure firewall
- download, checksum-verify, version-check, and install the root-owned `0755` static `x86_64` `bonesremote` release binary matching the local `bonesdeploy` version
- install validated `/etc/sudoers.d/bonesdeploy` with exact command arguments

Setup should run as root or bootstrap SSH user.

## Runtime Identity Model

The current model uses a single per-project identity:

- **Runtime user**: `<site>` (system user, nologin, no home)
- **Runtime group**: `<site>`
- Releases are owned/sealed using the runtime group
- Directories: `releases/` is `root:runtime_group 2750`, `shared/` is `runtime_user:runtime_group 0750`
- The deploy user (`git`) is NOT added to the runtime group

## Sudoers Contract

The deploy user can run only these narrow commands via sudo:

The `.bones` repository is not included in this policy. It is owned and pushed
by `root`, so its pre-receive hook can invoke `bonesremote site receive` without
crossing a sudo boundary.

```
bonesremote hook post-receive --site *
bonesremote service restart --site *
bonesremote release rollback --site *
bonesremote release drop-failed --site *
bonesremote release prune --site *
```

The hook command itself owns the privileged deploy orchestration. No broad `bonesremote deploy --site *` sudo is granted.

## Post-Receive Hook

A thin bash script at `<repo>/hooks/post-receive` derives the site name from `$GIT_DIR` and delegates:

```bash
exec sudo bonesremote hook post-receive --site "$SITE"
```

Branch filtering and deploy policy belong in `bonesremote hook post-receive`, not in the shell hook.

Source code must be pushed to the configured deployment branch before deploy can succeed. The bare repo's default branch (HEAD) is set via `git symbolic-ref HEAD refs/heads/<branch>` during provisioning.

______________________________________________________________________

# Runtime Provisioning

Runtime provisioning prepares per-site services.

Responsibilities:

- install runtime apt packages
- configure AppArmor
- configure nginx router
- configure per-site nginx service
- generate `<project>.target` as the site lifecycle entrypoint
- register nginx and runtime services in the target's requires directory
- provision declared `[shared].paths` under `shared/`
- run runtime-specific deploy operations

BonesInfra owns site service membership. Every generated site service participates
in `<project>.target`; BonesRemote must restart exactly `<project>.target` for
deploy and rollback (`systemctl restart <project>.target`). It must not discover
services by name-prefix wildcard. The target requires every registered service,
and services are ordered before it, so an immediate service-start failure fails
the restart. BonesRemote should also
verify every required service remains active after restarting, because a
`Type=simple` process can exit shortly after systemd reports a successful start.

Runtime setup is separate from SSL.

When `app.dns.domain` is empty, runtime setup installs and starts the
project-scoped `cloudflared` Quick Tunnel service against the per-site Nginx
Unix socket. The assigned `trycloudflare.com` URL is runtime state read from
journald and can change after a restart. A real domain instead uses the public
Nginx router and Certbot; successful SSL activation removes the Quick Tunnel.

______________________________________________________________________

# SSL Provisioning

SSL provisioning obtains and enables certificates.

Responsibilities:

- render HTTP challenge config
- run certbot webroot challenge
- render HTTPS config
- validate nginx
- reload nginx

SSL is intentionally separate from runtime setup.

______________________________________________________________________

# Runtime-Specific Infrastructure

Per-framework logic is maintained canonically in this package and copied into
each project's `.bones/infra/runtime.py` by `bonesdeploy init`. The copied files
are project-owned snapshots and orchestrate framework services using neutral core
helpers in `services/`:

- `services/linux/application.py` — `deploy_server` / `deploy_static` building
  blocks shared by generated runtimes (AppArmor, systemd, nginx wiring)
- `services/linux/systemd.py`, `nginx/`, `apparmor/` — service lifecycle,
  per-site nginx, AppArmor profiles
- `services/linux/runtime_paths.py`, `runtime_logs.py`, `validation.py` — shared
  provisioning helpers
- `services/languages/` — Python, Node, Ruby, PHP runtime installs

Each generated runtime stays small and reads like a story against those
primitives. Django, Rails, Node, Vue, etc. all follow the same `deploy(ctx)`
interface, and each framework's templates live beside it in
`infra/templates/`.

______________________________________________________________________

# Error Rules

Prefer explicit failure.

Do not silently skip broken runtime modules.

Do not silently ignore missing required config values.

Do not print success if pyinfra reports failure.

Recommended exit behavior:

```text
0 = success
1 = pyinfra/deploy failure
3 = invalid input/config
```

Keep exact codes simple unless the contract defines more.

______________________________________________________________________

# Testing Rules

Add tests that enforce boundaries.

Suggested tests:

```text
- cli/ files do not import pyinfra.operations
- config/ files do not import pyinfra
- project loader tests cover relative imports and entrypoint validation
- every generated framework runtime exposes `deploy(ctx)`
- deploy context parses typed dataclasses correctly
- `template_data()` produces expected flat dict keys
- no sys.path mutation
- no source file over 300-400 lines
```

The goal is not purity.

The goal is preventing the repo from turning back into:

```text
main.py does everything
setup.py does everything else
runtime modules all have different shapes
```

______________________________________________________________________

# Documentation Rules

Do not document BonesInfra as public user-facing UX.

Do document it as:

```text
hidden Python provisioning engine for BonesDeploy
```

Avoid saying:

```text
users should run bonesinfra
bonesinfra owns deployment
bonesinfra owns git hooks
bonesinfra owns release activation
```

Prefer saying:

```text
bonesdeploy invokes bonesinfra
bonesinfra runs pyinfra provisioning
bonesremote owns release deployment
```

Update migrations are also defined here under `src/bonesinfra/patches/`.
`bonesdeploy` invokes the private `bonesinfra patches apply` command for local
Git migrations and remote pyinfra migrations. The command preserves the
`0001-config-repo` and `0002-root-config-repo` markers and uses an update-only
root SSH override for remote plans.

______________________________________________________________________

# Current Target

The current target is clarity, not cleverness.

```text
Typer CLI -> DeployContext.from_files() -> pyinfra_runner.run(ctx) -> deploy plan -> grouped operations
```

Keep files small.

Keep boundaries obvious.

Make every module explainable in one sentence.
