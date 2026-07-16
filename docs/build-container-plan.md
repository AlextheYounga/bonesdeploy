# Build Container Plan

BonesDeploy build scripts should own framework-specific provisioning. `bonesremote` should provide a generic, isolated Linux build container and run the user/template scripts inside it without knowing whether the project is Nuxt, Laravel, Rails, Django, or something else.

## Current Problem

Today each build script is executed with its own `podman run --rm` invocation. That means filesystem changes outside the mounted source tree disappear after every script.

This breaks the intended script model:

1. `01_install_node_deps.sh` installs Node into `/workspace/build/node`.
2. That container exits and its overlay filesystem is removed.
3. `02_run_build.sh` starts a fresh container.
4. `/workspace/build/node` no longer exists, so the script falls back to whatever is in the image.

The scripts were written as if the build workspace persisted for the whole build phase. The runner currently does not provide that.

## Decision

Run all build scripts for a deployment inside one short-lived container.

`bonesremote` should:

1. Create and start one build container at the beginning of the build phase.
2. Mount only the exported source tree at `/workspace/source`.
3. Set the same build environment for every script.
4. Execute each script with `podman exec -i <container> bash -s`.
5. Stream output into the same per-script logs used today.
6. Remove the container on success or failure.

This preserves tool installs across scripts in a single deploy without persisting toolchains across deploys.

## Base Image

The `[runtime]` section of `bones.toml` should not choose the build image. For now, `bonesremote` uses one hardcoded base:

```text
docker.io/library/buildpack-deps:bookworm
```

The image provides common build and download prerequisites. Framework templates stay dynamic through shell scripts; if a framework needs PHP extensions, Ruby packages, Python packages, or additional system libraries, its build scripts install them inside the per-deploy container.

## Security Model

This keeps the existing isolation boundary:

- the container runs as the unprivileged site build user
- the source export is the only mounted project path
- `.env`, `shared/`, `current`, `releases/`, the bare repo, and root-owned control-plane state are not mounted
- the container is deleted after the build phase
- promotion still validates and hardens the release tree before activation

No toolchain cache is reused across deploys. A malicious dependency can affect the current build, but it cannot poison a persistent host toolchain for future builds.

## Tradeoff

Cold installs happen on every deploy. That is acceptable for the first implementation because it keeps the model simple and avoids cache poisoning concerns.

If deploy time becomes painful, the next step should be a content-addressed provisioning image:

1. Templates optionally provide a provisioning script.
2. `bonesremote` hashes that script.
3. If no local image exists for the hash, `bonesremote` builds one from the generic base.
4. Build scripts run in that image.

Avoid a mutable per-site toolchain volume unless there is a clear need. It speeds up deploys but lets one deploy's dependency install influence future deploys.

## Template Notes

- Nuxt still needs to remove its compatibility `dist` symlink after `generate`, because Nuxt creates `dist -> /workspace/source/.output/public` and absolute symlinks are rejected during promotion.
- Node installer scripts should not rely on PHP for `package.json` parsing. The common build image does not include PHP unless the template installs it.

## Implementation Checklist

- Create the build container once when the build phase starts, and keep its container ID for the rest of the deploy.
- Run every build script with `podman exec -i <container> bash -s` so filesystem changes survive across scripts.
- Remove the container in the same success and failure paths so a broken deploy cannot leak state.
