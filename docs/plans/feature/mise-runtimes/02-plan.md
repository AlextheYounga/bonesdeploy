# Plan

## Current behavior

`Runtime.node_version` and framework extras hold local language choices. Rails
offers exact Ruby choices and writes the selected value to `.env.build`. Django
retains a legacy `python_version` value but exposes no active version question
and builds with the pinned builder image's `/usr/bin/python3`. Node build scripts
resolve exact versions from `.env.build` and repository version files.

`crates/bonesdeploy/assets/kit/deployment/functions.sh` downloads Node archives
into the persistent build cache and compiles selected Ruby releases from source.
Django's build script installs distribution Python through APT, installs
requirements into `.python-packages`, and creates release launchers hard-coded
to `/usr/bin/python3`. Rails retains `vendor/bundle` in the native artifact.

PyInfra's `NodeRuntime` uses a second custom Node archive installer under
`/opt/bonesdeploy/node`. `RubyRuntime` and `PythonRuntime` validate configured
versions but install generic distribution packages and return `/usr/bin/ruby`
and `/usr/bin/python3`. Rails prepare and service commands therefore use
`/usr/bin/bundle`; Django launchers use `/usr/bin/python3`. Framework AppArmor
profiles encode those paths.

The site provisioning request already transports framework extras and a Node
version, although Python request parsing currently omits `site.node_version`
from `RuntimeConfig.data`. The deliberately narrow BonesRemote deployment
descriptor drops language build configuration. That descriptor need not control
language installation because PyInfra provisions the site runtime before
artifact deployment.

PHP follows a separate, working design: Laravel's local build installs selected
APT PHP CLI packages, while PyInfra provisions the Sury repository, PHP-FPM,
extensions, pools, sockets, and service integration.

## Intended behavior

Project configuration is the single source of the exact Ruby, Python, and Node
versions used by both build and provisioning. New Django projects select exact
Python `3.14.0` by default. Rails retains its supported exact Ruby releases and
Node retains exact patch versions. Managed build environment values are derived
from this configuration rather than independently editable copies.

The pinned mise `2026.10.0` binary installs precompiled runtimes into the local
project build cache and the root-owned production runtime store. Build and
production disable compilation fallback and floating registry behavior. PyInfra
creates stable per-site runtime links and renders direct executable paths into
services and security policy; runtime services never invoke mise.

Rails bundles gems locally with the managed Ruby and packages the Bundler needed
by the lockfile alongside the application bundle. Production uses the managed
Ruby and packaged Bundler for bundle validation, Puma, and migrations. Django
installs requirements locally with the managed Python and emits launchers that
resolve the site's managed Python link. Production uses those launchers for
Gunicorn, checks, migrations, and static collection. Runtime identity markers
cause prepare to fail before migrations when an artifact and provisioned site
runtime disagree.

Node build scripts use mise while retaining existing version-file resolution,
Corepack selection, lockfile rules, package-manager commands, persistent pnpm
store, and output pruning. Dynamic Next, Nuxt, and SvelteKit services use their
site's managed Node link. Static Node-built frameworks install no production
Node runtime unless their selected mode already requires one.

PHP and PHP-FPM behavior remain unchanged outside shared tests and documentation
assertions proving that the mise migration does not encompass them.

## Approach

Add a small shared mise runtime policy containing the pinned mise version,
verified installation source, managed data/cache locations, exact-version
validation, and precompiled-only environment. Local build helpers and PyInfra
language services consume that policy without loading project mise files.

In the local Docker runner, bootstrap the verified mise binary into the existing
project cache and set explicit mise data, cache, and config directories below
`BUILD_CACHE_DIR`. Replace custom Node and Ruby installation functions with
mise-backed exact installers, and add the equivalent Python helper. Keep current
Node version-source precedence at the BonesDeploy boundary, but pass the final
exact version to mise instead of delegating project discovery to mise.

Make the managed root configuration authoritative for runtime versions. Django
gains an exact Python framework default and question; generated build examples
no longer create an independent version decision. Build-contract environment
projection supplies selected framework versions to managed scripts. Site
provisioning preserves Ruby/Python extras and correctly projects Node into the
Python runtime context. The narrow deployment descriptor remains limited to
values BonesRemote itself consumes.

On production, PyInfra installs the verified mise binary and exact runtimes into
a root-owned global store. It creates stable links below each site's project
root, validates exact versions as the runtime user, and returns the linked
executable path to framework orchestration. Shared runtime installations are
immutable inputs; site deletion removes links, not globally shared versions.

Update Rails to invoke managed Ruby directly and package the lockfile's exact
Bundler implementation during local build, avoiding reliance on distribution
Bundler. Update Django launchers to calculate their site root and execute the
managed Python link while preserving release-local `.python-packages`. Add
artifact runtime markers and validate them at the start of prepare before
application checks or migrations.

Update framework runtime commands, placeholder setup, validation, manifests,
and AppArmor to use the stable site links. Placeholder dependency installation
remains limited to site setup and uses the managed runtime; deployment prepare
remains network-free and compilation-free. Replace stale tests and documentation
that describe distribution Ruby/Python or custom Node installation, regenerate
the embedded BonesInfra wheel, and leave PHP implementation files unchanged.

## Responsibilities and boundaries

- `bonesdeploy-core` owns exact runtime configuration, safe build-environment
  projection, provisioning transport, and artifact runtime identity types.
- `bonesdeploy` framework definitions own supported defaults and questions;
  build helpers own local mise bootstrap, cache use, version resolution, and
  application dependency construction.
- `bonesinfra.services.languages` owns verified production mise installation,
  the global runtime store, precompiled-only runtime installation, exact-version
  validation, and stable site runtime links.
- Framework runtime modules own the executable each service, placeholder, and
  validation command uses. Their AppArmor templates own the corresponding
  least-privilege runtime access.
- Framework build and prepare scripts own runtime markers and pre-mutation
  compatibility checks for their artifacts.
- BonesRemote continues to own safe artifact receipt and prepare execution, but
  it does not install runtimes or broaden its deployment configuration model.
- Existing PHP and Laravel runtime modules remain solely responsible for PHP.

## Affected areas

- Runtime configuration and transport under
  `crates/bonesdeploy-core/src/config/` and their focused tests.
- Framework defaults and generated environment examples under
  `crates/bonesdeploy/src/frameworks*` and `crates/bonesdeploy/assets/frameworks/`.
- Shared local build helpers in
  `crates/bonesdeploy/assets/kit/deployment/functions.sh` and Rails, Django, and
  Node-framework build and prepare assets.
- PyInfra request parsing, paths, language services, framework runtime modules,
  manifests, systemd command inputs, AppArmor templates, and tests under
  `crates/bonesinfra/python/`.
- Embedded BonesInfra wheel and Rust asset/materialization tests under
  `crates/bonesinfra/`.
- Rails and Django prepare-contract tests under `crates/bonesremote/tests/`.
- Django and Rails E2E fixtures and assertions, authored but not agent-executed.
- `README.md`, `CONTEXT.md`, `crates/bonesinfra/python/CONTEXT.md`,
  `docs/ARCHITECTURE.md`, and `docs/architecture/reference.md`.
- Earlier Ruby/Python distribution-runtime planning records receive explicit
  superseding clarifications while this feature plan remains authoritative.

## Decisions

- Mise is an implementation detail of runtime installation, not a replacement
  for PyInfra and not part of application service execution.
- Mise `2026.10.0` is pinned because managed build and production environments
  require reproducible installer behavior; updates occur through reviewed code
  changes rather than self-update.
- Production and build require precompiled runtimes. Silent source fallback
  would reintroduce resource exhaustion and make runtime provenance differ.
- Stable per-site links decouple service and artifact paths from mise's internal
  store layout while allowing exact runtime versions to be shared safely.
- Project BonesDeploy configuration, not project mise files, is authoritative.
  This preserves existing validation and prevents production from trusting
  application-defined hooks or installer settings.
- The narrow BonesRemote descriptor remains narrow. Runtime selection belongs to
  site provisioning; artifact markers provide deployment-time compatibility
  checks without restoring arbitrary framework configuration to the control
  plane.
- Existing Node version-file precedence remains compatible at the local build
  boundary; mise receives only the resolved exact version.
- Bundler is packaged with the Rails artifact because distribution or
  runtime-bundled Bundler versions need not match `Gemfile.lock`.
- PHP stays on its current APT, Sury, and PHP-FPM path because mise's PHP backend
  compiles from source and does not provide the production integration already
  working in BonesInfra.

## Risks

- Mise or an upstream runtime may lack a precompiled artifact for a configured
  version; strict mode must fail clearly rather than compiling or selecting a
  different release.
- Ruby/Python native extensions can still depend on shared system libraries.
  The Debian 12 build baseline and PyInfra runtime libraries must remain
  compatible across supported Debian and Ubuntu hosts.
- Mise's internal install layout may change. Only the installer boundary may
  depend on it; services and artifacts use stable site links.
- Shared runtime stores create concurrency and ownership concerns when multiple
  site setups install the same release. Installation must be idempotent,
  root-owned, and atomic from runtime users' perspective.
- Changing Django from a major/minor legacy value to an exact patch value can
  invalidate existing configuration. This breaking feature deliberately updates
  generated projects and must fail old ambiguous values with a clear migration
  message rather than silently selecting a patch release.
- Packaged Bundler invocation and site-linked Python launchers can be blocked by
  AppArmor unless executable and library paths are covered narrowly.
- Replacing the working custom Node installer could regress Corepack or
  version-file behavior; focused tests must preserve those contracts.
- Shared language abstractions could accidentally alter PHP. Diff review and PHP
  regression tests must prove that its installer and service behavior remain
  unchanged.

## Validation

- Focused Rust configuration tests prove exact Ruby, Python, and Node versions
  reach local build and provisioning while deployment descriptors remain narrow.
- Shell asset tests prove mise is pinned, verified, cache-backed, and
  precompiled-only; obsolete Ruby source compilation and custom Node download
  paths are absent.
- Python unit tests prove PyInfra installs shared managed runtimes idempotently,
  creates stable site links, returns direct executables, and leaves PHP package,
  FPM, extension, pool, and socket behavior unchanged.
- Rails tests prove the build packages `vendor/bundle`, exact Bundler, and Ruby
  identity; prepare rejects mismatches before migrations and never installs
  gems; Puma commands and AppArmor use managed runtime paths.
- Django tests prove requirements install under the exact managed Python,
  launchers use the site runtime link, prepare rejects mismatches before Django
  commands, and Gunicorn, systemd, and AppArmor use the managed runtime contract.
- Node tests prove existing exact-version resolution, Corepack/package-manager
  behavior, framework output pruning, and dynamic service commands survive the
  installer replacement.
- The regenerated BonesInfra wheel matches Python source, focused asset and
  lifecycle tests pass, and all non-E2E Rust/Python tests, Clippy, Ruff, Rustfmt,
  Shfmt, generated-artifact checks, and `git diff --check` complete cleanly.
- Human-run Rails, Django, and dynamic Node E2E deployments demonstrate that
  build and production report identical exact runtime versions and activate
  without production runtime or application compilation.
