# Distribution Ruby Runtime

<!-- Merged from fix/ruby-runtime-toolchain -->

## Idea

### Request

Install the production Rails Ruby runtime from the supported Debian or Ubuntu
distribution instead of compiling Ruby during site setup. Document this runtime
and deployment model clearly in the README.

### Problem

Rails site setup currently compiles a selected exact Ruby release from source.
That build can exhaust the memory of a 512 MiB production server and make the
server unresponsive. Shipping the application bundle produced by the local
Debian build container is not a safe substitute because supported production
hosts can provide a different Ruby ABI through their distribution packages.

### Definitions

**Distribution Ruby:** The default `ruby`, `ruby-dev`, and `ruby-bundler`
packages supplied by the production host's configured Debian or Ubuntu APT
repositories.

**Target bundle:** The production gems installed into `vendor/bundle` inside a
staged release by the production host's Ruby and Bundler before that release is
activated.

### Desired outcome

Rails site setup installs Ruby and Bundler through APT without compiling Ruby.
Each deployment creates its target bundle on the production host before running
migrations or activating the release, so native gems match the host's Ruby ABI.
The README makes the host-controlled Ruby version, deployment-time RubyGems
access, and application compatibility requirement explicit.

### Scope

This change includes distribution Ruby installation on production hosts,
distribution executable paths for Puma and Bundler, removal of locally compiled
gems from release artifacts, target bundle installation before migrations,
regression tests, and related documentation. The existing exact Ruby selection
continues to control the local asset build only.

### Constraints

Production Ruby packages must come from the host's configured APT repositories.
Target bundle installation must run as the site's runtime user while the staged
release is writable, and deployment must stop before activation when Bundler or
the application's Ruby requirements fail. Full E2E tests must not be run during
this work.

### Exclusions

This change does not add a third-party Ruby repository, arbitrary production
Ruby-version installation, Ruby version managers, support for non-Debian/Ubuntu
hosts, vendored Ruby gems, or changes to Rails application source code.


## Plan

### Current behavior

Before this change, `RubyRuntime` invoked `install-ruby.sh`, which downloaded a checksum-pinned Ruby
source archive, installs compiler dependencies, and builds the selected release
under `/opt/bonesdeploy/ruby/<version>`. Rails runtime commands and remote
migrations use that versioned installation.

The local Rails build independently compiled the same selected Ruby into the
build cache, installs production gems into `vendor/bundle`, and precompiles
assets. The complete build tree, including that bundle, is packaged and copied
to the production host. Production supports Debian 12+ and Ubuntu 24.04+, whose
distribution Ruby versions can differ from the build Ruby and from each other.

### Intended behavior

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

### Approach

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

### Responsibilities and boundaries

The BonesInfra language service owns host APT package installation and the
production Ruby executable path. The shared deployment functions continue to
own the local build-cache Ruby toolchain. The Rails local build owns removal of
build-only gems, while the Rails prepare script owns target bundle installation
and migrations. BonesRemote owns the generic prepare execution boundary and no
longer projects a managed Ruby path. Tests remain in the existing Rust, Python,
and shell-adjacent test suites.

### Affected areas

- `crates/bonesinfra/python/src/bonesinfra/services/languages/ruby.py`
- `crates/bonesinfra/python/tests/test_languages.py`
- `crates/bonesdeploy/assets/frameworks/rails/deployment/build/02_run_build.sh`
- `crates/bonesdeploy/assets/frameworks/rails/deployment/prepare/01_prepare_rails.sh`
- `crates/bonesdeploy/tests/assets.rs`
- `crates/bonesremote/src/release/lifecycle/prepare.rs`
- `crates/bonesremote/tests/release/lifecycle_prepare.rs`
- Rails AppArmor and runtime path tests
- `README.md`, `CONTEXT.md`, and architecture documentation that state Ruby support

### Decisions

Production uses only the host distribution's Ruby packages. This avoids both
the resource cost of compiling on production and an additional package
repository trust boundary, at the cost of making the available Ruby version a
property of the host OS.

Application gems are installed on the target because locally compiled native
extensions cannot be safely reused across the supported distribution Ruby ABIs.
The selected exact Ruby remains a local-build setting so existing projects and
asset builds retain their current behavior. It is not represented as the
production Ruby version in user-facing documentation.

### Risks

Deployments now require access to configured gem sources and can take longer
while Bundler downloads or compiles gems. The production host's Ruby may not
satisfy an application's Ruby or Rails requirement; Bundler must fail before
activation and leave the current release running. Native gem installation
requires compiler and database client development packages on the production
host. Removing `vendor/bundle` after local asset compilation must not remove
precompiled public assets.

### Validation

Focused Python tests prove the distribution package set and executable paths.
Rust asset tests prove the local bundle is removed and remote prepare installs
the target bundle before migrations. BonesRemote tests prove the obsolete Ruby
version environment projection is absent. Existing exact-version local-build
tests remain green. Python tests, Ruff, Cargo formatting, Clippy, shell
formatting, and the relevant Rust test targets must pass. The full E2E suite is
excluded.


## Tasks

### Implementation

- [x] Replace host Ruby source compilation with distribution Ruby, Bundler, and
  native gem build packages installed through PyInfra's APT operation.
- [x] Use distribution Ruby and Bundler paths throughout Rails runtime setup,
  placeholder installation, validation, systemd, and AppArmor configuration.
- [x] Remove the local build's `vendor/bundle` before artifact packaging while
  retaining precompiled assets.
- [x] Install the production bundle on the staged target release before running
  migrations, and remove the obsolete managed-Ruby environment projection.
- [x] Update focused Python and Rust regression coverage for the new host package,
  executable-path, artifact, and prepare behavior.

### Validation

- [x] Run focused Python and Rust tests for Ruby and Rails runtime, build, and
  prepare behavior.
- [x] Run the non-E2E Python and Rust suites, Ruff, Python formatting, Cargo
  formatting, Clippy, shell formatting, and `git diff --check` without errors.

### Completion

- [x] Explain distribution Ruby, host compatibility, target bundle installation,
  and deployment network/build requirements in the README.
- [x] Update applicable architecture/context documentation and review the final
  diff for stale managed-Ruby paths or source-install behavior.

### Completion notes

The earlier exact source-built production Ruby implementation was superseded by
the distribution Ruby decision recorded in `04-distribution-ruby-clarity.md`.
Rails setup now installs distribution Ruby and native gem build packages through
APT. Local Rails builds discard `vendor/bundle`, and remote prepare installs the
target bundle even when migrations are explicitly skipped.

`uv run ruff check .`, `uv run ruff format --check .`, and `uv run pytest` (518
tests) passed. `cargo test --workspace --exclude e2e`, `cargo clippy`, `cargo
fmt`, `shfmt -w .`, focused Rails/Ruby tests, and `git diff --check` passed. The
embedded BonesInfra wheel was rebuilt. Full E2E was not run.


## Distribution Ruby Clarification

### Trigger

Compiling Ruby during Rails site setup exhausted the resources of a 512 MiB
DigitalOcean server. The user directed production setup to install the Debian or
Ubuntu distribution Ruby through APT and requested that the README make the
resulting behavior explicit.

### Decision

Production Rails sites use the host distribution's Ruby, Ruby development
package, and Bundler package. Because supported hosts can expose different Ruby
ABIs, deployment does not ship the application bundle produced by the local
build. The staged release installs its production bundle on the target host
before migrations and activation. The existing exact Ruby selection remains
limited to the local asset build.

The README explicitly documents that the host OS controls the production Ruby
version, deployments require access to configured gem sources and may compile
native extensions, and applications must support the Ruby version supplied by
their selected production distribution.

### Supersedes

This supersedes the decision to compile and install the selected exact Ruby
release on production hosts and to ship the local build's `vendor/bundle` as the
production bundle. It does not supersede the exact Ruby toolchain used for local
asset compilation.

### Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` are updated to describe the
distribution Ruby runtime, target bundle installation, current execution state,
and required validation.
