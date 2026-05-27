# BonesDeploy Filesystem Layout

## Purpose

This section defines the current BonesDeploy filesystem shape.
It keeps release code, runtime state, and public exposure separated.

## Preferred Layout

```text
/srv/deployments/<project>/
  build/
    workspace/
  current -> runtime/<release-id>
  runtime/
    <release-id>/
  shared/
  bones/
    .staged_release

/var/www/<project> -> /srv/deployments/<project>/current
```

## Rules

- `/srv/deployments` is the deployment root.
- `public_path` is the only user-facing path.
- `current` is the deployment-managed symlink.
- `runtime/<release-id>` is immutable after activation.
- `shared/` holds explicit runtime state like `.env` and storage.
- staged release state lives under the bare repo in `git_dir/bones/.staged_release`.

## Ownership Model

- `deploy_user` owns staging and release creation.
- `service_user` owns the active release after hardening.
- shared writable paths are owned by `service_user`.
- `current` is managed by deploy/root helper commands only.

## Writable Paths

Allowed runtime writes should be narrow and explicit:

- `shared/storage`
- `shared/uploads`
- `cache`
- `tmp`
- `/run/<project>`
- `logs` when needed

## Findings

- public access to `current`, `shared`, or `runtime`
- writable release code
- broad write access under `/srv/deployments`
- secrets stored in public paths
