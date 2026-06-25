# BonesDeploy and BonesRemote Concerns

This document defines the concerns that belong in the Rust tools:

- `bonesdeploy`: the local CLI used by the operator.
- `bonesremote`: the remote release orchestrator and privileged mediator.

It intentionally excludes concerns handled by `bonesinfra`, such as OS package installation, user creation, directory provisioning, systemd unit rendering, nginx configuration, AppArmor profile installation, firewall setup, Podman installation, and other host bootstrap/runtime infrastructure work.

The target security model is:

```text
git = ingress only
foo = one per-site user/group, no login, no sudo
podman build = temporary build environment
shared/ = persistent runtime state owned by foo
releases/ = promoted artifacts sealed as root:foo
bonesremote/root = privileged mediator
```

## Shared Principles

`bonesdeploy` and `bonesremote` should enforce the deployment lifecycle, not provision the machine.

Their responsibilities are:

- trust only root-owned registry data for privileged decisions.
- treat repo-owned config and build output as untrusted input.
- keep `git` as a trigger, not a deploy identity.
- keep builds disposable and isolated from runtime state.
- harden build artifacts before promoting them into releases.
- run runtime preparation as the per-site user.
- activate releases only after preparation succeeds.
- avoid deployment-time ownership repair except for artifact sealing performed by `bonesremote`.

Their responsibilities are not:

- creating Unix users or groups.
- installing Podman or language runtimes.
- creating base directories under `/srv`, `/run`, `/etc`, or `/var/lib`.
- rendering or installing nginx, systemd, or AppArmor files.
- installing sudoers policy; `bonesremote` defines the commands it needs, and `bonesinfra` installs the sudoers rules for those commands.
- changing firewall rules.
- deciding host-level package policy.

## `bonesdeploy` Concerns

`bonesdeploy` is the local operator interface. It should prepare project-owned configuration, invoke remote commands, and synchronize project deployment assets.

### Local Project Configuration

`bonesdeploy` owns local project setup concerns:

- initialize `.bones/` project scaffolding.
- write and update `.bones/bones.toml`.
- write and update `.bones/runtime.toml` when selecting a runtime template.
- install or repair local Git hooks that call back into `bonesdeploy` or remote hooks.
- keep local config values coherent enough to call remote commands.

`bonesdeploy` should not treat local config as authority for privileged remote operations. Local config is operator input; the remote root-owned site registry is authority.

### Operator Commands

`bonesdeploy` owns user-facing command orchestration:

- `init` for local project scaffolding.
- `doctor` for local checks and remote reachability checks.
- `push` and `pull` for syncing `.bones/` assets to and from the remote repo area.
- `deploy` for explicitly asking the remote host to deploy the configured site.
- `rollback` for explicitly asking the remote host to roll back the configured site.
- `secrets` commands for local encrypted secret editing and remote secret delivery.
- `config`, `manage`, and `version` commands.

When a command needs privileged remote behavior, `bonesdeploy` should call `bonesremote` through SSH and let `bonesremote` validate against the root-owned registry.

### Remote Invocation

`bonesdeploy` may initiate remote actions, but it should not implement their privileged logic locally.

It may:

- SSH into the remote host.
- pass the intended site identifier or registry path.
- pass the intended revision when deploying a specific commit.
- stream or upload project-owned files where required.
- report remote command failures clearly.

It should not:

- compute trusted remote ownership decisions from local `.bones/*.toml`.
- directly `chown`, `chmod`, or mutate privileged paths over SSH.
- restart services directly.
- flip `current` directly.
- write release artifacts directly.

### Secrets Delivery

`bonesdeploy` owns local encrypted secret management and transport.

It may:

- manage encrypted secret files under `.bones/secrets/`.
- decrypt secrets locally for editing.
- upload secret material to a remote command or staging location.

It should not decide final ownership, group, mode, or destination from repo-owned config. Final placement must be mediated by `bonesremote` using the root-owned registry.

## `bonesremote` Concerns

`bonesremote` is the remote deployment state machine. It owns the secure transition from a Git revision to an activated release.

### Registry-Based Authority

Privileged `bonesremote` commands must read authority from the root-owned site registry, for example:

```text
/etc/bonesdeploy/sites/foo.toml
```

The registry is the trusted source for:

- site name.
- repository path.
- site root.
- shared path.
- releases path.
- current symlink path.
- runtime user and group.
- service names.
- framework/runtime identifier.
- allowed deploy paths.

`bonesremote` may read repo-owned `.bones/bones.toml` and `.bones/runtime.toml` only as untrusted deployment preferences. Those files must not authorize privileged paths, users, groups, or service names.

### Deploy Orchestration

`bonesremote deploy` owns the remote deployment sequence:

```text
validate registry
resolve revision
export source
run disposable build
promotion hardening
wire shared paths
run runtime prepare as foo
activate current as root
restart service as root
prune old releases
clean temporary state
```

On failure, `bonesremote` should clean temporary build state and incomplete release state without weakening permissions or mutating the active release.

### Source Export

`bonesremote` owns converting a registered Git revision into a build input.

It should:

- validate the requested repo and revision against the root-owned registry.
- export source into a temporary build context.
- avoid giving the build environment direct access to the bare repo.
- avoid exposing `.git` history unless explicitly required later.

The build context should be disposable and deleted after success or failure.

### Disposable Build Execution

`bonesremote` owns invoking the build, but not provisioning the container runtime.

It should:

- run builds in a temporary Podman container.
- mount only source, output, caches, and temporary directories required for the build.
- avoid mounting `shared/`, `.env`, SQLite databases, `current`, `releases/`, `/etc/bonesdeploy`, `/root`, or the bare repo.
- treat build scripts as project-controlled and untrusted.
- require the build to produce an artifact directory.

It should not install Podman, configure registries, install language packages on the host, or create cache directories. Those are `bonesinfra` concerns.

### Promotion Hardening

`bonesremote` owns promotion hardening: turning untrusted build output into a sealed release.

Podman output is not a release. It is raw build output produced by project-controlled code. Before activation, `bonesremote` must validate it, reject unsafe filesystem entries, and normalize the accepted files into a release tree.

`bonesremote` should:

- copy accepted artifact output into a new release directory.
- set release ownership to `root:foo`.
- set directory modes to `0750`.
- set regular file modes to `0640`.
- preserve executable bits only where appropriate, using `0750`.
- clear setuid and setgid bits.
- reject device files, FIFOs, sockets, and unsafe special files.
- reject or safely rewrite symlinks that are absolute or escape the release/shared boundary.

This is a core `bonesremote` concern because root is turning output generated by project-controlled build code into host-owned release files.

### Shared Path Wiring

`bonesremote` owns wiring runtime-mutable paths into a sealed release before runtime preparation.

For Laravel v1, this includes:

```text
release/.env                     -> shared/.env
release/storage                  -> shared/storage
release/bootstrap/cache          -> shared/bootstrap/cache
release/database/database.sqlite -> shared/database/database.sqlite
release/public/storage           -> ../storage/app/public
```

The release remains sealed as `root:foo`. The symlink targets under `shared/` remain mutable state owned by `foo:foo`.

By default, `bonesinfra` should create the shared files and directories a runtime needs, such as `shared/storage/` or `shared/database/database.sqlite`.

`bonesremote` should normally only link those existing shared paths into a release. If a deploy-time command ever creates a missing shared path, it must use the trusted registry path for `shared/`, not a path supplied by repo-owned config.

### Runtime Prepare

`bonesremote` owns running post-build runtime preparation commands as the per-site user.

For Laravel, this phase may include:

```sh
php artisan migrate --force
php artisan optimize
php artisan package:discover --ansi
php artisan queue:restart
```

This phase runs as `foo`, not root and not `git`.

This phase may see `.env`, SQLite, storage, and framework caches through `shared/` symlinks.

This phase must complete before activation. If it fails, the release must not become current.

Migrations can mutate the database before activation. `bonesremote` should make that ordering explicit in logs and documentation: rollback is code rollback, not database rollback.

### Activation and Service Control

`bonesremote` owns privileged activation and service control.

It should:

- atomically update `current` to the prepared release.
- restart or reload only registry-approved services.
- support rollback by repointing `current` to a previous sealed release.
- refuse service names or paths supplied only by repo-owned config.

`git` should reach this behavior only through a narrow sudo rule that permits the intended `bonesremote` command for registered sites.

### Release State and Cleanup

`bonesremote` owns deployment state tracking on the remote host.

It should:

- track the staged release being prepared.
- distinguish temporary build contexts from promoted releases.
- remove failed temporary build contexts.
- remove incomplete releases that never activated.
- prune old sealed releases according to registry or project policy.
- leave the active release untouched on failure.

## Hook Model Concerns

Git hooks should be thin triggers.

The remote `post-receive` hook should:

- identify the pushed repo and revision.
- exit early when the configured deployment ref was not updated.
- call a narrow `sudo bonesremote deploy ...` command for the registered site.

It should not:

- check out code.
- run build commands.
- write releases.
- write shared state.
- restart services.
- make ownership decisions.

## Explicitly Out of Scope for These Tools

The following concerns belong to `bonesinfra`, not `bonesdeploy` or `bonesremote`:

- creating the `git` user.
- creating the per-site `foo` user and group.
- removing `git` from site groups.
- creating `/srv/git`, `/srv/sites/foo`, `/srv/sites/foo/shared`, `/srv/sites/foo/releases`, `/run/foo`, or cache roots.
- setting base ownership and modes for provisioned directories.
- installing `bonesremote` onto the host.
- installing Podman.
- configuring Podman policy, storage, or registries.
- installing PHP, Composer, Node, npm, database clients, or framework dependencies on the host.
- rendering and installing systemd units.
- rendering and installing nginx config.
- rendering and installing AppArmor profiles.
- installing sudoers policy. `bonesinfra` installs it from the narrow command contract required by `bonesremote`.
- obtaining TLS certificates.
- configuring firewall rules.
- applying OS hardening.

`bonesdeploy` and `bonesremote` may validate that these prerequisites exist and report actionable errors, but they should not silently provision or repair them during deployment.

## Minimal v1 Boundary

The minimum useful v1 split is:

- `bonesinfra` provisions the host, users, directories, services, Podman, sudoers policy, and the registry parent directory.
- `bonesremote` writes and reads the site registry file used for privileged deployment decisions.
- `bonesdeploy` prepares local project config and asks the remote to deploy.
- `git` receives pushes and triggers `bonesremote` only.
- `bonesremote` validates registry data, builds in disposable Podman, hardens artifacts into sealed releases, wires shared paths, runs runtime prepare as `foo`, activates as root, restarts services, and cleans up.

The simplest rule is:

```text
bonesinfra creates the stage.
bonesdeploy asks for a deployment.
bonesremote performs the deployment safely.
```
