# BonesDeploy Filesystem Layout

## Purpose

This document defines the BonesDeploy filesystem layout as it exists today.
The layout is rooted under `/srv/deployments/<project>` and separates build state, runtime code, shared mutable state, and public exposure.

## Current Per-Project Layout

```text
/home/git/<project>.git/
└── bones/
    └── .staged_release

/srv/deployments/<project>/
├── build/
│   └── workspace/
├── current -> runtime/<release-id>
├── runtime/
│   └── <release-id>/
├── shared/
└── ...deploy-managed state...

/var/www/<project> -> /srv/deployments/<project>/current
```

This matches the current path helpers in `crates/bonesremote/src/release_state.rs`:

- `deploy_root/build/workspace`
- `deploy_root/runtime/<release-id>`
- `deploy_root/shared`
- `deploy_root/current`
- `git_dir/bones/.staged_release`

## Public Exposure Rules

Only `public_path` should be exposed by the web server.
In the current model that is usually:

```text
/var/www/<project>
```

which points to:

```text
/srv/deployments/<project>/current
```

The web server should not expose:

```text
/srv/deployments/<project>/runtime
/srv/deployments/<project>/shared
/srv/deployments/<project>/build
/home/git/<project>.git
```

## Ownership Rules

Each project should have its own Unix service user and group.

Expected ownership and control in the current model:

```text
/srv/deployments/<project>                    deploy/root managed
/srv/deployments/<project>/build             deploy-owned while staging
/srv/deployments/<project>/build/workspace   deploy-owned while building
/srv/deployments/<project>/runtime           deploy-owned container for releases
/srv/deployments/<project>/runtime/<id>      deploy-owned while staging, service-owned after activation hardening
/srv/deployments/<project>/current           symlink managed by deploy/root helpers
/srv/deployments/<project>/shared            service-owned or tightly restricted for service access
/home/git/<project>.git/bones                deploy-owned metadata
```

The deploy user prepares releases, but the active release tree should become `service_user`-owned after activation and post-deploy hardening.

## Permission Rules

Default permissions should prevent cross-project reads and writes.
A sensible baseline for the current layout is:

```text
/srv/deployments                             0751 root:root
/srv/deployments/<project>                   0750 root:<service-group> or deploy:<service-group>
/srv/deployments/<project>/build             0750 deploy:<service-group>
/srv/deployments/<project>/runtime           0750 deploy:<service-group>
/srv/deployments/<project>/shared            0750 <service-user>:<service-group>
/srv/deployments/<project>/shared/.env       0640 deploy:<service-group> or root:<service-group>
```

Secret files should be stricter when possible:

```text
private keys                                 0600 owner-only
SQLite DBs                                   0600 or 0640 depending on backup/group needs
```

World-readable project directories should be treated as findings unless explicitly justified.

## Writable Directory Rules

Runtime service processes should only be able to write to explicitly approved directories such as:

```text
/srv/deployments/<project>/shared/storage
/srv/deployments/<project>/shared/uploads
/srv/deployments/<project>/shared/cache
/srv/deployments/<project>/shared/tmp
/run/<project>
/var/log/<project> or another explicit log path
```

Service users should not be able to write to:

```text
/srv/deployments/<project>/current
/srv/deployments/<project>/runtime
/srv/deployments/<project>/build
/home/git/<project>.git
/usr
/etc
/root
/home
```

## Shared Path Wiring

BonesDeploy currently wires configured shared paths into the staged build workspace before activation.
That means paths such as `.env` and `storage` are expected to live under `shared/` and appear inside the release tree as symlinks.

This behavior is implemented by `bonesremote release wire` and is part of the current layout.

## Staged Release State

The staged release marker is currently stored at:

```text
<git_dir>/bones/.staged_release
```

This is deployment metadata, not a runtime directory.
It is the handoff between `bonesremote release stage`, `release wire`, `release activate`, and later hooks.

## Findings

The agent or operator should flag:

- public access to `runtime`, `shared`, `build`, or bare git state
- writable active release code
- broad write access under `/srv/deployments`
- service users able to write deployment metadata
- secrets stored under the served path
- project layouts that collapse build, runtime, and shared state into one tree
