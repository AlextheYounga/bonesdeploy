# Ansible Performance Refactor

## Goal

Reduce the runtime of `bonesdeploy init --setup-remote`, `bonesdeploy remote setup`, and `bonesdeploy remote ssl` without changing their current behavior.

`init` and `remote ssl` are currently working. This refactor is strictly about performance and ownership boundaries, not feature redesign.

## Current Problem

The current implementation uses Ansible as the primary execution engine for both host bootstrap and project-specific reconciliation.

That makes narrow operations expensive. In particular, `remote ssl` runs through the remote setup Ansible runner with tags instead of executing a focused SSL operation.

Current flow:

```text
bonesdeploy init --setup-remote
  -> remote_setup::run()
  -> ensure local ansible-playbook exists
  -> ensure remote Python exists
  -> run .bones/remote/playbooks/setup.yml

bonesdeploy remote ssl
  -> ensure local ansible-playbook exists
  -> ensure remote Python exists
  -> run .bones/remote/playbooks/setup.yml --tags nginx,ssl
```

This causes repeated general-purpose provisioning overhead for operations that should be deterministic and project-scoped.

## Product Constraints

- Debian and Ubuntu are the supported server platforms.
- No distro abstraction layer is needed.
- Current functionality must be preserved.
- Existing generated paths and scaffold behavior must remain stable unless intentionally changed in a separate feature.
- The refactor should prioritize clear ownership and fast idempotent operations.

## Target Ownership Boundary

```text
bonesdeploy
  local orchestration, config editing, .bones sync, SSH command execution

Ansible
  one-time or rare host bootstrap only

bonesremote
  fast project-specific reconciliation on the server
```

Ansible should not be the default executor for project lifecycle work.

## What Ansible Should Keep

Ansible remains acceptable for host bootstrap work that is naturally slow and package-manager dependent:

- create deploy and service users
- install base OS packages
- install runtime packages
- install nginx
- install AppArmor packages and enable the service
- install certbot during setup
- install or place `bonesremote`

Package installation should stay in Ansible for the first refactor pass.

## What Moves To `bonesremote`

Project-specific reconciliation should move into `bonesremote` because it is deterministic, idempotent, and should be fast:

- create bare repo parent
- initialize the bare git repo
- create the remote repo `bones/` directory
- ensure project root parent traversal permissions
- create placeholder release files
- point `current` at the placeholder release when needed
- render per-site nginx config
- render per-site systemd service
- render AppArmor profile
- load and enforce AppArmor profile
- install or update sudoers drop-in currently handled by `bonesremote init`
- run final server-side validation currently covered by `bonesremote doctor`

## Proposed Commands

Add a project reconciliation command:

```bash
bonesremote provision apply --config /home/git/<project>.git/bones/bones.yaml
```

Add a focused SSL command:

```bash
bonesremote ssl enable \
  --config /home/git/<project>.git/bones/bones.yaml \
  --domain app.example.com \
  --email ops@example.com
```

Both commands should require root when they touch privileged paths or services.

## Target Flow

### `bonesdeploy remote setup`

```text
bonesdeploy remote setup
  -> ensure local bootstrap requirements
  -> run Ansible host bootstrap
  -> sync .bones to remote repo
  -> ssh: sudo bonesremote provision apply --config <remote bones.yaml>
```

The Ansible playbook should no longer own project-specific nginx, AppArmor, bare repo, placeholder release, or doctor work once those pieces exist in `bonesremote provision apply`.

### `bonesdeploy init --setup-remote`

```text
bonesdeploy init --setup-remote
  -> scaffold local .bones
  -> save .bones/bones.yaml
  -> run bonesdeploy remote setup
  -> sync .bones
```

Behavior should remain the same from the user's perspective.

### `bonesdeploy remote ssl`

```text
bonesdeploy remote ssl --domain app.example.com --email ops@example.com
  -> validate and save SSL inputs locally
  -> sync .bones
  -> ssh: sudo bonesremote ssl enable --config <remote bones.yaml> --domain <domain> --email <email>
  -> mark ssl.enabled=true locally
  -> sync .bones again
```

`remote ssl` should not invoke Ansible after this refactor.

## `bonesremote provision apply` Responsibilities

`bonesremote provision apply` should be idempotent. Re-running it should converge the project state without doing unnecessary work.

Expected responsibilities:

```text
read bones.yaml
derive DeploymentPaths
ensure sudoers drop-in
ensure repo parent exists
ensure bare repo exists
ensure repo bones dir exists
ensure project root parent is traversable
ensure placeholder release exists
ensure placeholder index exists
ensure current symlink exists
render repo-local per-site nginx config
render systemd unit for per-site nginx
systemctl daemon-reload when unit changed
enable/start per-site nginx service
render AppArmor profile
load/enforce AppArmor profile when changed
run validation equivalent to current doctor checks
```

The command may use Debian/Ubuntu-specific paths and commands directly.

## `bonesremote ssl enable` Responsibilities

`bonesremote ssl enable` should only perform SSL enablement for one project.

Expected responsibilities:

```text
read bones.yaml
validate domain and email
verify certbot is installed
render HTTP challenge nginx config
nginx -t
reload nginx
run certbot certonly --webroot --keep-until-expiring
render HTTPS nginx config
nginx -t
reload nginx
```

Certbot should be installed during `remote setup`. If it is missing during `ssl enable`, fail with a clear message telling the user to run `bonesdeploy remote setup`.

## Ansible Changes

After `bonesremote provision apply` exists, trim the setup playbook to bootstrap only.

Keep Ansible tasks for:

- users
- base packages
- runtime packages
- nginx package
- AppArmor package and service
- certbot package
- `bonesremote` installation

Remove or move Ansible tasks for:

- bare git repo setup
- repo `bones/` directory setup
- placeholder release setup
- nginx config rendering
- per-site systemd unit rendering
- AppArmor profile rendering/loading/enforcing
- post-task `bonesremote doctor`

The playbook can end by invoking `bonesremote provision apply` until `bonesdeploy remote setup` owns that SSH call directly.

## Performance Expectations

Expected improvements:

- `remote ssl` avoids Ansible completely and should only take nginx plus certbot time.
- repeated `remote setup` runs avoid project reconciliation through Ansible modules.
- project-specific work becomes direct Rust filesystem/process operations on the remote host.
- future optimization can focus on host bootstrap only.

## Follow-Up Optimization

After project reconciliation moves to `bonesremote`, reassess setup runtime.

If setup is still too slow, optimize bootstrap separately:

- combine apt package installs
- avoid repeated apt cache refreshes
- stop compiling `bonesremote` on the server with `cargo install --git`
- install `bonesremote` from a release artifact or upload a locally built binary

These are separate from the ownership refactor and should not block moving project-specific work out of Ansible.

## Non-Goals

- Do not change user-facing behavior of `init`, `remote setup`, or `remote ssl`.
- Do not add non-Debian or non-Ubuntu support.
- Do not introduce a generic package manager abstraction.
- Do not remove Ansible completely in the first pass.
- Do not change deployment path semantics as part of this refactor.

## Acceptance Criteria

- `bonesdeploy init --setup-remote` still provisions a working project.
- `bonesdeploy remote setup` still provisions the same server and project state as today.
- `bonesdeploy remote ssl` still obtains and enables SSL as today.
- `remote ssl` no longer invokes Ansible.
- project-specific reconciliation is implemented in `bonesremote`.
- repeated setup and SSL runs are materially faster than the current roughly 30-minute runtime.
- existing tests continue to pass, with new regression coverage for the moved behavior.
