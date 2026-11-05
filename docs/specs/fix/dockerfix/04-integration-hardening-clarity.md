# Clarification

## Trigger

After the initial implementation and validation, the user requested correction
of every remaining issue from the integration audit. The audit found missing
focused lifecycle and diagnostic tests plus concrete Podman, package-source,
migration-cleanup, and manifest-failure defects.

## Decision

Complete the hardening in the existing Dockerfix change. Compose lifecycle tests
use a fake command executor and never require Docker. Compose-only doctor runs
skip Podman checks. Doctor findings use a pure classifier. Docker packages come
from Docker's official Debian/Ubuntu apt repository. Migration cleanup covers
the real `gunicorn` and `puma` service names. Manifest inspection failures are
reported explicitly.

The ignored Compose E2E scenario remains compile-only because repository policy
prohibits agents from running E2E tests locally.

## Supersedes

This supersedes the completion note that no implementation work remained and
the package-installation assumption that distribution and Docker-upstream
package names could be combined. It adds test detail without changing the
original Compose lifecycle or security contract.

## Effect on the record

`01-idea.md` now includes Compose-only diagnostic and package-source outcomes.
`02-plan.md` records the chosen Podman gating, doctor classifier, official apt
repository, manifest failure, and fake-command test behavior. `03-tasks.md`
contains the concrete hardening implementation and validation tasks.
