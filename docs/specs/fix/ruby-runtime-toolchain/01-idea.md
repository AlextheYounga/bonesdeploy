# Idea

## Request

Install the production Rails Ruby runtime from the supported Debian or Ubuntu
distribution instead of compiling Ruby during site setup. Document this runtime
and deployment model clearly in the README.

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

Rails site setup installs Ruby and Bundler through APT without compiling Ruby.
Each deployment creates its target bundle on the production host before running
migrations or activating the release, so native gems match the host's Ruby ABI.
The README makes the host-controlled Ruby version, deployment-time RubyGems
access, and application compatibility requirement explicit.

## Scope

This change includes distribution Ruby installation on production hosts,
distribution executable paths for Puma and Bundler, removal of locally compiled
gems from release artifacts, target bundle installation before migrations,
regression tests, and related documentation. The existing exact Ruby selection
continues to control the local asset build only.

## Constraints

Production Ruby packages must come from the host's configured APT repositories.
Target bundle installation must run as the site's runtime user while the staged
release is writable, and deployment must stop before activation when Bundler or
the application's Ruby requirements fail. Full E2E tests must not be run during
this work.

## Exclusions

This change does not add a third-party Ruby repository, arbitrary production
Ruby-version installation, Ruby version managers, support for non-Debian/Ubuntu
hosts, vendored Ruby gems, or changes to Rails application source code.
