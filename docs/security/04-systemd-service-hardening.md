# BonesDeploy Systemd Service Hardening

## Purpose

This document describes how BonesDeploy-managed services should be hardened under systemd.
The service should run with the least filesystem and privilege surface that still allows the application to function.

Each long-running app should be managed by a dedicated systemd service unless there is a specific reason not to.

## Baseline Service Shape

Each app service should use as many of the following settings as practical:

```ini
[Service]
User=<service-user>
Group=<service-group>
WorkingDirectory=/srv/deployments/<project>/current
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/srv/deployments/<project>/shared /run/<project>
CapabilityBoundingSet=
AmbientCapabilities=
RestrictSUIDSGID=true
LockPersonality=true
MemoryDenyWriteExecute=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true
RestrictRealtime=true
SystemCallArchitectures=native
TasksMax=256
MemoryMax=<appropriate-limit>
CPUQuota=<appropriate-limit>
```

## `ProtectSystem=strict`

`ProtectSystem=strict` should be preferred where possible.
It makes most of the filesystem read-only to the service and re-opens only the paths listed in `ReadWritePaths`.

In this policy, `ProtectSystem=strict` is the baseline model, not an invitation to add user-facing per-project knobs.
If one service genuinely needs an extra writable path, that should be handled as a host-side service-unit exception for that one service, not as a broad weakening of the default model.

Disabling `ProtectSystem` entirely should be treated as a significant regression unless there is a strong documented reason.

## `ProtectHome=true`

Services should not normally read `/home`, `/root`, or user home directories.
If a project needs home-directory access, that should be treated as suspicious unless explicitly justified.

This matters in BonesDeploy because deployment SSH keys and operator state commonly live under home directories.

## `NoNewPrivileges=true`

Services should use:

```ini
NoNewPrivileges=true
```

This prevents gaining additional privilege through exec transitions such as setuid binaries.

## Capability Policy

Most app services should have no Linux capabilities:

```ini
CapabilityBoundingSet=
AmbientCapabilities=
```

If a service needs a capability, the specific capability should be documented.

High-risk capabilities that should almost never be granted to app services:

```text
CAP_SYS_ADMIN
CAP_NET_ADMIN
CAP_SYS_MODULE
CAP_SYS_PTRACE
CAP_DAC_OVERRIDE
CAP_DAC_READ_SEARCH
CAP_SETUID
CAP_SETGID
CAP_CHOWN
CAP_FOWNER
CAP_SYS_RAWIO
CAP_MKNOD
```

## Resource Limits

Each app should have cgroup-backed resource limits through systemd.
Recommended controls:

```ini
MemoryMax=
MemoryHigh=
CPUQuota=
TasksMax=
IOWeight=
```

The exact values depend on the app, but unlimited memory and unlimited task creation should be treated as findings.

## BonesDeploy Expectations

Within the current BonesDeploy layout:

- `WorkingDirectory` should point at `project_root/current`
- the active release should be read-only to the service after post-deploy hardening
- writable runtime state should live under `shared/` or `/run/<project>`
- build state should not be writable by the long-lived application service

## Findings

The agent or operator should flag:

- service runs as root
- `NoNewPrivileges` missing
- `ProtectSystem` disabled
- `ProtectHome` missing
- broad `ReadWritePaths`
- capabilities granted without justification
- no memory or task limits
- build directories writable by the application service
