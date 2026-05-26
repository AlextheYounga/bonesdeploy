# BonesDeploy Systemd Service Hardening

## Purpose

This section describes how BonesDeploy services should be hardened.
The service should run with the least filesystem and privilege surface that still works.

## Baseline

```ini
[Service]
User=<service-user>
Group=<service-group>
WorkingDirectory=/srv/deployments/<project>/current
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/srv/deployments/<project>/shared /srv/deployments/<project>/runtime /run/<project>
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
```

## BonesDeploy Guidance

- Keep the active release read-only except for explicit writable paths.
- Put writable runtime state in `shared/`, `cache/`, `tmp/`, or `/run/<project>`.
- Prefer narrow exceptions over disabling protections globally.
- If `ProtectSystem=strict` breaks the service, fix the path list, not the protection model.

## Findings

- service runs as root
- `NoNewPrivileges` missing
- `ProtectSystem` disabled
- `ProtectHome` missing
- broad `ReadWritePaths`
- capabilities granted without justification
