# Plan

## Current behavior

Before this change, `RubyRuntime` invoked `install-ruby.sh`, which downloaded a checksum-pinned Ruby
source archive, installs compiler dependencies, and builds the selected release
under `/opt/bonesdeploy/ruby/<version>`. Rails runtime commands and remote
migrations use that versioned installation.

The local Rails build independently compiled the same selected Ruby into the
build cache, installs production gems into `vendor/bundle`, and precompiles
assets. The complete build tree, including that bundle, is packaged and copied
to the production host. Production supports Debian 12+ and Ubuntu 24.04+, whose
distribution Ruby versions can differ from the build Ruby and from each other.

## Intended behavior

Superseded by `feature/mise-runtimes`. Pinned mise installs the exact configured
Ruby precompiled-only for local builds and into the root-owned production store.
Puma and prepare use the stable site Ruby link. The artifact retains
`vendor/bundle` and lockfile-selected Bundler; prepare rejects runtime mismatch
before migrations and does not install gems.

## Approach

No further work is planned under this record. `feature/mise-runtimes` owns the
replacement implementation and validation.

## Responsibilities and boundaries

The BonesInfra language service owns host APT package installation and the
production Ruby executable path. The shared deployment functions continue to
own the local build-cache Ruby toolchain. The Rails local build owns removal of
build-only gems, while the Rails prepare script owns target bundle installation
and migrations. BonesRemote owns the generic prepare execution boundary and no
longer projects a managed Ruby path. Tests remain in the existing Rust, Python,
and shell-adjacent test suites.

## Affected areas

- `crates/bonesinfra/python/src/bonesinfra/services/languages/ruby.py`
- `crates/bonesinfra/python/tests/test_languages.py`
- `crates/bonesdeploy/assets/frameworks/rails/deployment/build/02_run_build.sh`
- `crates/bonesdeploy/assets/frameworks/rails/deployment/prepare/01_prepare_rails.sh`
- `crates/bonesdeploy/tests/assets.rs`
- `crates/bonesremote/src/release/lifecycle/prepare.rs`
- `crates/bonesremote/tests/release/lifecycle_prepare.rs`
- Rails AppArmor and runtime path tests
- `README.md`, `CONTEXT.md`, and architecture documentation that state Ruby support

## Decisions

The distribution-runtime decision is superseded. `feature/mise-runtimes` makes
the exact configured Ruby version authoritative for both build and production,
uses pinned mise `2026.10.0` with precompiled-only installation, and exposes a
stable site link. Rails packages application gems and the lockfile-selected
Bundler locally; production never installs or compiles them.

## Risks

The selected Ruby release must have a mise precompiled artifact. A runtime or
artifact identity mismatch must fail before activation and leave the current
release running.

## Validation

Validation is now defined by `feature/mise-runtimes`, including managed-runtime,
packaged-Bundler, direct-site-link, and human-run E2E checks.
