# BonesDeploy Severity Guide

## Critical Findings

- application service runs as root without necessity
- service user has `sudo`, `wheel`, or `docker` access
- application can read another project's secrets or database
- Docker socket exposed to application or container
- public web access to `.env`, `.git`, database files, backups, or private keys
- SSH private keys readable by service users
- database, Redis, or Memcached publicly exposed without strong protection
- build scripts run as root with untrusted input

## High Findings

- all projects run as one shared Unix service user
- AppArmor disabled or application services unconfined
- no cgroup limits on untrusted workers
- service user can write release or source-code directories
- broad write access under `/srv/sites/**`
- `NoNewPrivileges=false` or absent for application services
- dangerous capabilities granted unnecessarily
- secrets passed broadly to build jobs

## Medium Findings

- no `ProtectSystem` or weak systemd hardening
- no `PrivateTmp`
- logs readable by unrelated service users
- upload directories allow script execution
- backend binds publicly instead of localhost or a Unix socket
- application has broader read access than necessary

## Low Findings

- layout still uses `/var/www` for the public symlink but is otherwise isolated
- exceptions are documented but broader than necessary
- read access is broader than ideal but non-sensitive
- ownership naming conventions are inconsistent
