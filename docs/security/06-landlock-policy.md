# BonesDeploy Landlock Policy

## Purpose

Landlock should be used for child jobs that voluntarily reduce their own filesystem access.
It is best suited to builds, dependency installs, plugin execution, and deployment hooks.

## Rules

- Use Landlock for risky child processes where the job knows its exact workspace.
- Do not treat Landlock as a replacement for AppArmor or systemd hardening.
- Restrict each job to the smallest set of paths it actually needs.
- Deny access to unrelated projects, deploy keys, and secrets by default.

## BonesDeploy Notes

- Build and deploy worker steps should sandbox before running untrusted code.
- The policy should match the paths used by `bonesremote release-stage`, `release-wire`, and build scripts.
- Jobs should only read the active release tree when they genuinely need it.

## Findings

- risky job runs without any sandbox
- build script can read all project directories
- job can write outside its workspace
- job can access SSH keys or `.env` files it does not need
