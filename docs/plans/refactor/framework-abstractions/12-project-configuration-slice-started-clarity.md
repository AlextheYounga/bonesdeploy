# Clarification

## Trigger

The Framework slice is complete and the user authorized continuing with the next
dependency-ordered boundary: project configuration.

## Decision

The project configuration slice keeps the flat `.env` behavior while making the
existing Rust configuration module the owner of the Rust parse/write pair. The
full Rust key vocabulary is centralized in `bonesdeploy-core`. Python keeps its
existing parser and uses one shared key module plus that parser for framework
selection. The dead `bonesinfra_input` stdin contract and stale nested `App` serde
remain deferred to their designated child boundaries.

## Supersedes

This narrows the next implementation slice without changing the parent ownership
map or the exclusion of integration and stale-serde work.

## Effect on the record

`01-idea.md` records project configuration as the authorized second slice;
`02-plan.md` records the chosen Rust/Python key-vocabulary and parser approach;
`03-tasks.md` records the completed writer move, key centralization, tests, and
deferred integration/serde boundaries.
