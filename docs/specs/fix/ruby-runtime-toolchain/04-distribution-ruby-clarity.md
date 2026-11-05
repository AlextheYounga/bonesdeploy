# Distribution Ruby Clarification

## Trigger

Compiling Ruby during Rails site setup exhausted the resources of a 512 MiB
DigitalOcean server. The user directed production setup to install the Debian or
Ubuntu distribution Ruby through APT and requested that the README make the
resulting behavior explicit.

## Decision

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

## Supersedes

This supersedes the decision to compile and install the selected exact Ruby
release on production hosts and to ship the local build's `vendor/bundle` as the
production bundle. It does not supersede the exact Ruby toolchain used for local
asset compilation.

## Required Authoritative Updates

`01-idea.md`, `02-plan.md`, and `03-tasks.md` are updated to describe the
distribution Ruby runtime, target bundle installation, current execution state,
and required validation.
