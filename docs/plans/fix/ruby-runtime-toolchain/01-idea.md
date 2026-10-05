# Idea

## Request

This completed distribution-Ruby change is superseded by
`feature/mise-runtimes`: production Rails now uses the exact configured managed
Ruby runtime. Its historical goal was to avoid production source compilation.

## Problem

Rails site setup currently compiles a selected exact Ruby release from source.
That build can exhaust the memory of a 512 MiB production server and make the
server unresponsive. Shipping the application bundle produced by the local
Debian build container is not a safe substitute because supported production
hosts can provide a different Ruby ABI through their distribution packages.

## Definitions

**Distribution Ruby:** The default `ruby`, `ruby-dev`, and `ruby-bundler`
packages supplied by the production host's configured Debian or Ubuntu APT
repositories.

**Target bundle:** The production gems installed into `vendor/bundle` inside a
staged release by the production host's Ruby and Bundler before that release is
activated.

## Desired outcome

Rails local builds and production use the same exact configured Ruby installed
precompiled-only by pinned mise. The artifact packages `vendor/bundle` and the
lockfile-selected Bundler; prepare validates runtime identity and runs migrations
without downloading or compiling gems.

## Scope

This historical record remains for its completed source-build removal. The
current Rails runtime contract is owned by `feature/mise-runtimes`.

## Constraints

Production source-build fallback remains prohibited. The current exact runtime,
packaged Bundler, and E2E requirements are defined by `feature/mise-runtimes`.

## Exclusions

This change does not add a third-party Ruby repository, arbitrary production
Ruby-version installation, Ruby version managers, support for non-Debian/Ubuntu
hosts, vendored Ruby gems, or changes to Rails application source code.
