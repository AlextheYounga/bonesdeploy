# BonesDeploy Containers and Docker Policy

## Purpose

Containers can add isolation, but Docker socket access and broad host mounts are effectively host-level risks.

## Docker Socket

Access to the Docker socket is equivalent to root-level control of the host in many practical deployments.
The following should be treated as critical unless explicitly justified:

```text
service user can access /var/run/docker.sock
container mounts /var/run/docker.sock
service user is in docker group
```

## Container Hardening

Containers should use:

```text
--cap-drop=ALL
--security-opt no-new-privileges:true
read-only root filesystem where practical
specific writable volumes only
non-root user
memory limits
pids limits
CPU limits
AppArmor and seccomp profiles
```

## Volume Policy

Containers should not mount broad host paths such as:

```text
/
/home
/root
/etc
/srv/deployments
/var/run/docker.sock
```

unless there is a specific administrative container with strong justification.

## Findings

The agent or operator should flag:

- service user can access the Docker socket
- container mounts `/var/run/docker.sock`
- service user is in the `docker` group
- container mounts broad host paths such as `/`, `/home`, or `/etc`
