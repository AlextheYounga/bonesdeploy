# BonesDeploy Seccomp Policy

## Purpose

Where practical, services should use seccomp syscall filtering to reduce unnecessary syscall surface.

## Recommended Usage

For systemd services, consider:

```ini
SystemCallFilter=
SystemCallArchitectures=native
```

For containers, use runtime seccomp profiles.

Seccomp is especially useful when a service or worker handles untrusted input or executes child processes with complex dependency trees.

## High-Risk Syscall Families

The agent or operator should identify whether services have access to high-risk syscall families such as:

- mounting filesystems
- kernel module operations
- raw IO
- ptrace
- keyring abuse
- namespace creation when not required
- BPF operations when not required

Systemd hardening options may cover some of these more safely than maintaining a large custom syscall list by hand.

## Findings

The agent or operator should flag:

- services with no syscall restrictions when they run untrusted code
- broad permission for namespace creation in untrusted workers
- broad permission for mount-related syscalls
- `ptrace` available without need
- BPF-related permissions available without need
