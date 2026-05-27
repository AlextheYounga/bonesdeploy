# BonesDeploy Deployment and Build Worker Policy

## Purpose

Deployment and build work should be separated from runtime service execution.
The deploy identity can manage releases, but it should not become the runtime identity.

## Rules

- Deployment orchestration should run as `deploy_user`, not root.
- Release work should use release directories and atomic symlink flips.
- Build scripts should run in `deploy_root/build/workspace`, not in `public_path`.
- Build jobs should use the minimum secrets, permissions, and network access required.

## BonesDeploy Notes

- This policy matches `bonesremote release-stage`, `release-wire`, `release-activate`, and `hooks-post-deploy`.
- `bonesdeploy remote-setup` should create the layout needed for this separation.
- The active release tree should become service-owned after activation and hardening.

## Findings

- build script runs as root
- build workspace is shared across projects
- public path is writable during build
- deployment SSH keys are readable by the service user
