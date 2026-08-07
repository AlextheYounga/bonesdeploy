# Clarification

## Trigger

Anonymous RON struct values such as `(...)` do not tell a reader what subject
the enclosed fields describe, even when Rust deserializes them into a known
type.

## Decision

Every RON struct value must use named struct syntax with the corresponding Rust
type name. This applies to top-level specification objects and nested objects;
for example, `RuntimeDefaults(...)` contains `PermissionRule(...)` values.
Names make each object self-identifying where it is defined.

## Supersedes

This adds precision to the existing requirement that Core specifications are
typed. It supersedes the implicit assumption that deserializing a bare
parenthesized object into a Rust struct was sufficiently explicit.

## Effect on the record

`01-idea.md` now defines a typed Core specification as using named RON struct
values and records the rule as a constraint. `02-plan.md` applies named syntax
to every top-level and nested specification object and records the readability
decision and validation risk. `03-tasks.md` adds the RON edits, repository
convention, and repeated validation checks.
