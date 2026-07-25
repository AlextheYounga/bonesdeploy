# Code Conventions

Derived from our [Uncle Bob Craft](../.agents/skills/uncle-bob-craft/SKILL.md) criteria and the automated cleancode assertions in `tests/cleancode/`. Run `cargo test -p cleancode` to verify structural rules.

## File Size

- **Max 400 lines per file** (enforced by `cleancode_file_too_long`).
- Approaching the limit signals that a focused sub-module should be extracted.

## Naming

- Rust standard casing: `snake_case` for functions, variables, and modules; `CamelCase` for types and traits; `SCREAMING_SNAKE_CASE` for constants.
- Names should read without a comment — prefer `release_dir` over `get_path`.
- **Test function names read as natural-language sentences**: the name is the specification, not a shorthand. Example: `parse_ssh_style_url_parses_host_port_and_repo_path`, `materializes_base_bones_assets`.

## Functions and Modules

- Single purpose, single level of abstraction per function (SLAP). A function that both decides and executes belongs as two.
- Modules follow Single Responsibility: one concept, one reason to change.
- Do not introduce abstractions (traits, enums, generics) until duplication or variation actually justifies them — at the second or third occurrence, not speculatively.

## Error Handling

- Propagate errors with `?`; add context with `anyhow` at call sites.
- Never use `unwrap_or(x)` or `unwrap_or_else(f)` on a statically-known `Some(…)` or `Ok(…)` receiver — the fallback is dead code (enforced by `cleancode_no_literal_wrapped_fallback`).
- Never construct `Ok(…)` or `Some(…)` from an error arm of a `match` — this manufactures success silently (enforced by `cleancode_no_manufactured_success`).
- `.unwrap()` and `.expect()` are denied in production code (Clippy). They are acceptable in tests.

## State and Constants

- File paths, config keys, and environment variable names that appear in more than one file must be constants in `shared::paths` or an equivalent canonical location (enforced by `cleancode_no_duplicated_state`).
- `BONES_*` environment variable names are reserved for derived values; never define them in user code.

## Vocabulary

- The codebase must not contain the terms `legacy`, `hack`, `workaround`, `backcompat`, or `deprecated` in identifiers, comments, or string literals (enforced by `cleancode_no_legacy_terms`). Name things for what they are today, not what they replaced.

## Comments

- **Default to no comments.** Well-named identifiers carry the meaning.
- Add a comment only when the WHY is non-obvious: a hidden constraint, a subtle invariant, a workaround for an external bug.
- Every `unsafe` block **must** have a `// Safety:` comment explaining the invariant that makes it safe (enforced by Clippy `undocumented_unsafe_blocks = "deny"`). The comment belongs on the block, not repeated on every statement inside it. When a helper function encapsulates the unsafe, a single comment on the helper suffices.
- Public functions must have doc comments covering `# Errors` and `# Panics` where applicable (enforced by Clippy `missing_errors_doc` and `missing_panics_doc`). Limit prose to what function names and types do not already express.
- Multi-paragraph inline explanations inside function bodies are a signal to extract and rename.

## Architecture and Dependencies

- Dependencies point **inward**: `shared` is the innermost crate; `bonesdeploy` and `bonesremote` depend on it. Neither `shared` nor `bonesremote` may import from `bonesdeploy`.
- Introduce a design pattern only when duplication or an axis of variation demands it. Three similar lines are preferable to a premature abstraction.

---

## Tests

There are three tiers, defined by the process effects each test requires.

### Tier 1 — Inline unit tests

**Criterion:** Pure logic — no filesystem, no environment variables, no subprocess, no external state.

**Location:** `#[cfg(test)] mod tests { … }` at the bottom of the source file.

**Examples:** URL parsing (`infra/git.rs`), env-file parsing (`shared/src/env_build.rs`), config validation (`shared/src/config.rs`).

### Tier 2 — Module-level `tests.rs`

**Criterion:** Tests need filesystem access or substantial setup infrastructure, AND they must access `pub(crate)` test helpers. Must not mutate process-global state (environment variables, working directory).

**Location:** A `tests.rs` file alongside `mod.rs`, included via `#[cfg(test)] mod tests;`.

**Examples:** `crates/bonesremote/src/release/state/tests.rs` — filesystem ops on temp dirs using the `ScopedRoot` thread-local guard.

**Why not inline:** The test infrastructure would bloat the source file past the 400-line limit, or the helpers are substantial enough to warrant a dedicated file.

### Tier 3 — Workspace test crates

**Criterion:** The test spans multiple crates, scans the workspace structurally, or makes cross-crate assertions.

**Location:** A workspace member under `tests/<name>/`.

**Examples:** `tests/cleancode/` — structural assertions over every source file in the workspace.

---

### Tests that mutate process-global state

Tests that manipulate `HOME`, `XDG_*` variables, or the current working directory touch process-global state and cannot safely run in parallel. They require:

- A process-wide `Mutex` for serialization.
- RAII cleanup that restores every mutated variable in `Drop`.
- A `// Safety:` comment on each `unsafe` env-mutation block — placed once on the encapsulating helper, not repeated per call site.

**In a binary crate (no `lib.rs`)**, Rust integration tests cannot import the crate's modules, so these tests must live in a `tests.rs` file beside the command module they exercise. This is the correct placement given that constraint — it is not the same as a Tier 2 test, but it is the best available option without a library target. If `bonesdeploy` ever gains a `lib.rs`, these tests should migrate to `crates/bonesdeploy/tests/`.

**Example:** `crates/bonesdeploy/src/commands/init/tests.rs`.
