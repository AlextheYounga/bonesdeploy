# BonesInfra Project Notes

BonesInfra is the hidden Python provisioning engine embedded in the BonesDeploy
monorepo.

It is not the public product interface. It is called by `bonesdeploy` to run pyinfra-based provisioning, runtime setup, SSL setup, and runtime-specific infrastructure tasks.

The user should normally never call `bonesinfra` directly, except for dev
testing. The Rust `bonesinfra` crate embeds this Python tree into the
`bonesdeploy` binary, materializes it into an isolated cache, creates its
virtualenv, and invokes `python -m bonesinfra`.

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
- runtime catalog
- framework-specific infrastructure
- Jinja2 templates used by provisioning
- runtime package installation
- runtime services
- nginx/AppArmor/systemd provisioning details

Setup provisioning gives each `<site>-build` user its own persistent home,
distribution-allocated subordinate UID/GID mappings, and a lingering systemd
user manager for rootless Podman. Runtime application users remain home-less
and non-login.

Repository and site paths are always derived from `project_name`: `repo_path` defaults to `/srv/git/<project>.git` and `project_root` defaults to `/srv/sites/<project>`.

Each build user's outer `user-<UID>.slice` is limited by root-owned systemd
resource control at 80% CPU quota, 80% memory high, and 80% memory max.
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
bonesinfra runtime list
bonesinfra helpers apply --config <bones.toml>
bonesinfra setup apply --config <bones.toml>
bonesinfra runtime apply --config <bones.toml>
bonesinfra ssl apply --config <bones.toml>
```

`ssh_user` is read from `bones.toml` (`app.server.ssh_user` key, default `"root"`) instead of a CLI flag.

This command surface is an internal contract with `bonesdeploy`. Runtime
questions are owned by the Rust runtime definitions under
`crates/bonesdeploy/src/runtimes/`; BonesInfra receives the resulting
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
        ├── cli/
        ├── domain/
        ├── infra/
        ├── deploys/
        ├── runtimes/
        └── assets/
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
- runtime module loading internals
- server provisioning logic

CLI should stay thin.

Example shape:

```python
def setup_apply_cmd(config: str):
    ctx = DeployContext.from_files(config)
    run(ctx=ctx, config_path=config, deploy=deploy_setup)
```

## `cli/` and `infra/`

The current package keeps command orchestration in `cli/app.py` and the
pyinfra bridge in `infra/pyinfra_runner.py`; there is no separate Python
`app/` layer.

The CLI commands load `DeployContext.from_files()`, validate command-specific
requirements, select a deploy plan, and pass it to `infra.pyinfra_runner.run()`.
The runner owns pyinfra connection and execution concerns. CLI code should
not contain raw pyinfra operations, and the runner should not contain
framework-specific provisioning logic.

## `domain/`

Owns stable data concepts.

Examples:

```text
domain/context.py
domain/paths.py
```

Responsibilities:

- represent deploy context
- represent derived deployment paths
- normalize config data
- provide `template_data()` helper for Jinja2 rendering
- keep data-shaping logic testable

Domain code should not import pyinfra.

`domain/context.py` mirrors the top-level `bones.toml` sections:

- **`AppConfig`**: the `[app]`, `[app.server]`, `[app.dns]`, and `[app.deploy]` tables
- **`RuntimeConfig`**: the typed `[runtime]` identity fields, plus dynamic runtime settings
- **`DeployContext`**: wraps `app`, `runtime`, and `dbs` and provides derived deployment paths

No flat dict. No `host.data` side-channel.

## `infra/`

Owns external machinery.

Current examples:

```text
infra/pyinfra_runner.py
```

Responsibilities:

- run pyinfra programmatically
- load TOML files
- read stdin JSON overrides if supported
- bridge CLI-selected deploy plans to pyinfra

Infra code may import pyinfra.

Domain code should avoid direct pyinfra dependency; the runner and deploy
plans are the pyinfra boundary.

## `deploys/`

Owns pyinfra deploy plans.

Expected shape:

```text
deploys/
├── setup/
│   ├── plan.py
│   ├── packages.py
│   ├── users.py
│   ├── directories.py
│   ├── firewall.py
│   └── bonesremote.py
├── runtime/
│   ├── plan.py
│   ├── packages.py
│   ├── apparmor.py
│   ├── nginx.py
│   ├── template_runtime.py
└── ssl/
    ├── plan.py
    ├── nginx.py
    └── certbot.py
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

## `runtimes/`

Owns runtime-specific infrastructure.

Examples:

```text
runtimes/
├── __init__.py
├── common/
├── laravel/
├── django/
├── rails/
├── next/
├── nuxt/
├── sveltekit/
└── vue/
```

Each runtime should expose a consistent interface.

Current interface:

```python
def deploy(ctx) -> None: ...  # ctx: DeployContext
```

A runtime may have a no-op deploy, but it should be explicit. Runtime modules
are selected by the Python catalog for infrastructure application; user-facing
runtime questions are defined in the Rust CLI.

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
    dbs: DbsConfig
```

## AppConfig

Typed fields read from nested `bones.toml` tables:

```text
`app.project_name`, `app.repo_path`, `app.project_root`, `app.server.host`,
`app.server.ssh_user`, `app.server.port`, `app.deploy.branch`,
`app.dns.preview_domain`, `app.dns.ssl_enabled`, `app.dns.domain`, and
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

The runtime catalog should be explicit.

Avoid silent import failure.

Bad:

```python
try:
    import runtime
except ImportError:
    pass
```

Good:

```python
RUNTIMES = {
    "django": django,
    "laravel": laravel,
    "next": next_runtime,
    "nuxt": nuxt,
    "rails": rails,
    "sveltekit": svelte,
    "vue": vue,
}
```

If a declared runtime is broken, tests should fail.

Runtime query commands should return stable JSON:

```sh
bonesinfra runtime list
```

Expected `runtime list` response:

```json
["django", "laravel", "next", "nuxt", "rails", "sveltekit", "vue"]
```

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

It should not know about Laravel, SSL, nginx, config files, or runtime selection.

The runner no longer attaches a flat data dict to `host.data`. It calls `deploy(ctx)` directly, passing the typed `DeployContext`. Plan files receive the context as a parameter and pass it to sub-modules.

______________________________________________________________________

# Setup Provisioning

Setup provisioning prepares the machine.

Responsibilities:

- install base packages
- install Rust/cargo if needed
- ensure deploy user
- ensure runtime user
- ensure runtime group
- create bare repo parent
- initialize bare git repo
- set bare repo default branch (symbolic-ref HEAD refs/heads/<branch>)
- create project root
- create releases directory
- create shared directory
- seed blank `shared/.env`
- create trusted site registry parent directory
- seed placeholder release
- install deploy authorized key
- install thin post-receive hook (delegates to `bonesremote hook post-receive`)
- configure firewall
- install `bonesremote`
- install validated `/etc/sudoers.d/bonesdeploy`

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

Runtime modules should be small and grouped by concern.

Example Laravel layout:

```text
runtimes/laravel/
├── __init__.py
├── metadata.py
├── deploy.py
├── php_repo.py
├── php_packages.py
├── php_fpm.py
└── nginx.py
```

Laravel should not be one giant file.

Shared runtime helpers live under `runtimes/common/`, including common AppArmor, nginx, service, Node, path, validation, logging, and PHP-FPM pool helpers.

Django, Rails, Node, Vue, etc. can stay small, but they should still follow the same interface.

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
- domain/ files do not import pyinfra
- runtime registry imports every declared runtime
- every runtime exposes `deploy(ctx)` or an explicit no-op deploy
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

______________________________________________________________________

# Current Target

The current target is clarity, not cleverness.

```text
Typer CLI -> DeployContext.from_files() -> pyinfra_runner.run(ctx) -> deploy plan -> grouped operations
```

Keep files small.

Keep boundaries obvious.

Make every module explainable in one sentence.
