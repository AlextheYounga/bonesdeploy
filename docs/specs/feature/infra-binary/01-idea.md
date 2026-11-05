# Idea

## Request

Replace the expanded, committed `infra/.framework/` BonesInfra source tree with
one committed wheel artifact. Keep BonesInfra templates as ordinary committed
files under `infra/` so projects can inspect and edit them.

## Problem

Projects currently commit the complete managed BonesInfra source tree, including
many files that are not project-specific. This creates noisy repositories and
makes it difficult to distinguish project infrastructure from the BonesDeploy
implementation it depends on.

## Definitions

**Managed wheel:** A pure-Python `.whl` containing BonesInfra code, metadata, and
the package resources required by the runtime. It is generated and committed in
the `bonesinfra` Rust crate, embedded in the `bonesdeploy` executable, and copied
to each project as one artifact. It is not a native executable.

**Managed templates:** The Jinja and script templates supplied by BonesInfra and
copied into the project’s `infra/` directory. They are ordinary project files
and may be edited by the project owner.

## Desired outcome

An initialized project contains one committed BonesInfra wheel and a committed
template tree under `infra/`; it does not contain `infra/.framework/` or the
expanded managed Python source. BonesDeploy installs and executes the committed
wheel in its existing project-scoped cached virtualenv, while provisioning uses
the project’s committed templates and `infra/custom/` extensions.

`bonesdeploy update` replaces the managed wheel and refreshes managed templates
from the new executable. Managed templates are intentionally replaced wholesale
in v1. Project-specific behavior belongs in `infra/custom/`.

## Scope

This includes committed wheel generation and embedding, the project artifact
layout, wheel installation and hash detection, initialization, update
synchronization, template loading, custom provisioning composition, migration of
existing projects, and tests and documentation.

## Constraints

The wheel must be a portable pure-Python `py3-none-any` artifact. It is committed
in the Rust crate, included in the crates.io package, and embedded in the
executable. End users must not need Python wheel-building tools during Cargo
compilation. BonesDeploy installs the wheel rather than importing from the wheel
archive. Existing project-scoped dependency caching and atomic replacement
behavior are preserved.

All supported framework templates and shared template assets are copied into the
project template tree. `infra/custom/` and `infra/secrets/` remain project-owned.

## Exclusions

V1 does not create a native platform-specific executable, preserve edits to
managed templates across updates, merge templates, or create a template
manifest. It does not move `infra/custom/`, `infra/secrets/`, or deployment
scripts into the wheel, lock third-party dependencies, or redesign the remote
`bonesremote` lifecycle.
