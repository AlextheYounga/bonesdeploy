# Distribution Python Runtime

## Request

This completed distribution-Python change is superseded by
`feature/mise-runtimes`: Django now uses the exact configured managed Python
runtime. Its historical goal was to avoid production source compilation.

## Problem

BonesInfra downloads and compiles an optimized CPython release with parallel
`make` on every Django host. That can exhaust the memory available on small
production servers. Django artifacts also contain locally installed Python
packages and wrappers tied to the managed interpreter.

## Definitions

**Distribution Python:** The `python3` interpreter and supporting packages from
the supported Debian or Ubuntu APT repositories.

**Legacy Python version:** The accepted but unused `python_version` value from
existing project configuration and remote descriptors.

**Target virtualenv:** The release-owned `.venv` created with distribution
Python while preparing a staged release.

## Desired Outcome

Django local builds and production use the same exact configured Python installed
precompiled-only by pinned mise. The artifact packages dependencies and
site-linked launchers; prepare validates runtime identity, checks Django, runs
migrations unless skipped, and collects static files without installing packages.

## Scope

- This historical record remains for its completed source-build removal. The
  current Django runtime contract is owned by `feature/mise-runtimes`.

## Constraints

- Supported hosts remain Debian 12+ and Ubuntu 24.04+ on `x86_64`.
- Production source-build fallback remains prohibited. The current exact runtime,
  packaged dependency, and E2E requirements are defined by `feature/mise-runtimes`.
- Prepare scripts continue to run as the home-less site runtime user.
- Full E2E tests are not run unless explicitly requested.

## Exclusions

- Installing non-distribution Python versions on production hosts.
- Adding package managers or supporting dependency manifests other than the
  existing `requirements.txt` contract.
- Migrating old managed CPython installations.
