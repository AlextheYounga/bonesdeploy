# BonesDeploy Desired End State Summary

## Desired End State

A well-configured BonesDeploy server should look like this:

```text
/srv/deployments/<project> layout per project
one Unix service user per project
deploy user separate from service users
no service users with sudo, docker, or root access
service users cannot write immutable release code
only project-specific writable directories are writable
secrets readable only by intended service users
web_root exposes only the active release through the current symlink
systemd hardening enabled per service
capabilities dropped by default
NoNewPrivileges enabled
cgroup limits set
AppArmor enabled and enforcing where supported
Landlock used for risky child jobs where practical
seccomp used where practical
logs and backups protected
no public databases, caches, or admin services
no Docker socket exposure to applications
```

## Core Principle

The most important practical principle is:

```text
A compromised app should only be able to damage itself and the small set of resources it explicitly needs.
```
