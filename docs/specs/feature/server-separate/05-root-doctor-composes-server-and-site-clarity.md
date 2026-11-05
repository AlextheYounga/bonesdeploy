# Clarification

## Trigger

The root `bonesdeploy doctor` command should remain available as a convenience
workflow and should run both the host and project checks.

## Decision

Retain root `bonesdeploy doctor` as a thin diagnostic composition command. It
runs `bonesdeploy server doctor` followed by `bonesdeploy site doctor`, shows
both results, and returns failure if either check fails. It still invokes the
site check after a server-check failure so one command reports the complete
diagnostic state. The scoped doctor commands remain independently callable,
including `site doctor --local`.

## Supersedes

This supersedes the earlier plan language that removed the root `doctor`
command. It adds no change to server/site doctor ownership or to the server
readiness guard used by site setup.

## Effect on the record

`01-idea.md` now defines root doctor as a retained composed workflow and limits
command removal to the obsolete flat inspection commands, `remote`, and
`guide`. `02-plan.md` assigns root doctor to a thin server-then-site diagnostic
composition with aggregated failure. `03-tasks.md` adds implementation and
integration-test work for that composition and its public-help review.
