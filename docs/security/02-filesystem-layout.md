# BonesDeploy Filesystem Layout

## Purpose

This document defines the BonesDeploy filesystem layout as it exists today.
The layout is rooted under `/srv/sites/<project>` and separates build state, release code, shared mutable state, and web exposure.

## Current Per-Project Layout

```text
/home/git/<project>.git/
└── bones/
    └── .staged_release

/srv/sites/<project>/
├── build/
│   └── workspace/
├── current -> releases/<release-id>
├── releases/
│   └── <release-id>/
├── shared/
└── ...deploy-managed state...

/srv/sites/<project>/current/<web_root>
```

This matches the current path helpers in `crates/bonesremote/src/release_state.rs`:

- `project_root/build/workspace`
- `project_root/releases/<release-id>`
- `project_root/shared`
- `project_root/current`
- `repo_path/bones/.staged_release`

## Public Exposure Rules

Only `web_root` should be exposed by the web server.
In the current model that is usually:

```text
/srv/sites/<project>/current/<web_root>
```

which points to:

```text
/srv/sites/<project>/current
```

The web server should not expose:

```text
    /srv/sites/<project>/releases
/srv/sites/<project>/shared
/srv/sites/<project>/build
/home/git/<project>.git
```

## Ownership Rules

Each project should have its own Unix service user and group.

Expected ownership and control in the current model:

```text
/srv/sites/<project>                    deploy/root managed
/srv/sites/<project>/build             deploy-owned while staging
/srv/sites/<project>/build/workspace   deploy-owned while building
/srv/sites/<project>/releases          deploy-owned container for releases
/srv/sites/<project>/releases/<id>     deploy-owned while staging, service-owned after activation hardening
/srv/sites/<project>/current           symlink managed by deploy/root helpers
/srv/sites/<project>/shared            service-owned or tightly restricted for service access
/home/git/<project>.git/bones               deploy-owned metadata
```

The deploy user prepares releases, but the active release tree should become `service_user`-owned after activation and post-deploy hardening.

## Permission Rules

Default permissions should prevent cross-project reads and writes.
A sensible baseline for the current layout is:

```text
/srv/sites                             0751 root:root
/srv/sites/<project>                   0750 root:<service-group> or deploy:<service-group>
/srv/sites/<project>/build             0750 deploy:<service-group>
/srv/sites/<project>/releases          0750 deploy:<service-group>
/srv/sites/<project>/shared            0750 <service-user>:<service-group>
/srv/sites/<project>/shared/.env       0640 deploy:<service-group> or root:<service-group>
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
/srv/sites/<project>/shared/storage
/srv/sites/<project>/shared/uploads
/srv/sites/<project>/shared/cache
/srv/sites/<project>/shared/tmp
/run/<project>
/var/log/<project> or another explicit log path
```

Service users should not be able to write to:

```text
/srv/sites/<project>/current
/srv/sites/<project>/releases
/srv/sites/<project>/build
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
<repo_path>/bones/.staged_release
```

This is deployment metadata, not a release directory.
It is the handoff between `bonesremote release stage`, `release wire`, `release activate`, and later hooks.

## Findings

The agent or operator should flag:

- public access to `releases`, `shared`, `build`, or bare git state
- writable active release code
- broad write access under `/srv/sites`
- service users able to write deployment metadata
- secrets stored under the served path
- project layouts that collapse build, release, and shared state into one tree
