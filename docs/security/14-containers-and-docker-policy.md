# BonesDeploy Containers and Docker Policy

## Purpose

Containers can help isolation, but the Docker socket and broad host mounts are effectively host-level risks.

## Rules

- Treat Docker socket access as critical unless explicitly justified.
- Use dropped capabilities, non-root users, and narrow writable volumes.
- Keep host mounts as small as possible.
- Apply AppArmor, seccomp, and resource limits to containers where possible.

## BonesDeploy Notes

- Containerized projects should still follow the `public_path` and release-directory model.
- Service users should not be able to reach `/var/run/docker.sock` unless a documented admin workflow requires it.

## Findings

- service user can access the Docker socket
- container mounts `/var/run/docker.sock`
- service user is in the `docker` group
- container mounts broad host paths like `/`, `/home`, or `/etc`
