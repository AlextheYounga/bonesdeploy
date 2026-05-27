# BonesDeploy Landlock Policy

## Purpose

Landlock should be used where a process can voluntarily reduce its own filesystem access before executing risky child code.
It is a dynamic per-process or per-job sandboxing layer, not a replacement for AppArmor or systemd hardening.

## Landlock Usage Model

Good Landlock use cases in BonesDeploy include:

- build jobs
- dependency install steps
- asset compilation
- plugin execution
- project-specific deploy hooks
- any child process that should only access a small workspace

Landlock is most useful when the job knows exactly which paths it should be allowed to read and write.

## Deployment Worker Intent

A deployment worker may have broad enough privileges to manage releases, but each risky child job should be restricted to only the paths it actually needs.

For job `job-123` deploying `app1`, the intended Landlock shape should approximate:

```text
allow read:       /usr
allow read:       /bin
allow read:       /lib
allow read:       /lib64
allow read:       /etc/ssl
allow read/write: /srv/deployments/app1/build/workspace
allow read/write: /srv/deployments/app1/runtime/<new-release>
deny:             /srv/deployments/app2
deny:             /srv/deployments/app3
deny:             /srv/deployments/app1/shared/.env unless required
deny:             /root
deny:             /home
deny:             /etc/ssh
```

If the job does not need network access, network access should be denied or avoided through additional controls.

## Boundaries

Landlock should be described only as a job-time sandbox for child processes.
It is not the main policy tool for:

- long-lived static host confinement
- user identity separation
- reverse proxy exposure
- secret placement

Those are covered by separate sections.

## Findings

The agent or operator should flag:

- risky child jobs run without any sandbox
- build scripts inherit access to deployment secrets unnecessarily
- jobs can read all project directories
- jobs can write outside their workspace
- jobs can access SSH keys, `.env` files, database files, or tokens not needed for the task
- Landlock is assumed to protect a process that never actually invokes it
