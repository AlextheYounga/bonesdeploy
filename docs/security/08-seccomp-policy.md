# BonesDeploy Seccomp Policy

## Purpose

Seccomp should be used where practical to reduce risky syscall families.
It is especially useful when the service or worker handles untrusted input.

## Rules

- Prefer systemd syscall filtering for long-running services when it fits the app.
- Use runtime seccomp profiles for containers.
- Treat mount, ptrace, raw IO, kernel module, and BPF access as high-risk unless required.
- Document any exception that needs a dangerous syscall family.

## BonesDeploy Notes

- Seccomp complements the systemd baseline in `docs/security/04-systemd-service-hardening.md`.
- It should be evaluated for app services, build workers, and containerized workers.

## Findings

- untrusted service has no syscall restrictions
- broad namespace creation permissions
- mount-related syscalls available without need
- ptrace or BPF access available without need
