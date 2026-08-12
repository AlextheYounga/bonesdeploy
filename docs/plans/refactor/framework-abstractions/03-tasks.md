# Tasks

## Implementation

- [ ] Add the `Framework` enum with seven variants, `ALL` const, `name()`, and `parse()` methods to `crates/bonesdeploy/src/frameworks.rs`. Place before the existing submodule declarations.

- [ ] Implement public methods on `Framework`: `questions()`, `validate_answers()`, `configure()`, `environment_example()`. Each method body contains a single `match self { ... }` dispatching to per-framework `pub(super)` functions.

- [ ] Implement `pub(crate) fn build_environment_example(&self, runtime: &Runtime)` on `Framework`, dispatching through the same single-match pattern.

- [ ] Change all per-framework module function visibility from `pub`/`pub(crate)` to `pub(super)`. Unify `build_environment_example` signatures to accept `runtime: &Runtime` in all 7 modules. Next, Nuxt, Rails, SvelteKit, and Vue modules add `_runtime: &Runtime`; they ignore it.

- [ ] Fix Rails `build_environment_example` to read `runtime.extra.get("ruby_version")` with fallback to `DEFAULT_RUBY_VERSION`, matching the Laravel/Django pattern.

- [ ] Convert `commands/init/framework.rs` callers: parse the template string to `Framework` in `resolve_template()`, change `FrameworkSelection.template` from `Option<String>` to `Option<Framework>`, and replace `frameworks::validate_answers(name, ...)` and `frameworks::questions(name)` with `framework.validate_answers(...)` and `framework.questions()`.

- [ ] Convert `commands/init/scaffold.rs`: replace `frameworks::configure(&template_name, cfg)` with `framework.configure(cfg)`.

- [ ] Convert `commands/init/mod.rs`: adapt to `FrameworkSelection.template` type change.

- [ ] Convert `commands/secrets/mod.rs`: extract `Framework` from `FrameworkSelection` or config, replace `frameworks::environment_example(template, ...)` with `framework.environment_example(...)`.

- [ ] Convert `infra/assets/frameworks.rs`: accept `Framework` instead of `&str` in `scaffold_framework_env_build`, replace `frameworks::build_environment_example(framework, ...)` with `framework.build_environment_example(...)`, and replace `framework_names()` iterator with `Framework::ALL.iter()`.

- [ ] Update tests in `frameworks.rs` to loop over `Framework::ALL` instead of string slices. Add tests: `Framework::ALL` has 7 variants, `name()`/`parse()` round-trip, Rails reads `ruby_version` from runtime.

- [ ] Update tests in `infra/assets/frameworks.rs` to loop over `Framework::ALL` instead of `framework_names()`. Rebind `framework` variable from `String` to `Framework` in the `every_framework_has_a_build_environment_example` test.

## Validation

- [ ] Run `cargo test -p bonesdeploy` — all framework-related tests pass, no regressions in integration tests.

- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings` — no warnings.

- [ ] Run `cargo fmt` — no formatting changes.

- [ ] Run `shfmt -w .` — no formatting changes.

- [ ] Grep the codebase for `use crate::frameworks::` (not `use crate::frameworks;`) outside `frameworks.rs` — no direct import of per-framework modules.

- [ ] Grep for `frameworks::questions(`, `frameworks::validate_answers(`, `frameworks::configure(`, `frameworks::environment_example(` — no remaining free-function calls (the module-level dispatch functions are removed).

## Completion

- [ ] Final diff review: verify the `Framework` enum is the single architectural front door, per-framework modules are `pub(super)`, and no callers bypass the enum methods.

## Completion notes
- (To be filled after implementation)