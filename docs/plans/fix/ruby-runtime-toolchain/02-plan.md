# Plan

## Current behavior

`RubyRuntime` in `crates/bonesinfra/python/src/bonesinfra/services/languages/ruby.py`
accepts an `X.Y` value and installs `ruby<X.Y>` and `ruby<X.Y>-dev` with APT,
then returns `/usr/bin/ruby<X.Y>`. Rails exposes `3.2`, `3.3`, and `3.4` as
choices. The Rails build script independently installs the same invalid APT
package names. The E2E fixture requests `3.4` and uses a Rails application
created with Ruby 3.4.8. Node is installed independently from a verified
archive and cached separately for host and build contexts.

## Intended behavior

Rails selects one of the supported exact Ruby releases. BonesInfra installs that
release from a checksum-verified official Ruby archive under
`/opt/bonesdeploy/ruby/<version>` and returns its `bin/ruby` path. The Rails
build script installs the same selected release into the build cache and runs
Bundler through that versioned path. Puma, prepare scripts, and host validation
use the executable returned by the Ruby runtime.
Existing `X.Y` values resolve to their corresponding supported exact release.

## Approach

Add a focused Ruby installer script beside the existing Node installer. It
validates the selected version against a finite supported-release checksum map,
installs native compilation packages with APT, downloads the official archive,
verifies its SHA-256 hash, compiles it under a versioned prefix, and confirms
the installed interpreter version. RubyRuntime invokes this installer and
returns its versioned executable. Rails build functions gain the equivalent
cache-local installation logic and the Rails build script uses it instead of
APT Ruby packages. Rails configuration choices and the E2E fixture use exact
versions.

## Responsibilities and boundaries

The BonesInfra language service owns host runtime installation and executable
paths. Its installer asset owns download, checksum, compilation, and
idempotency mechanics. The shared deployment functions own build-cache Ruby
installation and activation. The Rails framework owns allowed configuration
values and Rails-specific invocation. Tests remain in the existing Rust,
Python, and shell-adjacent test suites.

## Affected areas

- `crates/bonesinfra/python/src/bonesinfra/services/languages/ruby.py`
- `crates/bonesinfra/python/src/bonesinfra/assets/scripts/install-ruby.sh`
- `crates/bonesinfra/python/tests/test_languages.py`
- `crates/bonesdeploy/src/frameworks/rails.rs`
- `crates/bonesdeploy/tests/config_frameworks.rs`
- `crates/bonesdeploy/assets/kit/deployment/functions.sh`
- `crates/bonesdeploy/assets/frameworks/rails/deployment/build/02_run_build.sh`
- `e2e/tests/setup/rails.rs`
- Rails and project documentation that state Ruby support.

## Decisions

Ruby uses official source releases instead of an external APT repository or
unversioned distro packages. This honors the configured version on all
supported distributions and avoids adding a repository trust boundary. The
supported releases form a finite checksum map so provisioning never accepts an
unverified user-selected archive. Installations are versioned and do not create
or replace `/usr/bin/ruby` symlinks, which permits different sites to coexist.
Legacy `X.Y` configuration values are accepted because existing project
configuration is persisted; new project configuration writes exact values.

## Risks

Source compilation increases first-run setup time and requires build tools.
The installer must clean temporary data and fail before installation on checksum
or version validation errors. Build execution must activate the cached Ruby
before invoking `bundle`; otherwise it could silently use the container Ruby.

## Validation

Focused Python tests prove Ruby validation, installer invocation, and executable
paths. Rust tests prove Rails uses exact Ruby build settings. Shell formatting
validates the installers. Python tests, Ruff, Cargo formatting, Clippy, and
the relevant Rust test targets must pass. The full E2E suite is excluded.
