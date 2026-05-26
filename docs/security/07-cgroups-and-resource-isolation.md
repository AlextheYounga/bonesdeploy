# BonesDeploy Cgroups and Resource Isolation

## Purpose

Cgroups should prevent one app or worker from exhausting the host.
Resource limits are part of the security model, not just performance tuning.

## Rules

- Each app service should have memory, CPU, and task limits.
- Build workers should also have cgroup limits.
- Untrusted or bursty processes should never run with unlimited resources.
- Shared service cgroups across unrelated apps should be avoided.

## BonesDeploy Notes

- Apply limits to the service managed after `bonesremote release activate`.
- Build and deployment jobs should use explicit limits in the same spirit as the app service.
- Keep the limits documented so `bonesdeploy doctor` can compare intent against reality.

## Findings

- no `TasksMax` or very high task limit
- no memory limit on an app service
- no CPU control for untrusted workers
- multiple apps share one service cgroup unnecessarily
