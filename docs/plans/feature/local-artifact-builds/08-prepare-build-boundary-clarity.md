# Prepare Build Boundary Clarification

## Trigger

Distribution-runtime changes made Rails and Django install application
dependencies during remote prepare. A Rails deployment compiled native gems on
a small production server and exhausted its resources. This also contradicted
the established complete-artifact contract, which defines dependency
installation and native-extension compilation as local build work.

## Decision

Rails and Django local builds produce complete runnable artifacts. Rails retains
its production `vendor/bundle`; Django retains its production dependency tree
and release-local launchers. Their prepare scripts may validate that required
artifact output exists, but may not invoke Bundler or pip, create a virtual
environment, download application dependencies, or compile native extensions.

Rails and Django prepare continue to own database migrations and other
production-state operations such as Django deployment checks and static-file
collection. Missing or incompatible packaged dependencies fail prepare before
activation without attempting a production build.

## Supersedes

This supersedes the later Ruby and Python distribution-runtime decisions that
moved application dependency installation into remote prepare. It restores and
does not otherwise change the complete-artifact and prepare definitions in
`01-idea.md` and `02-plan.md`.

## Required Authoritative Updates

`01-idea.md` and `02-plan.md` already state the required build and prepare
boundary. `03-tasks.md` records the regression correction and its validation.
