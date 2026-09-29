# Narrow Remote Configuration Clarification

## Trigger

After application builds moved entirely to local Docker, the BonesDeploy config
sync still serialized the complete local `Runtime` model to BonesRemote. That
model includes framework selection, local build language settings, permission
defaults, and arbitrary framework values that BonesRemote does not consume.

## Decision

BonesDeploy sends a dedicated backend-specific remote runtime descriptor instead
of the local `Runtime` model. Native descriptors contain only the web root and
the optional Ruby version required by the server-side Rails migration script.
Docker descriptors contain only the optional Compose ingress port and Compose
startup timeout. The descriptor retains the release retention count and an
explicit backend discriminator.

Framework template, Node version, permission defaults, Python version, and
arbitrary runtime extras are not synchronized to BonesRemote. Python version is
removed from the remote prepare environment because no server-side prepare
script consumes it; Django uses it only during the local build.

The new descriptor is the only accepted control-plane snapshot format. This
breaking-release branch does not retain deserialization compatibility for the
old broad runtime snapshot. The artifact manifest remains unchanged because its
identity, kind, platform, format, length, digest, and Compose inventory fields
are enforceable transport facts rather than local build configuration.

Nearby control-plane code, tests, direct dependencies, and documentation made
obsolete by narrowing the descriptor are removed as part of the same change.
Provisioning requests sent to BonesInfra and production secrets written to
`shared/.env` remain separate contracts and are not narrowed by this decision.

## Supersedes

This clarification supersedes serializing `RemoteDeploymentConfig.runtime` as
the complete local `Runtime` model and retaining unused runtime values in the
persisted BonesRemote snapshot. It adds detail to the existing breaking-release
decision that previous configuration formats do not constrain this release.

## Required Authoritative Updates

- `01-idea.md` includes the narrow remote control-plane contract in scope and
  preserves the distinction between deployment, provisioning, and secrets.
- `02-plan.md` defines backend-specific descriptor ownership, consumers,
  cleanup, risks, and validation.
- `03-tasks.md` records implementation and validation of the narrowed contract.
