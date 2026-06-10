# Future

This document captures upcoming changes we want to fold into BonesDeploy without forcing them into the current flow yet.

## Build Isolation With Podman

We want an optional build runner based on Podman so build steps get isolation without depending on Docker.

Desired shape:

- keep the current `/srv/deployments/<project>/build/workspace` layout
- run build steps inside a rootless Podman container when enabled
- mount only the build workspace and any explicitly allowed cache paths
- pass only an allowlisted set of build-time environment variables
- keep runtime secrets out of the container by default
- keep activation, post-deploy hardening, and service restarts on the host

This is meant to harden dependency-heavy build steps, not to turn BonesDeploy into a container-first platform.

## Build And Deploy Separation

We want to make the build/deploy boundary more explicit.

Current intent:

- build produces an artifact or release-ready output
- deploy publishes that output
- runtime services consume the published release

Long term, the deploy path should avoid rebuilding on the production host when a reusable artifact already exists.

## Origin-Based Deployment Flow

Today the deployment model is effectively local -> production.

We want to support a Coolify-style flow as well:

1. local changes are pushed to `origin`
2. production pulls from `origin`
3. deploy then promotes what was pulled into the live release

This would give us a more explicit source-of-truth remote and a cleaner promotion path between local work and production deployment.

## Secrets Model

Build steps should not automatically receive shared production secrets.

Instead:

- build secrets should be narrow and job-specific
- runtime secrets should stay in runtime-only locations
- deploy credentials should remain separate from build credentials

## Follow-Up Ideas

- document a build-secret allowlist model
- define which deployment scripts are allowed to run in Podman
- define which scripts must stay host-side
- decide how `bonesdeploy deploy` should behave when `origin` is the deployment source
