# Clarification

## Trigger

The export is a local administrative operation. The configured root SSH
connection already serves setup and administrative commands; `git` exists only
for deployment.

## Decision

`bonesdeploy site export` connects as the configured root user and runs the
fixed archive command directly over SSH. It does not invoke `bonesremote`, use
the deployment identity, or require sudo. This reuses the existing
administrative boundary and avoids adding server-side authority for a
local-machine operation.

## Supersedes

The prior export plan's decision to stream through a root-backed BonesRemote
`export` subcommand authorized for the `git` deployment identity by a new
sudoers rule.

## Effect on the record

`01-idea.md` now defines export as root-SSH local administration and excludes
BonesRemote, sudoers, and BonesInfra changes. `02-plan.md` assigns the remote
archive invocation to the existing privileged SSH connection. `03-tasks.md`
tracks removal of the mistaken server-side path and validation of the simplified
local implementation.
