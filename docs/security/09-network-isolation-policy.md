# BonesDeploy Network Isolation Policy

## Purpose

Network exposure should be minimal and intentional.
Public ports, backend listeners, and outbound access should all be narrowed to the smallest practical set.

## Rules

- Only necessary public ports should be exposed.
- App backends should bind to localhost or a Unix socket.
- The reverse proxy should be the only public ingress layer.
- Outbound access should be reduced for build jobs and other risky workers.

## BonesDeploy Notes

- `docs/commands/bonesremote/landlock-nginx.md` should keep nginx tied to the site's `public_path`.
- `docs/commands/bonesremote/release-activate.md` and `hooks-post-deploy.md` assume the web layer reaches the active release through that path.
- Do not expose databases, Redis, or admin dashboards publicly unless there is a documented reason.

## Findings

- app listens on a public interface unnecessarily
- database or cache bound to `0.0.0.0`
- backend not restricted to localhost or a socket
- build worker has unnecessary access to internal services
