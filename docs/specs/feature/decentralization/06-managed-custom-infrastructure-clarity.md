# Clarification

<!--
This document records a settled change to the existing planning record.

Never use a clarification file to ask a question or preserve undecided
alternatives.
-->

## Trigger

<!-- State the new information or instruction that required clarification. -->

## Decision

<!-- State the settled decision and why it was made. -->

## Supersedes

<!--
Identify the earlier definition, assumption, scope boundary, or decision
this replaces. If it replaces nothing, state that it adds detail without
changing an earlier decision.
-->

## Effect on the record

<!--
Summarize the corresponding updates made to 01-idea.md, 02-plan.md,
and 03-tasks.md.

The clarification is not complete until the authoritative files reflect
the new current truth.
-->
# Clarification

## Trigger

Moving BonesInfra project-facing provisioning into the application repository
needs an ownership boundary that keeps infrastructure visible and versioned
without permanently forking BonesDeploy-managed behavior.

## Decision

Project-facing provisioning is split into two committed directories:

- `infra/.framework/` contains BonesDeploy-supplied, managed framework code
  code. It remains visible to users and may be refreshed by an explicit
  BonesDeploy update.
- `infra/custom/` contains project-owned provisioning code. Updates
  preserve it and never silently overwrite it.

Core provisioning runs before custom provisioning through ordinary explicit
Python composition. Managed core is the complete copied BonesInfra execution
engine, so every project can execute its committed core without a hidden source
fallback. No general plugin registry is introduced.

An update refreshes managed core files only when their project copies are
unmodified. If a managed file was changed, update reports the conflict and
refuses to overwrite it. The update path does not perform a three-way merge.
There is no canonical per-project infrastructure tree under
`~/.config/bonesdeploy`; the application repository is authoritative.

## Supersedes

Superseded by `refactor/project-local-bonesinfra-core`: managed core is the
complete copied BonesInfra distribution rather than a framework-only snapshot.

## Effect on the record

- `01-idea.md`: Defines managed core and custom provisioning, adds the desired
  directory structure, and records update and composition constraints.
- `02-plan.md`: Adds current-state analysis, scaffolding, composition,
  update-conflict behavior, responsibilities, risks, and validation for the
  ownership boundary.
- `03-tasks.md`: Adds implementation tasks for core/custom scaffolding,
  explicit composition, and safe managed-core updates.
