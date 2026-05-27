# BonesDeploy Backup Policy

## Purpose

Backup jobs often need broad read access, which makes them high-value and high-privilege.

## Backup Access

Backup users or services should be treated as privileged infrastructure.
Backup credentials should not be readable by service users.

## Backup Storage

Backups should not be stored inside `public_path` or under application paths readable by unrelated service users.

## Findings

The agent or operator should flag:

- `.tar`, `.zip`, `.sql`, `.sqlite`, `.bak`, or `.dump` files under public directories
- backups readable by unrelated service users
- backups containing secrets without encryption
- backup credentials available to runtime applications
