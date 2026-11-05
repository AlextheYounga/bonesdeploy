# Clarification

## Trigger

The requested command break should not remove root `bonesdeploy setup`. The
existing convenience workflow remains useful when provisioning a fresh host
and project together, and all provisioning operations are idempotent.

## Decision

Retain root `bonesdeploy setup [--yes]` as a thin composition command. It runs
`bonesdeploy server setup --yes` first and then `bonesdeploy site setup --yes`.
The root command owns sequencing and failure propagation only; server and site
commands retain all provisioning responsibilities and can be run independently
or repeatedly. If server setup fails, root setup stops and does not invoke site
setup. If both delegated workflows succeed, root setup reports the same
completed site workflow and next-step guidance as site setup.

## Supersedes

This supersedes the earlier plan language that removed the root `setup`
command. It adds no change to the server/site ownership boundary, the server
readiness guard, or the exact site setup sequence.

## Effect on the record

`01-idea.md` now defines root setup as a retained convenience workflow and
limits command removal to obsolete flat inspection commands, `remote`, and
`guide`. `02-plan.md` assigns root setup to a thin Rust composition command and
specifies its server-then-site delegation and failure behavior. `03-tasks.md`
adds implementation and validation work for the composition path and updates
the public-help review requirement.
