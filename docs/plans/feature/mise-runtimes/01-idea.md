# Idea

## Request

Use PyInfra to install mise, then use the same pinned mise release to manage
exact Ruby, Python, and Node runtimes in both local Docker builds and production.
PHP remains an outlier because its existing APT and PHP-FPM integration works
correctly and must not change.

## Problem

Ruby and Python currently use different local-build and production runtimes.
Rails builds with an exact source-built Ruby but production runs the host's
distribution Ruby. Django builds dependencies with the builder image's Python
but production runs the host's distribution Python. Native gems and Python
extensions are interpreter-ABI-specific, so complete local artifacts cannot be
relied on across those mismatched runtimes. The current Rails deployment exposes
this defect by shipping `vendor/bundle` for Ruby 3.3 while production Bundler
searches the path for another Ruby ABI.

Node already installs exact upstream binaries in both environments, but it uses
separate custom installers and path conventions. Keeping a second runtime
installation mechanism after adopting mise for Ruby and Python would preserve
unnecessary code and prevent a common runtime contract.

## Definitions

**Managed runtime:** An exact Ruby, Python, or Node release installed from a
precompiled artifact by the BonesDeploy-pinned mise release. A managed runtime
does not include application dependencies, package-manager caches, PHP, or
operating-system libraries.

**Runtime contract:** The selected language name, exact semantic version,
`linux/amd64` platform, and mise installation policy shared by local build and
production. Ruby and Python artifacts additionally record enough interpreter
identity to reject a mismatched production runtime before state-changing
prepare work.

**Runtime store:** A root-owned, host-global directory containing immutable,
versioned mise installations that may be shared by multiple sites. Each native
site receives stable read-only links to only the managed runtimes it uses.

**Precompiled-only installation:** Mise must install a published binary runtime
or fail. It may not fall back to compiling Ruby, Python, or Node on a local build
machine or production host. This does not prohibit local compilation of
application dependencies such as native gems or Python extension modules.

## Desired outcome

Rails and Django artifacts are built and executed with the same exact managed
Ruby or Python version. Dynamic Node applications likewise build and execute
with the same exact managed Node version. A production host installs these
runtimes without compiling them, and services invoke stable absolute runtime
paths without shell activation, shims, or ambient user configuration.

Rails prepare validates the packaged bundle and runs migrations without
installing gems. Django prepare uses the packaged dependency tree and runs its
existing production-state operations without installing packages. Static and
server-side Node framework artifact contracts remain unchanged apart from the
runtime installer and executable paths.

## Scope

- Pin and verify one mise release for local Docker builds and PyInfra-managed
  production hosts.
- Replace the custom local Ruby source build, local Django distribution Python,
  local Node archive installer, and corresponding production runtime installers
  with precompiled-only mise installations.
- Make exact Ruby and Python versions first-class framework configuration and
  keep exact Node version configuration authoritative.
- Give local builds and PyInfra provisioning the same runtime version inputs,
  stable cache/store policy, and direct executable paths.
- Update Rails and Django artifact launchers, prepare scripts, framework runtime
  commands, validation, systemd inputs, manifests, and AppArmor rules for the
  managed runtimes.
- Preserve local Bundler, pip, npm, pnpm, Yarn, and Corepack application
  dependency behavior while removing obsolete language-runtime installers.
- Add runtime identity validation before incompatible Rails or Django artifacts
  can mutate production state or activate.
- Update generated infrastructure assets, focused tests, architecture records,
  framework documentation, and E2E fixtures for human validation.

## Constraints

- PyInfra remains responsible for installing mise, operating-system packages,
  runtime stores and links, systemd, AppArmor, users, directories, and services.
- Mise manages only Ruby, Python, Node, and their runtime-adjacent executable
  selection. It does not replace PyInfra or the application package managers.
- Mise is pinned to version `2026.10.0`; automatic self-update and floating
  registries are disabled in managed environments.
- Ruby, Python, and Node versions are exact patch releases. The Django default is
  Python `3.14.0`; existing exact Rails and Node defaults remain unchanged.
- Ruby, Python, and Node source-build fallback is disabled. Missing precompiled
  artifacts fail setup or build clearly.
- Local builds remain isolated `linux/amd64` Docker builds using the pinned
  Debian 12 builder baseline, project cache mount, explicit non-secret build
  environment, and no production credentials.
- Application dependencies and native extensions are installed or compiled only
  during local artifact construction. Remote prepare may validate and perform
  production-state work but may not build the application.
- Production services use direct root-controlled runtime paths. They do not run
  `mise exec`, mise shims, shell activation, or project-owned mise configuration.
- Existing Debian 12+ and Ubuntu 24.04+ production support remains in force.
- E2E tests are written for human execution and are not run by an agent unless
  explicitly requested.

## Exclusions

- Any change to PHP version selection, PHP APT repositories, PHP-FPM, PHP
  extensions, Composer behavior, Laravel runtime provisioning, or Laravel's PHP
  service integration. Laravel may continue using managed Node for its optional
  local frontend build.
- Using mise as a replacement for PyInfra, system package management, systemd,
  AppArmor, Nginx, databases, or release lifecycle management.
- Installing application dependencies through mise package backends.
- Interactive shell integration, per-user runtime managers, automatic runtime
  upgrades, or production use of application-owned `mise.toml` files.
- Production compilation of language runtimes or application dependencies.
- Changes to Docker Compose application images, whose runtimes remain owned by
  their images.
