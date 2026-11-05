# SSL Module Layout Clarification

## Trigger

Review of the SSL command implementation found that
`bonesinfra.cli.commands.site.ssl` remained a package only because an earlier
implementation used `ssl/plan.py`. The current package contains no submodules
and is inconsistent with the neighboring single-file site commands.

## Decision

Move the SSL command implementation from `site/ssl/__init__.py` to
`site/ssl.py`. Preserve the public import path
`bonesinfra.cli.commands.site.ssl`, so callers and test patch paths do not
change.

## Supersedes

This adds a small command-layout cleanup to the affected area without changing
the approved router-activation behavior or any public interface.

## Required Authoritative Updates

- `01-idea.md`: include normalization of the touched SSL command module in
  scope.
- `02-plan.md`: identify `site/ssl.py` as the SSL command boundary and record
  the package-to-module move.
- `03-tasks.md`: track the move and its validation.
