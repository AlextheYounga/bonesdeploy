# BonesDeploy Agent Audit Checklist

## Purpose

This section lists the minimum checks an auditing agent should collect before comparing a host to policy.

## Checklist

- System overview: kernel, distro, systemd, mounts
- Users and groups: passwd, group, sudo rules, memberships
- Filesystem layout: `/srv`, project directories, secret files, ownership
- Web exposure: listening ports, nginx or proxy config, public roots
- systemd hardening: unit settings and `systemd-analyze security`
- AppArmor: status and loaded profiles
- Capabilities: bounding and ambient sets
- cgroups: memory, CPU, task limits
- Docker/containers: socket exposure, mounts, security options
- Secrets search: paths and permissions only, not contents

## BonesDeploy Notes

- Prefer comparing the host against the current `deploy_user`, `service_user`, and `public_path` model.
- Report findings with paths and permissions, not secret contents.

## Findings

- missing audit coverage for one of the major control areas
- secret search prints contents instead of metadata
