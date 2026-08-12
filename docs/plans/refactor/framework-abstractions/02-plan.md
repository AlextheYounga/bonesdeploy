# Plan

## Current behavior

### Framework identity

`crates/bonesdeploy/src/frameworks.rs` defines `Question`, `QuestionKind`, and
shared helper functions. It declares seven private submodules: `laravel`,
`django`, `next`, `nuxt`, `rails`, `sveltekit`, `vue`.

Five public dispatch functions each contain a full `match` on `&str` template
name:

- `questions(template: &str) -> Result<&'static [Question]>`
- `validate_answers(template: &str, answers: &Map<String, Value>) -> Result<()>`
- `configure(template: &str, cfg: &mut Bones)`
- `environment_example(template: &str, project_name: &str, domain: &str, preview_domain: &str) -> Option<String>`
- `build_environment_example(template: &str, runtime: &Runtime) -> Option<String>` (pub(crate))

The 7 template name strings (`"laravel"`, `"django"`, `"next"`, `"nuxt"`,
`"rails"`, `"sveltekit"`, `"vue"`) are duplicated across these 5 match
expressions plus 2 test functions.

### Per-framework modules

Each of the 7 modules exports a subset of these functions:

| Function | Visibility | Laravel | Django | Next | Nuxt | Rails | SvelteKit | Vue |
|---|---|---|---|---|---|---|---|---|
| `questions()` | `pub` | Y | Y | Y | Y | Y | Y | Y |
| `environment_example(project, url)` | `pub(crate)` | Y | Y | Y | Y | Y | Y | Y |
| `build_environment_example()` | `pub(crate)` | Y (w/ `&Runtime`) | Y (w/ `&Runtime`) | Y (no params) | Y (no params) | Y (no params + bug) | Y (no params) | Y (no params) |
| `configure(cfg)` | `pub(crate)` | N | N | Y | Y | N | N | N |

`build_environment_example` has two incompatible signatures:
- Laravel and Django accept `runtime: &Runtime` and read a language version from `runtime.extra`.
- Next, Nuxt, Rails, SvelteKit, and Vue take no parameters.

Rails asks for `ruby_version` (Choice: 3.2/3.3/3.4) but `build_environment_example()` hardcodes `DEFAULT_RUBY_VERSION` (3.3) instead of reading `runtime.extra["ruby_version"]`.

### Callers

All callers are within `crates/bonesdeploy/src/`:

| File | Functions called |
|---|---|
| `commands/init/framework.rs:58` | `frameworks::validate_answers(template_name, &user_vars)` |
| `commands/init/framework.rs:71` | `frameworks::questions(template_name)` |
| `commands/init/scaffold.rs:47` | `frameworks::configure(&template_name, cfg)` |
| `commands/secrets/mod.rs:61` | `frameworks::environment_example(template, project_name, domain, preview_domain)` |
| `infra/assets/frameworks.rs:52` | `frameworks::build_environment_example(framework, framework_config)` |
| `infra/assets/frameworks.rs:144` | `frameworks::build_environment_example(&framework, &Runtime::default())` (test) |

`commands/init/framework.rs` resolves template names from CLI flags or
interactive prompts and holds framework identity as `Option<String>` in a
`FrameworkSelection` struct.

`ui/prompts.rs` imports `Question` and `QuestionKind` types directly from
`crate::frameworks`.

### Framework name discovery

`infra/assets/frameworks.rs::framework_names()` derives framework names by
scanning the top-level directories of the embedded `assets/frameworks/`
filesystem at runtime. This is decoupled from the Rust source modules and has
no compile-time check for consistency.

### Tests

Tests live inline in `frameworks.rs` (`#[cfg(test)] mod tests`) and in
`infra/assets/frameworks.rs`. They cover: every template has questions and
environment examples, validation rejects bad input, configure overrides web_root
for static sites, build environment includes correct language versions, and
framework defaults fit the Runtime schema.

## Intended behavior

### Framework enum

A `pub enum Framework` with seven variants replaces the `&str` template name
as the canonical framework identity:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Framework {
    Laravel,
    Django,
    Next,
    Nuxt,
    Rails,
    SvelteKit,
    Vue,
}
```

It provides: a `parse()` / `from_str()` constructor for converting user input
to a `Framework` variant; an `ALL` constant for iteration; and a `name()`
method returning the lowercase string representation.

### Framework methods

All current dispatch functions become methods on `Framework`:

```rust
impl Framework {
    pub fn questions(&self) -> &'static [Question];
    pub fn validate_answers(&self, answers: &Map<String, Value>) -> Result<()>;
    pub fn configure(&self, cfg: &mut Bones);
    pub fn environment_example(&self, project_name: &str, domain: &str, preview_domain: &str) -> Option<String>;
}

// pub(crate)
impl Framework {
    pub(crate) fn build_environment_example(&self, runtime: &Runtime) -> Option<String>;
}
```

`configure()` remains `pub` but every variant dispatches through the method.
Five variants that previously had no `configure()` (Laravel, Django, Rails,
SvelteKit, Vue) return immediately — no-op, not special-cased in match.

`build_environment_example()` accepts `&Runtime` uniformly. Frameworks that
don't need it (Next, Nuxt, SvelteKit, Vue) ignore the parameter. Rails reads
`runtime.extra["ruby_version"]` instead of hardcoding `DEFAULT_RUBY_VERSION`.

### Caller changes

Callers convert the template string to `Framework` once at the user input
boundary, then pass the `Framework` value:

- `frameworks::questions(name)` → `framework.questions()`
- `frameworks::validate_answers(name, answers)` → `framework.validate_answers(answers)`
- `frameworks::configure(name, cfg)` → `framework.configure(cfg)`
- `frameworks::environment_example(name, ...)` → `framework.environment_example(...)`
- `frameworks::build_environment_example(name, runtime)` → `framework.build_environment_example(runtime)`

`FrameworkSelection.template` changes from `Option<String>` to
`Option<Framework>`.

### Per-framework module changes

Per-framework module functions become `pub(super)` — callable only from
`frameworks.rs` method bodies. Callers outside the `frameworks` directory only
interact with `Framework` enum methods.

The `rust-embed`-based `framework_names()` in `infra/assets/frameworks.rs` is
replaced by `Framework::ALL` or an equivalent constant/list method.

### Rail bug fix

Rails `build_environment_example` reads `runtime.extra.get("ruby_version")`
and falls back to a `DEFAULT_RUBY_VERSION` constant when absent, matching the
pattern already used by Laravel (PHP) and Django (Python).

## Approach

1. Introduce the `Framework` enum with seven variants above the existing
   submodule declarations in `frameworks.rs`. Add `ALL`, `name()`, and `parse()`.

2. Implement the five methods on `Framework`. Each method body contains a match
   on `self` that dispatches to the corresponding `pub(super)` function in the
   per-framework module. This is the same match pattern as today, but it's a
   single match per method instead of five separate match expressions on `&str`.

3. Change per-framework module function visibility from `pub`/`pub(crate)` to
   `pub(super)` so they are only accessible from `frameworks.rs`.

4. Unify `build_environment_example` signatures: all modules accept
   `runtime: &Runtime`. Next, Nuxt, Rails, SvelteKit, Vue modules gain a
   `_runtime: &Runtime` parameter that they ignore (Rails reads
   `extra["ruby_version"]` instead of hardcoding).

5. Convert callers:
   - `commands/init/framework.rs`: parse the template string into `Framework`
     once in `resolve_template()`. Store `Framework` in `FrameworkSelection`.
   - `commands/init/scaffold.rs`: pass `Framework` to configure.
   - `commands/secrets/mod.rs`: pass `Framework` to environment_example.
   - `infra/assets/frameworks.rs`: accept `Framework` in
     `scaffold_framework_env_build` and `build_environment_example` calls.
   - `ui/prompts.rs`: already only imports `Question`/`QuestionKind` types —
     no change needed.

6. Replace `framework_names()` in `infra/assets/frameworks.rs` with a call to
   `Framework::ALL` that maps through `.name()`. This removes the filesystem-based
   name derivation.

7. Update tests:
   - Adapt the 7 existing tests in `frameworks.rs` to use `Framework` enum
     instead of string template names.
   - Adapt the 9 tests in `infra/assets/frameworks.rs` to use `Framework`
     instead of loop over `framework_names()`.
   - Add a test that `Framework::ALL` contains all 7 variants.
   - Add a test that `Framework::parse(name)` round-trips through
     `Framework::name()` for every variant.
   - Add a test that Rails `build_environment_example` reads
     `runtime.extra["ruby_version"]` from the Runtime.

8. Pass `cargo clippy`, `cargo fmt`, `cargo test` for affected crates.

## Responsibilities and boundaries

**`frameworks.rs`** — owns the `Framework` enum, the `Question`/`QuestionKind`
types, the public method implementations, and the submodule declarations. It is
the single public front door for all framework-related behavior.

**Per-framework modules** (`frameworks/<name>.rs`) — own the concrete
implementation details: question arrays, environment example content, and
framework-specific configuration logic. These are implementation details behind
the `Framework` enum.

**`commands/init/framework.rs`** — owns the CLI-to-Framework conversion: parsing
`--template` flags, prompting for template selection, and collecting answers.
It converts the user's string choice to `Framework` once, then delegates.

**`commands/init/scaffold.rs`** — calls `Framework::configure()` and passes
`Framework` to asset scaffolding functions. It does not know which framework is
selected beyond having the enum value.

**`commands/secrets/mod.rs`** — calls `Framework::environment_example()`. It
does not dispatch on framework identity.

**`infra/assets/frameworks.rs`** — owns embedded framework asset scaffolding.
Receives a `Framework` from callers instead of a `&str`. Derives names from
`Framework::ALL` instead of filesystem scanning.

**`ui/prompts.rs`** — unchanged. Continues to import `Question`/`QuestionKind`
types and render interactive prompts. It is framework-agnostic.

## Affected areas

- `crates/bonesdeploy/src/frameworks.rs` — add `Framework` enum and methods
- `crates/bonesdeploy/src/frameworks/laravel.rs` — change visibility to `pub(super)`
- `crates/bonesdeploy/src/frameworks/django.rs` — change visibility to `pub(super)`
- `crates/bonesdeploy/src/frameworks/next.rs` — change visibility to `pub(super)`, add `_runtime` param
- `crates/bonesdeploy/src/frameworks/nuxt.rs` — change visibility to `pub(super)`, add `_runtime` param
- `crates/bonesdeploy/src/frameworks/rails.rs` — change visibility to `pub(super)`, add `_runtime` param, fix Ruby version
- `crates/bonesdeploy/src/frameworks/sveltekit.rs` — change visibility to `pub(super)`, add `_runtime` param
- `crates/bonesdeploy/src/frameworks/vue.rs` — change visibility to `pub(super)`, add `_runtime` param
- `crates/bonesdeploy/src/commands/init/framework.rs` — parse to `Framework`, change `FrameworkSelection`
- `crates/bonesdeploy/src/commands/init/scaffold.rs` — pass `Framework` instead of `&str`
- `crates/bonesdeploy/src/commands/init/mod.rs` — adapt to `FrameworkSelection` type change
- `crates/bonesdeploy/src/commands/secrets/mod.rs` — pass `Framework` to `environment_example`
- `crates/bonesdeploy/src/infra/assets/frameworks.rs` — accept `Framework`, replace `framework_names()`

## Decisions

1. **Enum, not trait.** A closed-set enum is simpler than a trait with 7 unit
   structs. We have 7 known frameworks; extensibility does not require an open
   set. Exhaustiveness checks catch missing dispatch arms at compile time. No
   trait object indirection or `dyn Framework` conversions needed.

2. **`pub(super)` visibility for per-framework module functions.** `pub(super)`
   restricts access to `frameworks.rs` only, which is the desired boundary —
   the enum methods are the public API, the per-module functions are
   implementation details. `pub(crate)` would allow direct bypass from other
   files within the crate.

3. **`build_environment_example` always takes `&Runtime`.** Consistent
   signature avoids the current two-signature awkwardness. Frameworks that
   don't use it (Next, Nuxt, SvelteKit, Vue) prefix the param with `_`. This
   is the minimal change that unifies the contract.

4. **`FrameworkSelection.template` changes from `Option<String>` to
   `Option<Framework>`.** This propagates the typed framework identity through
   the init command path instead of carrying a loose string. The `None` case
   (custom/no template) remains.

5. **`Framework::ALL` replaces `framework_names()`.** A const slice on the enum
   is the single source of truth. No filesystem dependency.

## Risks

- **Rail build_environment_example behavioral change.** Rails currently
  hardcodes `RUBY_VERSION=3.3`. The fix reads `runtime.extra["ruby_version"]`
  and falls back to the same `DEFAULT_RUBY_VERSION` (3.3). Existing projects
  with a `ruby_version` configured will now see it reflected in `.env.build`
  for the first time. This is a bug fix, not a regression, but the behavior
  change should be noted.

- **Missed callers.** The `Framework` enum replaces `&str` template names in
  function signatures, so the compiler will catch all missed call sites. No
  silent behavioral drift is possible.

## Validation

- `cargo test -p bonesdeploy` — the full bonesdeploy crate test suite, which
  includes all framework tests (`frameworks.rs` tests + `infra/assets/frameworks.rs`
  tests + `commands/init/framework.rs` tests + integration tests in `tests/`).
- Assert that `Framework::ALL` has 7 variants.
- Assert that `Framework::name()` round-trips through `Framework::parse()` for
  all 7 variants.
- Assert that `Framework::name()` matches the expected lowercase string for
  all 7 variants (used by TOML and CLI).
- Assert that Rails `build_environment_example` reads `ruby_version` from runtime.
- `cargo clippy --all-targets --all-features -- -D warnings` — no warnings.
- `cargo fmt` — no formatting changes.
- `shfmt -w .` — no formatting changes.
- Manual inspection of the final diff for boundary cleanliness: no direct
  `use crate::frameworks::laravel` or similar bypass outside `frameworks.rs`.