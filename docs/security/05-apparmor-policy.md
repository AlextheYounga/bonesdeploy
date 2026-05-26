# BonesDeploy AppArmor Policy

## Purpose

AppArmor should be the default static confinement layer for app services and risky workers.
It should narrow read, write, exec, and process access beyond what systemd alone can express.

## Rules

- AppArmor should be installed, enabled, and enforcing where supported.
- Each long-running app should have a dedicated profile or a documented exception.
- Profiles should restrict access to unrelated projects, `/root`, `/home`, and deploy secrets.
- Broad unconfined execution should be treated as a finding.

## BonesDeploy Notes

- Use profiles alongside the `deploy_user` / `service_user` split defined in `docs/PROJECT.md`.
- AppArmor should cover the active release tree after `bonesremote release activate`.
- Deployment workers should use tighter profiles for build and hook execution where practical.

## Findings

- app service runs unconfined
- profile is in complain mode instead of enforce mode
- service can read other projects' release trees or secrets
- service can write outside approved writable paths
