# BonesDeploy methodology

## The model

Three identities. Not two, not five. Three.

| Identity | Owns | Job |
|----------|------|-----|
| `git` (deploy user) | bare repo | ingress only |
| `<site>` (runtime user) | `shared/`, writable paths, `/run/<site>` | mutates runtime state |
| `root` | system units, config dirs, users, releases | provisions, deploys, restarts |

The runtime user is dedicated per project. Not `www-data`. Not a shared
`applications` user. One project, one user. Isolation is enforced by the
kernel, not by your discipline.

`shared/` is owned by the runtime user. Only the app writes there.
`releases/` are owned by the runtime user while prepare runs, then sealed
`root:<site>` before activation. The setgid bit on `releases/` lets the
runtime group inherit read access without a post-deploy `chown`.

## Permissions are a provisioning-time contract

Not a deployment-time repair. The ownership layout is established once
during `bonesdeploy remote bootstrap` and never rewritten by deploy
commands. If you find yourself wanting to `chmod` during a deploy, you are
fixing the wrong thing. Fix the provisioning.

## Just-in-time mutations

A mutation happens at the last responsible moment — immediately before
the system would fail if it didn't. Not earlier. Not "while we're here."

- pre-deploy steps validate and prepare *isolated* state. They don't touch live state.
- build steps run on isolated workspace state.
- activation happens at activation time.
- permission hardening happens *after* a successful activation, not before.
- a failed deploy leaves no broadened access, no half-applied live mutations.

If a mutation can be delayed safely, it is delayed. If a mutation affects
live state, it is justified by an immediate need. This is not aesthetic
preference. It is the difference between a deploy that fails clean and a
deploy that fails into a security incident.

## What we don't do

- **No shared groups with 660/770 everywhere.** The "let the deploy user
  read everything" pattern is a tangle of logic traps. We use dedicated
  users and the setgid bit instead.
- **No ACLs.** Opaque. Unreadable. We use ordinary Unix ownership.
- **No inotify systems.** Cumbersome, fragile, invisible. We use systemd
  services and explicit restart.
- **No silent Podman reset.** A damaged rootless Podman namespace is
  reported before any release state is created. Resetting it would stop
  the build user's containers, so we don't do it behind your back.
- **No `chown -R` on shared state during deploy.** Narrow, local changes
  beat recursive ownership rewrites.

## The build container

Build scripts run in `buildpack-deps:bookworm` with `cwd=/workspace/source`.
The container gets the exported source tree and a private persistent build
cache at `/workspace/cache`. It does *not* get `.env`, `shared/`,
`current/`, `releases/`, the bare repo, or host `bonesremote` control-plane
files. Build input is disposable. Build output is what gets promoted.

`bonesremote` runs each script through the build user's systemd user
manager with `systemd-run --machine=<site>-build@ --user`, not `runuser`.
The long-lived build container is a transient user service that tracks
Podman's monitor process; each script streams its output through
foreground `podman exec`.

## Prepare scripts

Prepare scripts run as the runtime user, in a runtime-owned candidate
release, after shared paths are wired, before `current` is repointed.
Migrations, cache warmups, runtime-state work — this is the place.
`bonesremote` opens the root-owned `functions.sh` and the script and
streams both as one stdin input to the runtime-user shell. The runtime
user never gets filesystem access to the deployment bundle.

## The lock

`bonesremote` holds one OS-backed deployment lock per site. Deploys,
cancellations, and site imports all take it. Nothing stages or overwrites
state while a release is building, preparing, or interrupted. The lock
lives outside the replaceable site dataset, so replacing the dataset
doesn't replace the lock.

## Service restart

`bonesremote service restart` restarts `<project>.target`, which restarts
every registered site service. It's the only `bonesremote` command that
needs root. `bonesinfra` owns site service membership. `bonesremote`
restarts exactly `<project>.target` for deploy and rollback — nothing
more, nothing less.

## Runtime sandboxing

Systemd `ProtectSystem=strict`, `NoNewPrivileges=yes`, `PrivateTmp=yes`,
and AppArmor profiles. Per-project services run as the dedicated runtime
user. Blast radius is bounded by the kernel, not by your hope.
