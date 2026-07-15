# Rootless build user

Each site has a dedicated `<site>-build` user for rootless Podman builds. The
runtime user remains separate and does not need a home directory.

## Build execution

`bonesremote` creates a disposable build context under:

```text
/srv/sites/<site>/tmp/build-...
```

It chowns that context to `<site>-build`, then runs Podman through the user's
systemd session:

```text
systemd-run --machine=<site>-build@ --quiet --user --collect --pipe --wait podman ...
```

This is required for rootless Podman with cgroup v2; `runuser` plus a synthetic
`HOME`/`XDG_RUNTIME_DIR` is not a valid substitute. The build user needs a real,
writable home, lingering, subordinate UID/GID ranges, and delegated `cpu`,
`cpuset`, `memory`, and `pids` controllers.

Build scripts remain root-owned and are piped to the container over stdin. The
build user therefore receives only the selected script and cannot read the
deployment source tree directly.

## Promotion

Root promotes the finished build context into a sealed release, wires shared
paths, runs prepare scripts as the runtime user, and removes the disposable
context. The runtime user never needs Podman or a writable home.

`bonesremote doctor` checks the build user's home and delegated controllers
before deployment.

## Provisioning

Infrastructure provisioning must install rootless Podman prerequisites, create
the per-site build user and home, enable lingering, allocate unique subordinate
UID/GID ranges, and verify rootless Podman as that user. These settings belong
in bonesinfra rather than in deployment-time workarounds.
