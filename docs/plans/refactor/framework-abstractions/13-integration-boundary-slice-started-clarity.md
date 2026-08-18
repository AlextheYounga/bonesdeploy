# Clarification

## Trigger
The user authorized continuation after the project configuration slice. The next
dependency-ordered boundary is Git and SSH integration, and repository inspection
found direct Rust process calls in doctor, update release version checks, and
release-source cloning.

## Decision

This slice strengthens the existing `infra::git` and `infra::ssh` boundaries and
migrates those Rust callers through them. `infra::git` owns local branch inspection
and release-source cloning. `infra::ssh` owns remote command execution for the
remote version check. The Python bare-repository setup, secrets shell composition,
and dead `bonesinfra_input` contract remain deferred to their designated child
boundaries.

## Supersedes

This adds implementation scope to the integration boundary without changing the
parent ownership map or the deferred Python and input-contract exclusions.

## Effect on the record

`01-idea.md` records the integration slice as the authorized third slice;
`02-plan.md` records the selected wrapper operations and deferred work; and
`03-tasks.md` records the completed integration tasks and focused validation.
