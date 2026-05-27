# BonesDeploy Identity and User Policy

## Purpose

This document defines the Unix identity model BonesDeploy should use.
The model is intentionally simple: one `deploy_user` for deployment orchestration and one dedicated `service_user` per project for runtime execution.

## One Project, One Service User

Each project should run as its own unprivileged Unix service user.

Bad:

```text
all projects run as www-data
all projects run as git
all projects run as root
```

Good:

```text
app1 runs as app1
app2 runs as app2
app3 runs as app3
```

The runtime identity should be specific enough that filesystem permissions can isolate one project from another.

## Deploy User Requirements

The deploy identity is the SSH and orchestration identity.
In the current docs and examples this is usually `git`.

Expected properties:

- has a home directory
- accepts key-based SSH access
- has no password login
- has only narrow sudo rights for approved helper commands
- may manage release directories, symlinks, and deploy metadata

The deploy user should not be the long-lived service identity.

## Service User Requirements

The service identity runs the application process.

Expected properties:

- no home directory unless explicitly justified
- no interactive login shell
- no sudo
- no membership in `wheel`, `sudo`, or `docker`
- should not own or read deploy credentials
- should not own deploy metadata in the bare git repo

Recommended shells for non-login service accounts:

```text
/usr/sbin/nologin
/bin/false
```

Temporary debugging access, if ever granted, should be time-limited and removed afterward.

## Separation Rules

The deployment user should remain distinct from:

- all per-project `service_user` identities
- the web server user such as `nginx` or `www-data`
- backup identities
- container admin or host-management identities

Example split:

```text
deploy user: git
service users: project-a, project-b, project-c
web server user: nginx or www-data
```

This separation is necessary so a compromised app does not inherit deploy-wide access.

## BonesDeploy Expectations

Within BonesDeploy's flow:

- `deploy_user` stages releases and writes deployment metadata
- `deploy_user` or root-managed helpers flip `web_root` and `current`
- `service_user` runs the app after activation
- `service_user` should not mutate old releases, deploy state, or the bare repo

## Critical Findings

The following should be treated as critical findings:

```text
service user in sudo group
service user in wheel group
service user in docker group
service user has NOPASSWD sudo rule
service user can restart arbitrary system services with sudo
service user can run package manager commands with sudo
service user can read deployment SSH keys
```

## Additional Findings

The agent or operator should also flag:

- service user has a login shell
- multiple projects share one `service_user`
- service user can write release directories
- service user can read another project's shared state
- deploy user and service user are the same account
