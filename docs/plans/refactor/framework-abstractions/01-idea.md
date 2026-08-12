# Idea

## Request

Consolidate the Rust-side framework system into a single architectural concept
with a clear public front door. The desired shape is:

> One enum or trait representing "a supported application framework," with each
> concrete framework as an implementation. Callers use the concept through its
> public API; they do not dispatch to per-framework modules themselves.

## Problem

The current Rust framework system lacks a single architectural front door. Seven
framework modules (laravel, django, next, nuxt, rails, sveltekit, vue) exist as
sibling modules behind a dispatch module (`frameworks.rs`) that routes `&str`
template names to per-module functions through repeated `match` expressions.

Specific problems:

1. **No canonical framework identity.** The seven template names are string
   literals repeated across five `match` expressions and multiple test functions.
   There is no `enum`, `const` array, or macro that collects them. Adding an
   eighth framework requires touching five match expressions and hoping all call
   sites stay in sync.

2. **Duplicate dispatch across every public function.** `questions()`,
   `validate_answers()`, `configure()`, `environment_example()`, and
   `build_environment_example()` each contain their own full `match` on template
   name, routing to per-module functions. This is the same dispatch pattern
   implemented five times.

3. **Inconsistent signatures for the same concept.**
   `build_environment_example()` takes `&Runtime` for Laravel and Django but
   nothing for the other five frameworks. The dispatch function accommodates both
   by hardcoding which match arm passes which arguments.

4. **Inconsistent visibility.** `questions()` is `pub` from framework modules
   while `environment_example()` and `build_environment_example()` are
   `pub(crate)`. The `Question` and `QuestionKind` types are `pub` and leak
   through to the `ui/prompts` module directly from the dispatch module.

5. **Framework names derived from the filesystem.**
   `framework_assets::framework_names()` discovers template names by listing
   top-level directories in the embedded `assets/frameworks/` filesystem. This
   is decoupled from the Rust source modules — a directory without a
   corresponding module (or vice versa) would cause a runtime error.

6. **Rails ignores its own question.** Rails asks for `ruby_version` (Choice:
   3.2/3.3/3.4) but `build_environment_example()` hardcodes
   `DEFAULT_RUBY_VERSION` (3.3) instead of reading `runtime.extra["ruby_version"]`.

7. **`configure()` is special-cased.** Only Next and Nuxt have a `configure()`
   hook. Five frameworks lack it. The dispatch function has a partial `match`
   that silently no-ops for the other five.

## Definitions

**Framework:** A supported web application framework (Django, Laravel, Next,
Nuxt, Rails, SvelteKit, Vue). Each framework provides prompt questions,
answer validation, environment example generation, build environment generation,
and optional post-scaffold configuration.

**Framework identity:** A single canonical representation of which framework is
selected — an `enum Framework` variant — replacing the current `&str` template
name as the dispatch key.

**Public front door:** The `Framework` enum and its methods. Callers convert a
user-provided string to a `Framework` once, then call methods on it. They never
match on framework identity themselves.

**Per-framework module:** A private (`pub(crate)`) module under
`crates/bonesdeploy/src/frameworks/<name>.rs` containing the implementation
details of that framework. Implementation functions become `pub(super)` or
`pub(crate)` — not directly importable by callers outside the `frameworks`
directory.

**Framework contract:** The set of methods available on the `Framework` enum.
Every variant implements every method. Methods that are irrelevant for a given
framework (e.g. `configure()` for Django) have a default no-op.

## Desired outcome

1. `Framework` is an enum with seven variants, each matching one supported
   framework. It is the single canonical source of framework identity.

2. All current public dispatch functions (`questions`, `validate_answers`,
   `configure`, `environment_example`, `build_environment_example`) become
   methods on `Framework`.

3. Callers convert `&str` template names to `Framework` once at the CLI / user
   input boundary, then pass the `Framework` value to the rest of the system.
   No code outside `frameworks.rs` matches on framework identity.

4. Per-framework module functions become `pub(super)` — only callable from
   `frameworks.rs`'s method bodies. The `Question` and `QuestionKind` types
   remain `pub` and accessible through the `Framework` API.

5. `framework_assets::framework_names()` is replaced by `Framework::ALL` or an
   equivalent constant, removing the filesystem-based name derivation.

6. `build_environment_example()` has a unified signature: all framework variants
   accept `&Runtime`. Frameworks that don't need it ignore it.

7. `configure()` is a method on `Framework` with a default no-op implementation
   in the enum method body, not in each variant.

8. Rails' `build_environment_example()` reads `runtime.extra["ruby_version"]`
   instead of hardcoding `DEFAULT_RUBY_VERSION`.

9. Existing behavior is preserved: CLI commands, config formats, output content,
   error messages, and test coverage remain unchanged.

## Scope

The change includes:

- Introducing a `pub enum Framework` with seven variants.
- Moving all dispatch logic into methods on `Framework`.
- Reducing per-framework module function visibility to `pub(super)`.
- Replacing `frameworks::questions(template_name)` and similar free-function
  calls with `framework.questions()` calls at all call sites.
- Removing the five duplicate match expressions.
- Unifying `build_environment_example()` to accept `&Runtime` uniformly.
- Fixing Rails to consume `runtime.extra["ruby_version"]` instead of hardcoding.
- Replacing `framework_names()` filesystem derivation with a method on
  `Framework`.
- Adding or updating tests that verify the Framework enum contract.

## Constraints

- The `Question` and `QuestionKind` types must remain `pub` — they are consumed
  by `ui/prompts.rs` and `commands/init/framework.rs` to build interactive
  prompts.
- Per-framework module files must stay under 400 lines. Most are already under
  100 lines; moving implementation into the enum file must not violate this.
- `Framework` enum `impl` blocks may be split across the file or dispatch to
  per-module functions — the enum file must not exceed 400 lines.
- CLI behavior, config output, error messages, and test coverage must not
  regress.
- `cargo clippy`, `cargo fmt`, and `cargo test` for affected crates must pass.

## Exclusions

- This change does not touch the Python-side framework system (`bonesinfra`).
- It does not introduce a trait or trait objects. The enum is the chosen Rust
  mechanism for a closed set of known framework implementations.
- It does not add or remove framework support.
- It does not refactor the Git, SSH, or deployment orchestration boundaries.
- It does not change framework question content, environment example output, or
  build environment content beyond the Rails bug fix.
- It does not change `ui/prompts.rs` beyond adapting imports and call signatures.
- It does not touch `infra/assets/frameworks.rs` beyond replacing
  `framework_names()`.