# BonesDeploy Database and Local Service Policy

## Purpose

Databases and local daemons should be isolated per project and not exposed broadly.

## Rules

- SQLite files should be app-specific and permissioned tightly.
- Network databases should bind to localhost or a private interface unless external access is intentional.
- Each app should have distinct credentials.
- Redis and Memcached should not be publicly exposed.

## BonesDeploy Notes

- SQLite databases usually belong under the project's protected shared state, not under `public_path`.
- Backup and deploy tooling should be able to reach data only when explicitly required.

## Findings

- SQLite database readable by unrelated service users
- database bound publicly
- multiple apps share one DB user
- Redis or Memcached exposed on a public interface
