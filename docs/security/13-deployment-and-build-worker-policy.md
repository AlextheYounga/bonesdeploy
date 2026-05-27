# BonesDeploy Deployment and Build Worker Policy

## Purpose

Deployment and build work should be separated from runtime service execution.
The deploy identity can manage releases, but it should not become the runtime identity.

## Separate Deployment Service

Deployment orchestration should run as the BonesDeploy deploy user, not as root unless absolutely necessary.
Where root actions are needed, use narrow, audited helper commands rather than broad sudo.

## Atomic Releases

Deployments should use release directories and atomic symlink flips:

```text
/srv/deployments/<project>/runtime/<release-id>
/srv/deployments/<project>/current -> runtime/<release-id>
/var/www/<project> -> /srv/deployments/<project>/current
```

Service users should not mutate old releases or deployment metadata.

## Build Isolation

Builds should occur in a staging workspace, not directly inside `public_path` or the active release tree.
In the current model that workspace is:

```text
/srv/deployments/<project>/build/workspace
```

Build scripts should run with:

- deploy user or another dedicated build identity
- no sudo
- dropped capabilities where possible
- cgroup limits
- AppArmor profile where available
- Landlock restrictions where practical
- minimal secrets
- controlled network access

## Findings

The agent or operator should flag:

- build scripts running as root
- build scripts running as deploy user with broad access to all projects
- package install scripts inheriting production secrets
- build workspace shared across projects
- public path writable during build
- current symlink writable by service user
- deployment SSH keys readable by service user
