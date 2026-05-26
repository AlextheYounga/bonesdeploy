# BonesDeploy Package Manager and Runtime Policy

## Purpose

Production runtime processes should not need package manager privileges or write access to dependency stores.

## Rules

- Dependencies should normally be installed during build or deploy, not at runtime.
- Service users should not write to global package-manager locations.
- Runtime processes should not receive broader dependency-write access than necessary.

## BonesDeploy Notes

- This fits the release/deploy split used by `bonesremote release-stage` and `release-activate`.
- Keep runtime code immutable so post-deploy hardening can make the active release service-owned but not mutable.

## Findings

- service user can write package-manager globals
- runtime process installs dependencies on demand
- dependency directories are writable when they should not be
