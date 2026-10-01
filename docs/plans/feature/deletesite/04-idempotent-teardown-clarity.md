# Idempotent Teardown Clarification

## Trigger

The persisted decommissioning plan and deletion tombstone made successful site
deletion remain visible as an active registration, blocked fresh setup, and
introduced lifecycle states that do not match the command's intended behavior.

## Decision

Site deletion is a direct, idempotent teardown. BonesDeploy validates the
current site inventory, confirms deletion, asks BonesInfra to remove the
site-owned resources, and then asks BonesRemote to remove the site's registration
directory. Missing resources and an already-missing registration are successful
no-ops, so rerunning the command resumes naturally after interruption.

BonesRemote no longer persists deletion plans, decommissioning markers, or
tombstones. It does not block normal mutations based on deletion state, and site
setup does not reactivate deleted state. A deleted site has no registration and
is recreated through the normal setup path.

## Supersedes

This supersedes the persisted deletion plan, decommissioning, tombstone,
mutation-blocking, verification transition, and setup-reactivation design in the
original deletion plan.

## Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` are rewritten to define and track
the idempotent teardown design.
