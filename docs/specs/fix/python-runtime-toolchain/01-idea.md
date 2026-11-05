# Distribution Python Runtime

## Request

Apply the distribution-runtime approach used for Ruby to Django so provisioning
does not compile CPython on production servers.

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

Django host provisioning installs Python through APT without compiling CPython.
The local build does not install Python dependencies. Before activation, the
target host creates `.venv`, installs `requirements.txt`, validates Django, runs
migrations unless skipped, and collects static files.

## Scope

- Django production Python provisioning and runtime paths.
- Django artifact cleanup and target-side dependency installation.
- AppArmor, remote descriptor compatibility, tests, generated wheel, and
  documentation affected by the runtime boundary.

## Constraints

- Supported hosts remain Debian 12+ and Ubuntu 24.04+ on `x86_64`.
- Existing `python_version` values remain readable but do not select an
  interpreter and are not generated for new projects.
- Prepare scripts continue to run as the home-less site runtime user.
- Full E2E tests are not run unless explicitly requested.

## Exclusions

- Installing non-distribution Python versions on production hosts.
- Adding package managers or supporting dependency manifests other than the
  existing `requirements.txt` contract.
- Migrating old managed CPython installations.
