# Idea

## Request

Incorporate Rusty Object Notation (RON) into `bonesdeploy-core` so that the
infrastructure paths, application defaults, runtime defaults, build defaults,
service defaults, and critical security defaults have one readable,
topic-oriented source of truth under `crates/bonesdeploy-core/specs`.

## Problem

Core path constants and configuration defaults are currently scattered through
Rust modules, and `crates/bonesdeploy/assets/kit/bones.toml` repeats initial
configuration defaults. Reviewing the infrastructure decisions requires
scanning implementation code, and a changed default can diverge between the
Core model and the scaffolded kit.

## Definitions

**Core specification:** A typed RON document embedded in `bonesdeploy-core`
that defines static infrastructure values or default configuration values.
Every RON struct value includes its Rust struct name so the object identifies
its subject at the point of definition. A Core specification is compiled into
the binaries and is not a user-editable deployment configuration file.

**Project configuration:** The `.bones/bones.toml` file written for an
individual project. It remains TOML, is owned by the project, and contains
project-specific values selected during `bonesdeploy init`.

**Shared infrastructure decision:** A static value used by Core and another
crate that governs filesystem locations, executable names, system locations,
or a default deployment policy. It is represented only by the Core
specification after this change.

**Release permission default:** The default path rule applied during release
promotion: directories have mode `0750` and files have mode `0640`. These
rules are typed Core configuration, not untyped TOML.

## Desired outcome

Each binary embeds the topic-oriented Core RON specifications. `bonesdeploy
init` creates a project `bones.toml` from the typed Core defaults plus values
collected for that project. The static kit `bones.toml` no longer exists.
Changing an infrastructure path or a configuration default requires changing
one RON specification, and the existing generated project configuration and
runtime behavior retain their current defaults.

## Scope

This change adds typed RON specifications for Core paths, application,
runtime, build, and service defaults; migrates every static value in
`bonesdeploy-core/src/paths.rs` and Core configuration default into those
specifications; replaces untyped release permission values with typed
configuration; updates Core consumers and `bonesdeploy init`; and removes the
duplicated kit `bones.toml`.

The change also replaces duplicated asset values only where they represent a
shared infrastructure decision owned by a Core specification.

## Constraints

RON files live under `crates/bonesdeploy-core/specs` and are embedded with
`include_str!`. Core deserializes them into typed specifications when defaults
are requested. Every RON struct value uses named struct syntax rather than an
anonymous parenthesized object. Project configuration remains TOML. The
generated configuration keeps the existing observable defaults, including Node
`24.18.0`, build timeout `300`, SSH user `root`, SSH port `22`, branch `master`,
five retained releases, and the release permission default.

The change must use the project Rust test framework and pass `cargo clippy`,
`cargo fmt`, and `shfmt -w .`. End-to-end tests are excluded from local
validation.

## Exclusions

This change does not change deployment behavior, user-editable TOML schema,
operational shell-script behavior, Nginx template behavior, hook behavior, or
framework-specific configuration. It does not move asset files out of the kit
or framework asset directories. Literals local to those assets remain there
unless they duplicate a shared infrastructure decision owned by Core.
