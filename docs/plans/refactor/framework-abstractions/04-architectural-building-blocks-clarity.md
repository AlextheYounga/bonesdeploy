# Clarification

## Trigger

The framework-only plan did not represent the requested change. The target is a
repository-wide move toward clear, reusable building blocks for framework
selection, configuration, Git, SSH, provisioning, updates, migrations, doctor,
deployment state, and lifecycle behavior.

## Decision

The existing framework-abstractions record is superseded by an umbrella
architectural refactor plan. `Framework` remains an implementation slice, not the
overall objective.

The refactor will identify major responsibilities and canonical owners, strengthen
existing concepts before introducing new abstractions, give each concept a small
public API with private collaborators, migrate callers through those concepts,
close direct reach-through, preserve crate and Rust/Python/revision boundaries, and
proceed as sequenced reviewable slices rather than one all-at-once rewrite.

The architecture documentation is included in the first slice because it must
describe the post-decentralization system and the target concepts.

## Supersedes

This supersedes the previous framework-only request, scope, intended outcome, and
task sequence in this Acta record. It does not discard the concrete framework
findings; it places them under the broader building-block program.

## Effect on the record

- `01-idea.md` now defines the repository-wide architectural goal, target concepts,
  boundaries, constraints, and exclusions.
- `02-plan.md` now describes the current cross-crate architecture, side doors,
  composition rules, and sequenced refactor approach.
- `03-tasks.md` now sequences configuration, integrations, infrastructure updates,
  provisioning, framework, deployment, doctor, and side-door closure work.
- Architecture documentation updates are explicit implementation tasks, including
  correction of stale `.bones`/`bones.toml`/removed-command references.
