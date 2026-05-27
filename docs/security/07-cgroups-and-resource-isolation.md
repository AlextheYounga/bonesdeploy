# BonesDeploy Cgroups and Resource Isolation

## Purpose

Cgroups should prevent one project or worker from exhausting the host.
Resource limits are part of the security model, not just performance tuning.

## Required Controls

Every application service should have resource limits appropriate to its role.
Minimum recommended controls:

```ini
TasksMax=256
MemoryMax=<app-specific>
CPUQuota=<app-specific>
```

For heavier services, use documented higher values.

## Security Goals

Cgroups should help prevent:

- fork bombs
- unlimited memory growth
- one project monopolizing CPU
- excessive process or thread creation
- some forms of IO abuse

## Scope

Resource isolation should cover:

- long-lived application services
- deploy workers that execute untrusted or bursty jobs
- other background workers that can create many subprocesses or consume large memory spikes

## Findings

The agent or operator should flag:

- no `TasksMax` or very high task limits
- no memory limit on application services
- no CPU control for untrusted or bursty workers
- build workers with no cgroup limits
- multiple projects sharing one service cgroup unnecessarily
