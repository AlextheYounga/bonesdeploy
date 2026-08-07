# Plan

## Current behavior

`bonesdeploy-core/src/paths.rs` exports static path and infrastructure
constants along with functions that derive project- and site-specific paths.
`bonesdeploy-core/src/config.rs` and `src/app.rs` independently define
application, runtime, and build defaults. `Runtime.permissions` is currently
an `Option<toml::Value>`.

`crates/bonesdeploy/assets/kit/bones.toml` repeats several defaults, including
the Node version, build timeout, and release permission rules. The kit is
embedded with `rust-embed` and copied by `kit::scaffold` during fresh project
initialization. `commands/init/mod.rs` subsequently serializes the collected
`Bones` value to the same `bones.toml`, making the static asset redundant.

Both `bonesdeploy` and `bonesremote` directly use `bonesdeploy_core::paths`.
The existing Core configuration loader deserializes project TOML and applies
derived defaults for SSH user and preview domain. `bonesdeploy::config::save`
serializes `Bones` to the project-owned TOML file.

## Intended behavior

`bonesdeploy-core` exposes typed embedded Core specifications loaded from RON
files grouped by paths, application defaults, runtime defaults, build defaults,
and service defaults. Each RON struct value names its corresponding Rust struct
instead of using anonymous `(...)` syntax. Every former static Core path value
is read from the paths specification, while existing path-building functions
continue to derive project- and site-specific paths from those values.

The Core `Bones`, `App`, `Runtime`, `Build`, and `Services` defaults read their
values from the appropriate typed specifications. Release permission rules are
a typed Core value and serialize to the existing TOML shape. Fresh initialization
creates the same project `bones.toml` through `bonesdeploy::config::save`
without a kit `bones.toml` asset.

## Approach

Add the `ron` dependency to `bonesdeploy-core` and add a focused `specs`
module. The module embeds each RON document with `include_str!`, deserializes
the documents into topic-specific structs, and exposes the typed values to
Core. It returns contextual errors for malformed embedded specifications rather
than masking specification errors.

Define `paths.ron`, `application_defaults.ron`, `runtime_defaults.ron`,
`build_defaults.ron`, and `service_defaults.ron` under
`crates/bonesdeploy-core/specs`. Organize fields by the existing path and
configuration concepts, and use the corresponding Rust struct name for every
RON struct value. Retain path derivation in Rust because project and site
identifiers are runtime inputs.

Replace path constants and hard-coded default constructors with access through
the specifications. Update all `bonesdeploy`, `bonesremote`, and Core call
sites to use the typed paths API. Replace `Runtime.permissions` with typed
permission-rule structures that preserve the currently generated TOML.

Remove `assets/kit/bones.toml` and prevent the kit embedder from treating it as
an asset. Keep kit scripts, templates, hooks, and framework assets intact.
Fresh initialization continues to collect project values and writes the final
TOML once through the existing save boundary.

## Responsibilities and boundaries

`bonesdeploy-core/specs` owns the readable source documents.
`bonesdeploy-core/src/specs.rs` owns embedding, deserialization, and typed
access to Core specifications.
`bonesdeploy-core/src/paths.rs` owns path derivation and exposes path values
from the paths specification.
`bonesdeploy-core/src/config.rs` and `src/app.rs` own typed project
configuration defaults and validation.
`bonesdeploy/src/commands/init` owns project-specific value collection and
delegates final TOML output to `bonesdeploy/src/config.rs`.
`bonesdeploy/src/infra/assets/kit.rs` owns only remaining static kit assets.
`bonesdeploy` and `bonesremote` consume Core paths and defaults; they do not
define shared infrastructure decisions.

## Affected areas

- `crates/bonesdeploy-core/Cargo.toml`
- `crates/bonesdeploy-core/specs/*.ron`
- `crates/bonesdeploy-core/src/lib.rs`
- `crates/bonesdeploy-core/src/specs.rs`
- `crates/bonesdeploy-core/src/paths.rs`
- `crates/bonesdeploy-core/src/config.rs`
- `crates/bonesdeploy-core/src/app.rs`
- Core, `bonesdeploy`, and `bonesremote` Rust call sites that access migrated
  path constants or defaults
- `crates/bonesdeploy/assets/kit/bones.toml` for removal
- `crates/bonesdeploy/src/infra/assets/kit.rs`
- `crates/bonesdeploy/src/commands/init/tests.rs`
- Core configuration and path tests
- `README.md` if its configuration documentation needs to state the generated
  configuration source

## Decisions

- RON is embedded through `include_str!` instead of generated Rust source so
  infrastructure decisions remain readable in their native structured files.
- Specifications are split along the settled topic boundaries so a reader can
  inspect one category without scanning unrelated infrastructure decisions.
- RON struct values use named syntax such as `RuntimeDefaults(...)` and
  `PermissionRule(...)`; bare `(...)` does not identify the object's subject to
  a reader.
- Project TOML remains the public, editable project interface. RON is an
  internal Core source and does not replace project configuration.
- All former static Core path values migrate to one typed path specification;
  Rust retains only dynamic derivation operations.
- Release permissions use typed structs to make the security policy explicit,
  validate its shape through deserialization, and remove untyped TOML from Core
  defaults.
- The kit `bones.toml` is removed because initialization always writes the
  collected Core `Bones` value after scaffolding.

## Risks

- An omitted or malformed RON field can prevent defaults from loading. Tests
  must deserialize every embedded specification and assert its observable
  defaults.
- A RON type name that does not correspond to the deserialized Rust struct can
  make a specification misleading. The embedded-specification tests must parse
  every named document and nested object into its declared Rust type.
- Migrating a path field can change a filesystem location, executable name, or
  operating-system path. Existing path behavior must remain byte-for-byte
  equivalent for representative project and site identifiers.
- Typed release permissions can serialize differently from the existing inline
  TOML rules. Initialization tests must validate the saved TOML structure and
  modes.
- Broad Core path use across both binaries makes incomplete call-site migration
  a compile-time or behavior regression risk.

## Validation

- Add Core tests that deserialize the embedded RON documents and assert every
  migrated default, path derivation result, and typed release permission rule.
- Update initialization tests to verify fresh initialization creates
  `bones.toml` without a kit source file and that the saved TOML retains the
  existing Core defaults and typed permission entries.
- Run focused Core and `bonesdeploy` tests, followed by the workspace test
  suite excluding end-to-end tests.
- Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`.
- Review the final diff to ensure no duplicate Core path or default values
  remain in the removed kit configuration or Rust default constructors.
