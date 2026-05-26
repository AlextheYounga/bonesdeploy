# BonesDeploy Severity Guide

## Purpose

This section gives a consistent way to classify security findings.

## Critical

- app runs as root without necessity
- service user has sudo, wheel, or docker access
- app can read another app's secrets or database
- Docker socket exposed to apps or containers
- public web access to `.env`, `.git`, backups, or private keys

## High

- all apps share one service user
- AppArmor disabled or services unconfined
- no cgroup limits on untrusted workers
- service user can write release or source directories
- dangerous capabilities granted unnecessarily

## Medium

- weak systemd hardening
- logs readable by unrelated service users
- upload directories allow script execution
- backend binds publicly instead of localhost or a socket

## Low

- layout still uses `/var/www` but is otherwise isolated
- exception is documented but broad
- naming conventions are inconsistent
