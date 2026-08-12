# Idea

## Request

Validate `nginx -t` against the site's nginx configuration before deployment
activation, so a deployment cannot activate a release when nginx will fail.

## Problem

The pre-cut-over nginx gate currently invokes `nginx -t` without `-c`. That
tests nginx's default configuration rather than the per-site configuration
started by `<site>-nginx.service`. A broken include in `/srv/conf/<site>/nginx.conf`
can therefore pass the gate and fail only after the release becomes current.

## Definitions

**Site nginx configuration:** The nginx configuration at
`/srv/conf/<site>/nginx.conf` used by the site's `<site>-nginx.service`. It is
separate from nginx's global default configuration.

**Pre-cut-over gate:** A deployment check performed while the existing release
continues to serve and before the `current` symlink is changed.

## Desired outcome

Before activation, a deployment runs `nginx -t -c /srv/conf/<site>/nginx.conf`.
If that command fails, deployment exits with the nginx diagnostic, removes the
staged release, and leaves `current` unchanged.

## Scope

The deployment preflight command, its regression coverage, and deployment
documentation.

## Constraints

Use the existing Phase A pre-cut-over lifecycle and the centralized path
constants. Do not run end-to-end tests or commit changes.

## Exclusions

This change does not alter nginx service sandboxing, file ownership, AppArmor,
or post-activation service verification.
