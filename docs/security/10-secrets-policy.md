# BonesDeploy Secrets Policy

## Purpose

Secrets should stay in protected, app-specific locations and only be readable by the identities that need them.

## Rules

- Keep secrets out of Git, logs, public paths, and shared build caches.
- Prefer `shared/.env`, `EnvironmentFile`, or another strict per-app secret location.
- Do not hand every build job every secret.
- Service users should not be able to read deployment SSH keys.

## BonesDeploy Notes

- This policy aligns with the `deploy_user` / `service_user` split in `docs/PROJECT.md`.
- `bonesdeploy init` and remote setup should keep secret paths separate from the served path.
- Post-deploy hardening should never broaden secret readability.

## Findings

- `.env` world-readable
- secrets stored in a web-served directory
- secrets copied into release artifacts
- SSH keys readable by service users
- build jobs receive secrets they do not need
