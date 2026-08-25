# Idea

## Request

Add the Rails Ruby runtime corrections identified while diagnosing the failed
Rails E2E setup.

## Problem

Rails converts its configured Ruby version into versioned Debian packages such
as `ruby3.4` and `ruby3.4-dev`. Debian 13 does not provide those package names,
so provisioning a Rails site configured for Ruby 3.4 fails before Puma is
installed. The Rails build container has the same package-name assumption and
cannot guarantee the configured host and build Ruby versions match.

## Definitions

**Ruby toolchain:** A pinned Ruby interpreter, its standard Bundler command,
and the native libraries required to compile it. It is installed independently
of the Linux distribution's Ruby packages.

**Exact Ruby version:** A stable `X.Y.Z` Ruby release selected by a Rails
project. It is distinct from an `X.Y` release series and determines the source
archive and executable installation path.

## Desired outcome

Rails setup and deployment work on supported Debian and Ubuntu hosts for the
selected exact Ruby version. Runtime provisioning and release builds use the
same verified Ruby source release, while multiple sites can use different Ruby
versions without changing a global Ruby executable.

## Scope

This change includes exact Rails Ruby version selection, verified source-based
installation on the host and in the build cache, versioned Ruby execution for
Puma and Rails build scripts, regression tests, and related documentation.
Existing Rails projects using the former `X.Y` values continue to resolve to
their supported exact release.

## Constraints

Ruby archives must be downloaded only over HTTPS and verified against committed
SHA-256 checksums. Host provisioning must use the existing BonesInfra runtime;
release builds must keep using the private build cache. Full E2E tests must not
be run during this work.

## Exclusions

This change does not add arbitrary Ruby-version installation, Ruby version
managers, support for non-Debian/Ubuntu hosts, or changes to Rails application
source code.
