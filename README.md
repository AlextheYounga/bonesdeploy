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
| Vue | Working | Static frontend setup |
| Nuxt | Working | Nuxt runtime setup |
| Svelte | Planned | Not working yet |
| Django | Planned | Not working yet |
| Rails | Planned | Not working yet |
| Next.js | Planned | Not working yet |

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

This creates:

```text
.bones/
├── bones.toml
├── runtime.toml
├── hooks/
│   ├── hooks.sh
│   ├── pre-push
│   └── post-receive
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

Check the setup:

```sh
bonesdeploy doctor
```

Check only the local side:

```sh
bonesdeploy doctor --local
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
remote_name = "production"
project_name = "myproject"
repo_path = "/home/git/myproject.git"
project_root = "/srv/sites/myproject"
port = "22"
branch = "master"
domain = ""
preview_domain = ""
email = ""
deploy_on_push = false
ssl_enabled = false
releases = 5
```

Common defaults:

## Project Structure

```
.bones/
├── bones.toml           # project configuration
├── runtime.toml         # framework runtime configuration
├── hooks/
│   ├── hooks.sh         # (legacy) shared hook functions imported by hook entrypoints
│   ├── pre-push         # symlinked to .git/hooks/pre-push
│   └── post-receive     # thin adapter → calls bonesremote deploy
└── deployment/
    ├── build/
    │   └── 01_*.sh      # build scripts (run sequentially in the Debian build container)
    └── prepare/
        └── 01_*.sh      # prepare scripts (run as the site user before activation)
```

Hooks are written to `.bones/hooks/` once during init. `pre-push` is now a self-contained guard; remote `post-receive` is a thin trigger that delegates to `sudo bonesremote hook post-receive --site <project>`. After that they belong to you and can be edited freely. Build scripts in `.bones/deployment/build/` must be numbered (for example `01_install_deps.sh`, `02_build.sh`) and run in order inside bonesremote's Debian build container. Prepare scripts in `.bones/deployment/prepare/` also run in order, but on the host as the site runtime user after shared paths are wired and before activation.

Git hooks exist as an optional transport — `bonesdeploy deploy` is the primary deployment command. `post-receive` is a thin adapter that delegates to `bonesremote hook post-receive`, which resolves policy from bonesremote-managed site state.

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
