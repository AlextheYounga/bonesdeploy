# BonesDeploy Network Isolation Policy

## Purpose

Network exposure should be minimal and intentional.
Public ports, backend listeners, and outbound access should all be narrowed to the smallest practical set.

## Inbound Exposure

Only necessary public ports should be exposed.
Typical public ports:

```text
22/tcp   SSH, ideally restricted by source IP or key-only auth
80/tcp   HTTP redirect or ACME challenge
443/tcp  HTTPS
```

Application backend ports should bind only to localhost or private interfaces.

Bad:

```text
app listens publicly on 3000, 8000, 8080, 9000
```

Good:

```text
reverse proxy listens on 80 and 443
app listens on 127.0.0.1:3000 or a Unix socket
```

## Outbound Access

Outbound access should be minimized for high-risk workers.
Build jobs are a special case because package managers often need internet access, but build jobs should not automatically receive access to internal metadata services, private service networks, database ports, or secrets services.

## Findings

The agent or operator should flag:

- applications listening on public interfaces unnecessarily
- internal admin dashboards exposed publicly
- databases bound to `0.0.0.0`
- Redis or Memcached bound publicly
- Docker API exposed over TCP
- build workers with unnecessary access to internal services
