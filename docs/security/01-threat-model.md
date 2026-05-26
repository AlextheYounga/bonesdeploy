# BonesDeploy Threat Model

## Purpose

This is the security target for BonesDeploy-managed projects on a single Linux host.
The goal is strong practical isolation between projects controlled by the same operator.

## Core Goals

- A compromised app should not reach unrelated projects.
- A compromised app should not gain sudo/root.
- A compromised app should not read deploy credentials or server-wide secrets.
- A failed deploy should not leave broadened access behind.

## BonesDeploy Model

- `deploy_user` handles deployment orchestration.
- `service_user` runs the application.
- `public_path` is the user-facing path.
- `deploy_root` stores staging, releases, and deployment metadata.

## Assumed Threats

- RCE in the app
- Malicious dependency code
- Unsafe build scripts
- Secret leakage from config or logs
- SSRF or unsafe internal access
- Runtime tampering with deployed files

## Non-Goals

- Hostile multi-tenant isolation
- Protection from kernel bugs
- Protection from a malicious root user
- Protection from a malicious hosting provider

## What Good Looks Like

- deploy user stages work just in time
- service user only sees the active release and approved writable paths
- active release ownership is hardened back to the service user
- `public_path` is a stable symlink to the active release
