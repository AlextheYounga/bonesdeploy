# BonesDeploy ☠️

## Git Deployments with a Spine in a Barebones Framework 🏴‍☠️

A drop-in Rust deployment system for git-based deployments over SSH. BonesDeploy scaffolds hook scripts and deployment configs into your repo, syncs them to a remote bare repository, and manages file ownership and permissions across deploys without forcing containers, a control plane, or a platform layer.

It produces two binaries:
- **`bonesdeploy`** — local CLI for setup and management
- **`bonesremote`** — server-side tool for remote operations, installed on the deployment host

## Why BonesDeploy

BonesDeploy is built for developers who want `git push` deployments without handing deployment over to a PaaS or rebuilding everything around Docker.

- **Drop-in** — add it to an existing repo, scaffold `.bones/`, and deploy over your existing SSH + bare repo workflow
- **Git-native** — hooks, remotes, and bare repos stay the source of truth instead of hiding deployment behind a daemon
- **Permission-aware** — BonesDeploy treats deploy-user to service-user handoff as a first-class concern instead of leaving shared groups or ACL sprawl behind
- **Self-hosted and lightweight** — ideal for VPSes, old servers, and Raspberry Pis where simplicity matters more than orchestration
- **Editable by design** — the generated hooks and deployment scripts are yours; BonesDeploy gives you structure, not lock-in

If you want a Heroku-style abstraction layer, use a platform. If you want a disciplined, transparent deployment skeleton that drops into a normal Linux box, use BonesDeploy.

## How It Works

BonesDeploy uses a two-user deployment model:

1. A **deploy user** (default: `git`) handles SSH access and runs deployment scripts. This user has restricted sudo ability but no password login.
2. A **service user** (default: `applications`) owns the deployed files. This user has no home folder, no login, and no sudo ability — limiting attack scope.

During deployment, `bonesremote` temporarily changes file ownership to the deploy user so scripts can write, then hardens permissions back to the service user afterward. The sudoers configuration is strictly limited to `bonesremote` commands only.

This gives you a clean privilege boundary:

- the **deploy user** can connect and deploy
- the **service user** ends up owning the app
- `bonesremote` is the only privileged bridge between those two phases

## Installation

### Local (bonesdeploy)

```sh
cargo install --git https://github.com/AlextheYounga/bonesdeploy.git bonesdeploy
```

### Server (bonesremote)

```sh
sudo cargo install --root /usr/local --git https://github.com/AlextheYounga/bonesdeploy.git bonesremote --force
```

Then run the one-time server setup as root:

```sh
sudo bonesremote init
```

This installs a sudoers drop-in at `/etc/sudoers.d/bonesdeploy` so the deploy user can run only the privileged `bonesremote` commands without a password.

## Usage

### Initial Setup

In your project repository:

```sh
bonesdeploy init
```

This will:
1. Create a `.bones/` folder with hooks and deployment script templates
2. Prompt for project name, branch, remote name, host, and port
3. Add `.bones` to `.gitignore`
4. Symlink the `pre-push` hook into `.git/hooks/`
5. Create a local deployment git remote if needed

BonesDeploy assumes opinionated server defaults unless you change them in `.bones/bones.yaml`:

- `port = "22"`
- `live_root = "/var/www/<project_name>"`
- `deploy_root = "/srv/deployments/<project_name>"`
- `deploy_user = "git"`
- `service_user = "<project_name>"`
- `group = "www-data"`

Before first deploy, run server setup:

```sh
bonesdeploy server setup
```

This runs `.bones/server/playbooks/setup.yml` locally with Ansible against your configured remote host.
If `ansible-playbook` is missing, BonesDeploy installs Ansible automatically with `python3 -m pip install --user ansible`.
Template-based projects also scaffold language-specific setup roles (for example: Laravel installs PHP + PHP-FPM, Django installs Python runtime packages, Node templates install global PM2/PNPM tools).
Every setup also installs nginx and provisions a default project vhost that serves `.bones/server/roles/nginx/defaults/index.html.j2` until your first deployment is live.

To customize nginx behavior, edit `.bones/server/nginx/site.conf.j2` and re-run `bonesdeploy server setup`.

When DNS is ready, enable SSL with certbot:

```sh
bonesdeploy server ssl --domain app.example.com --email ops@example.com
```

This obtains a Let's Encrypt certificate and updates the managed nginx site to listen on 443 and redirect HTTP to HTTPS.

### Syncing Configuration

After editing hooks or deployment scripts in `.bones/`:

```sh
bonesdeploy push
```

This rsyncs `.bones/` to the remote bare repo and symlinks the hooks.

### Deploying

Just push to your deployment remote:

```sh
git push production master
```

The hook chain handles the rest:
1. **pre-push** (local) — runs `bonesdeploy doctor --local`
2. **pre-receive** (remote) — runs `bonesremote doctor`, then `sudo bonesremote release stage --config ...`
3. **post-receive** (remote) — runs the deployment pipeline:
    - `bonesremote hooks post-receive --config ...` (checkout + wire shared paths)
    - `bonesremote hooks deploy --config ...` (run deployment scripts + activate/drop-failed)
    - `sudo bonesremote hooks post-deploy --config ...` (permission hardening + pruning)

`pre-push -> pre-receive -> post-receive`

If you set `deploy_on_push = false`, pushes only update refs. Run manual deploy when ready:

```sh
bonesdeploy deploy
```

### Health Checks

```sh
bonesdeploy doctor          # check local + remote
bonesdeploy doctor --local  # check local only
```

## Configuration

`bonesdeploy init` generates `.bones/bones.yaml`:

```yaml
data:
  remote_name: "production"
  project_name: "myproject"
  git_dir: "/home/git/myproject.git"
  live_root: "/var/www/myproject"
  deploy_root: "/srv/deployments/myproject"
  branch: "master"
  deploy_on_push: true

permissions:
  defaults:
    deploy_user: "git"
    service_user: "applications"
    group: "www-data"
    dir_mode: "750"
    file_mode: "640"
  paths:
    - path: "storage"
      mode: "770"
      recursive: true
    - path: "database/database.sqlite"
      mode: "660"
      type: "file"

releases:
  keep: 5
  shared_paths: [".env", "storage"]

# Optional runtime launcher settings (only needed for service/landlock-managed apps).
# runtime:
#   command: ["/usr/bin/node", "server.js"]
#   working_dir: "."
#   writable_paths: []

ssl:
  enabled: false
  domain: ""
  email: ""
```

`host` and `git_dir` are inferred from the deployment remote URL when possible; if parsing fails, init asks only for those missing values.

## Project Structure

```
.bones/
├── bones.yaml           # project configuration
├── hooks.sh             # shared hook functions imported by hook entrypoints
├── deployment/
│   └── 01_*.sh          # deployment scripts (run sequentially)
└── hooks/
    ├── pre-push         # symlinked to .git/hooks/pre-push
    ├── pre-receive
    └── post-receive
```

Hooks are written to `.bones/hooks/` once during init and import shared functions from `.bones/hooks.sh`. After that they belong to you — edit freely. Deployment scripts in `.bones/deployment/` must be numbered (e.g. `01_install_deps.sh`, `02_build.sh`) and are always run in order.

## Good Fit

BonesDeploy is a strong fit when you want:

- direct Linux deploys over SSH
- simple app hosting on one machine at a time
- explicit file ownership and permission hardening
- a lightweight alternative to container-first deployment stacks
- something you can run comfortably on low-cost hosts and Raspberry Pis

BonesDeploy can still deploy Docker-based apps if your deployment scripts call `docker compose`, but Docker is optional rather than the foundation.

## License

MIT
