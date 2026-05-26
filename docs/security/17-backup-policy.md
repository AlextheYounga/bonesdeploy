# BonesDeploy Backup Policy

## Purpose

Backups need broad read access in some cases, but that makes them high-value targets.

## Rules

- Backups should not live inside public paths.
- Backup credentials should not be readable by service users.
- Backup archives should not be world-readable.
- Secrets in backups should be encrypted or otherwise protected as required by policy.

## BonesDeploy Notes

- Backups should not interfere with the active release or `shared/` runtime state.
- Treat backup jobs as privileged infrastructure, not app runtime.

## Findings

- backup archives under public directories
- backup credentials readable by service users
- backups exposed to unrelated projects
