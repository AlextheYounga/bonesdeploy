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

BonesInfra installs the host distribution's `ruby`, `ruby-dev`, and
`ruby-bundler` packages plus the native build dependencies used by supported
Rails database adapters. Puma, placeholder setup, validation, and migrations
use `/usr/bin/ruby` and `/usr/bin/bundle`.

The local build retains its selected exact Ruby solely to install build-time
gems and precompile assets, then removes `vendor/bundle` before artifact
packaging. During remote prepare, Bundler installs production gems into the
staged release's `vendor/bundle` before migrations. A Ruby incompatibility or
bundle failure prevents release activation. Existing persisted `ruby_version`
values remain valid as local-build configuration but no longer select the
production interpreter.

## Approach

Replace the host source installer with a PyInfra APT package operation in
`RubyRuntime` and return the stable distribution executable. Keep the local
Ruby build toolchain unchanged because it operates on the developer's build
machine and is still needed for asset compilation. Change the Rails build
cleanup to remove its local `vendor/bundle`.

Extend the Rails prepare script to run `/usr/bin/bundle install` with production
groups excluded and `BUNDLE_PATH=vendor/bundle`, then run migrations through
the same Bundler command. Remove the obsolete Ruby-version environment
projection from BonesRemote prepare. Update runtime path expectations, tests,
and documentation, with a dedicated README explanation of the production Ruby
and deployment-time bundle contract.

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

Production uses only the host distribution's Ruby packages. This avoids both
the resource cost of compiling on production and an additional package
repository trust boundary, at the cost of making the available Ruby version a
property of the host OS.

Application gems are installed on the target because locally compiled native
extensions cannot be safely reused across the supported distribution Ruby ABIs.
The selected exact Ruby remains a local-build setting so existing projects and
asset builds retain their current behavior. It is not represented as the
production Ruby version in user-facing documentation.

## Risks

Deployments now require access to configured gem sources and can take longer
while Bundler downloads or compiles gems. The production host's Ruby may not
satisfy an application's Ruby or Rails requirement; Bundler must fail before
activation and leave the current release running. Native gem installation
requires compiler and database client development packages on the production
host. Removing `vendor/bundle` after local asset compilation must not remove
precompiled public assets.

## Validation

Focused Python tests prove the distribution package set and executable paths.
Rust asset tests prove the local bundle is removed and remote prepare installs
the target bundle before migrations. BonesRemote tests prove the obsolete Ruby
version environment projection is absent. Existing exact-version local-build
tests remain green. Python tests, Ruff, Cargo formatting, Clippy, shell
formatting, and the relevant Rust test targets must pass. The full E2E suite is
excluded.
