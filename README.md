# BonesDeploy

A deployment CLI for plain Linux servers.

<div style="margin:0 auto; display: block;">
  <img width="600" height="600" src="docs/images/bonesdeploy.png" alt="BonesDeploy" />
</div>

> WARNING: BonesDeploy is still under active development. You probably shouldn't use this yet. There may be some cool bugs!

BonesDeploy deploys project releases to a remote Linux server over SSH. It scaffolds deployment configs and scripts into your repo, publishes the `.bones/` dataset into root-owned `bonesremote` site state, and runs the release lifecycle remotely without turning the bare Git repo into the control plane.

No platform.  
No control plane.  
No required Docker setup.  
No pretending your VPS is a tiny Kubernetes cluster.

It gives you versioned releases, rollback, shared runtime state, service restarts, and per-site Linux isolation using the tools already on the box.

**It's also AI agent friendly, with dedicated commands to help your agent understand how to setup and manage your server without ever leaving your machine.**

BonesDeploy builds two binaries:

- **`bonesdeploy`** — the local CLI
- **`bonesremote`** — the remote release runner

And wraps a Python runtime:
- **`bonesinfra`** - https://github.com/AlextheYounga/bonesinfra

## The Point

Deploying small apps should not require a platform team.

Most apps need a few boring things done correctly:

- put each release in its own directory
- keep uploads and runtime files outside the release
- restart the right service
- keep a few old releases around
- roll back without drama
- stop one site from casually reading another site's files

That is what BonesDeploy is for.

## Site Isolation

This is the part I care about.

BonesDeploy treats each site as its own thing on the server. Each site gets its own isolated services via systemd.

Each site can get its own:

- Linux user
- Linux group
- writable shared paths
- systemd runtime services
- nginx config
- AppArmor policy
- Seccomps configs

The deploy user deploys.  
The runtime user runs the app.  
Root provisions the machine.

That is the whole model.

## Why Not Just Docker?

Docker is useful. It gives you packaging, repeatability, and another layer of isolation.

But Docker is heavy, and slow, and you see this when you try running multiple Docker sites on a machine with less than 8GB of RAM. 

Docker is also where a lot of people hide from Linux.

Instead of setting up users, groups, permissions, services, sockets, nginx, PHP-FPM, AppArmor, and runtime directories correctly, we stuff the app in a container and call it done.

Sometimes that is the right trade.

BonesDeploy takes the other trade.

It assumes the server is the deployment target, and then does the annoying work of centralizing the Linux setup per site.

You can still use Docker with BonesDeploy. Put `docker compose` in your deploy scripts.

Docker just is not the foundation.

## Runtime Templates

Runtime templates set up the Linux pieces for a framework.

| Template | Status | Notes |
| --- | --- | --- |
| Laravel | Working | PHP / PHP-FPM setup |
| Next.js | Working | Node runtime setup |
| Nuxt | Working | Nuxt runtime setup |
| Vue | Working | Static frontend setup |
| Django | Not tested | Python / Gunicorn not tested yet |
| Rails | Not tested | Ruby not tested yet |

Templates are not magic. They are shared server setup so every project does not become a custom snowflake.

## Install

Install the local CLI:

```sh
cargo install --locked --git https://github.com/AlextheYounga/bonesdeploy.git bonesdeploy
```

Install the remote runner on the server:

```sh
sudo cargo install --locked --root /usr/local --git https://github.com/AlextheYounga/bonesdeploy.git bonesremote --force
```

Remote host provisioning, including sudoers policy, is handled by `bonesinfra` during `bonesdeploy init` remote setup.

## Start a Project

From your project repo:

```sh
bonesdeploy init
```

For CI or AI agents, pick a runtime template and pass variables non-interactively:

```sh
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template laravel --runtime-var php_version=8.5 --db postgres --db valkey
```

See `bonesdeploy skill doc templates` for every template and its variables.

This creates:

```text
.bones/
├── bones.toml
└── deployment/
    └── 01_*.sh
```

The files are yours.

Edit them.

Commit them.

Read them when something breaks.

Deployment scripts run in filename order:

```text
01_install_deps.sh
02_build.sh
03_migrate.sh
```

## Set Up the Server

Provision the base server:

```sh
bonesdeploy remote setup
```

Provision the site runtime:

```sh
bonesdeploy remote runtime
```

Database services selected at init are provisioned by `bonesdeploy setup`, or later with:

```sh
bonesdeploy remote dbs
```

Supported services are PostgreSQL, MariaDB, MySQL, MongoDB, Valkey, and Redis. They listen only on localhost; use an SSH tunnel for workstation access. Generated credentials live in the protected remote `shared/.env`, never in `.bones/`. MariaDB and MySQL are alternatives and cannot share one host.

Add SSL after DNS points at the server:

```sh
bonesdeploy remote ssl --domain app.example.com --email ops@example.com
```

SSL is separate on purpose. Get the site working first. Add certificates after DNS is real.

## Deploy

Deploy:

```sh
bonesdeploy deploy
```

Rollback:

```sh
bonesdeploy rollback
```

Inspect releases, including a release that is currently building:

```sh
bonesdeploy releases
```

Cancel a named building or interrupted release and clean its temporary build state:

```sh
bonesdeploy releases kill 20260715_225306
```

Check the setup:

```sh
bonesdeploy doctor
```

Check only the local side:

```sh
bonesdeploy doctor --local
```

`doctor` reports three states: green checks are healthy, yellow pending items
are expected next steps (such as the first Git push after setup), and red
failures need attention. Pending first-push state exits successfully so setup
can finish without looking broken. For agents and scripts, use the stable
machine-readable next-step guide:

```sh
bonesdeploy skill next --format json
```

Embedded documentation for AI agents lives under the `skill` command:

```sh
bonesdeploy skill                    # orientation doc
bonesdeploy skill list               # names of every embedded doc
bonesdeploy skill doc workflows      # end-to-end flows
bonesdeploy skill doc methodology    # permission model and doctrine
```

Sync `.bones/` changes to the server:

```sh
bonesdeploy push
```

Update the local and remote binaries:

```sh
bonesdeploy update
```

## Config

`bonesdeploy init` creates `.bones/bones.toml`:

```toml
[app]
remote_name = "production"
project_name = "myproject"
repo_path = "/home/git/myproject.git"
project_root = "/srv/sites/myproject"

[app.server]
host = "deploy.example.com"
ssh_user = "root"
port = "22"

[app.deploy]
branch = "master"
deploy_on_push = false
releases = 5

[app.dns]
domain = ""
preview_domain = ""
email = ""
ssl_enabled = false

[build]
vars = []

[runtime]
template = "custom"
```

Common defaults:

## Project Structure

```
.bones/
├── bones.toml           # project, build, and runtime configuration
└── deployment/
    ├── build/
    │   └── 01_*.sh      # build scripts (run sequentially in the buildpack-deps container)
    └── prepare/
        └── 01_*.sh      # prepare scripts (run as the site user before activation)
```

The optional git push transport uses two thin internal adapters (local pre-push guard and remote post-receive trigger) that are embedded in the binaries. You do not see or manage them under `.bones/`. Set `deploy_on_push = true` in `.bones/bones.toml` to enable git-triggered deploys; the default is `false`.

Build scripts in `.bones/deployment/build/` must be numbered (for example `01_install_deps.sh`, `02_build.sh`) and run in order inside bonesremote's `buildpack-deps:bookworm` container. Bonesremote streams an ephemeral copy of the deployment bundle into the container at `/workspace/deployment`, so the build user never needs host access to bonesremote's control-plane files. BonesInfra provisions a private persistent cache for each build user; bonesremote mounts it at `/workspace/cache` and exposes `BUILD_CACHE_DIR`. The shared deployment functions use it for Node, Corepack, npm, pnpm, Yarn, Composer, and Bundler downloads. Installed dependency trees and build output remain disposable. Prepare scripts in `.bones/deployment/prepare/` also run in order, but on the host as the site runtime user after shared paths are wired and before activation. Bonesremote streams the shared functions into each prepare shell before the prepare script, so prepare scripts do not source the root-owned deployment bundle.

Build scripts can set framework-specific runtime options such as `NODE_OPTIONS=--max-old-space-size=<MiB>` when a project needs a V8 heap limit. Node does not provide a general CPU-percentage limit; `UV_THREADPOOL_SIZE` only changes libuv's file-system, crypto, DNS, and zlib worker pool.

Rootless Podman commands run through the dedicated build user's systemd user manager. Deploy verifies that manager, Podman, and the Infra-provisioned build cache before staging a release. The runtime application user remains a separate home-less, non-login account and never owns or operates the build container.

Git hooks are an optional transport — `bonesdeploy deploy` is the primary deployment command. The remote `post-receive` trigger is embedded in the `bonesremote` binary and installed into the bare repo automatically.

## Good Fit

BonesDeploy is for:

- one-server apps
- VPS deployments
- small production apps
- side projects that grew up
- Raspberry Pis and old servers
- developers who want to understand their deploys
- developers who want Linux isolation without making Docker mandatory

## Bad Fit

BonesDeploy is not trying to be:

- Kubernetes
- Heroku
- Nomad
- a PaaS
- a dashboard
- a managed database service
- a multi-node orchestration layer

Use those when you need those.

## Coverage

Install:

```sh
cargo install cargo-llvm-cov
```

Run:

```sh
cargo cov
```

LCOV:

```sh
cargo cov-lcov
```

HTML:

```sh
cargo cov-html
```

Reports go here:

```text
target/coverage/
```

## License

MIT
