# Clarification

## Trigger

The umbrella architecture record contained broad implementation tasks and
provisional concepts. Acta requires a settled change, one chosen approach, and
concrete tasks with clear completion conditions.

## Decision

This Acta record is narrowed to defining and decomposing the architectural
refactor program. It does not claim to implement every repository-wide refactor.
Each implementation boundary will receive its own reviewed child Acta change.

The parent record contains only the settled architecture vocabulary, canonical
owners, child-change order, child-plan requirements, and validation of the planning
record itself.

## Supersedes

This supersedes the previous umbrella implementation roadmap in `01-idea.md`,
`02-plan.md`, and `03-tasks.md`, which treated all child refactors as one broad
implementation plan.

## Effect on the record

- `01-idea.md` defines a documentation-and-decomposition parent change with settled
  scope and exclusions.
- `02-plan.md` contains the repository-grounded architecture inventory, ownership
  rules, chosen parent/child approach, and child-plan contract.
- `03-tasks.md` contains concrete parent planning tasks and explicit child-plan
  deliverables rather than generic implementation tasks.
- Implementation waits for the relevant child Acta plan and human approval.
