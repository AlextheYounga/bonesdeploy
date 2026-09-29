# BonesDeploy methodology

## The model

Three identities. Not two, not five. Three.

| Identity | Owns | Job |
|----------|------|-----|
| `git` (deploy user) | deployment SSH entry point | artifact transport and deployment SSH entry point |
| `<site>` (runtime user) | `shared/`, writable paths, `/run/<site>` | mutates runtime state |
| `root` | system units, config dirs, state, releases | provisions and runs the allowlisted BonesRemote lifecycle |

The runtime user is dedicated per project. Not `www-data`. Not a shared
`applications` user. One project, one user. Isolation is enforced by the
kernel, not by your discipline.

`shared/` is owned by the runtime user. Only the app writes there.
`releases/` are owned by the runtime user while prepare runs, then sealed
`root:<site>` before activation. The setgid bit on `releases/` lets the
runtime group inherit read access without a post-deploy `chown`.

## Permissions are a provisioning-time contract

Not a deployment-time repair. The ownership layout is established once
during `bonesdeploy server setup` and never rewritten by deploy
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
- **No production build fallback.** A local native or Compose build failure leaves
  the server untouched; production never builds application code or images.
- **No `chown -R` on shared state during deploy.** Narrow, local changes
  beat recursive ownership rewrites.

## The build container

Build scripts run locally in the project-pinned `buildpack-deps:bookworm` Docker
image with `cwd=/workspace/source`, targeting `linux/amd64`. The container gets
the exported committed source tree, a scoped local cache at `/workspace/cache`,
fixed public build metadata, and committed public `.env.build` values. It does
*not* get the root `.env`, runtime secrets, `shared/`, `current/`, `releases/`,
production source repositories, host home, SSH agent, credential stores, or Docker socket. Build
input is disposable. Build output becomes the checksummed artifact BonesRemote
receives before promotion.

## Prepare scripts

Prepare scripts run as the runtime user, in a runtime-owned candidate
release, after shared paths are wired, before `current` is repointed.
Migrations, production-secret-based configuration, cache warmups, and
runtime-state work belong here. Application dependency installation and
compilation do not.
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

Native services use systemd `ProtectSystem=strict`, `NoNewPrivileges=yes`,
`PrivateTmp=yes`, AppArmor profiles, and a dedicated runtime user. Compose is
an explicit reduced-guarantee mode: the trusted project file controls container
users, mounts, capabilities, namespaces, networks, and ports through the
rootful Docker daemon. BonesDeploy does not automatically mount the Docker
socket into a service.
