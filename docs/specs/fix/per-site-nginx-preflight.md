# Per Site Nginx Preflight

<!-- Merged from fix/per-site-nginx-preflight -->

## Idea

### Request

Validate `nginx -t` against the site's nginx configuration before deployment
activation, so a deployment cannot activate a release when nginx will fail.

### Problem

The pre-cut-over nginx gate currently invokes `nginx -t` without `-c`. That
tests nginx's default configuration rather than the per-site configuration
started by `<site>-nginx.service`. A broken include in `/srv/conf/<site>/nginx.conf`
can therefore pass the gate and fail only after the release becomes current.

### Definitions

**Site nginx configuration:** The nginx configuration at
`/srv/conf/<site>/nginx.conf` used by the site's `<site>-nginx.service`. It is
separate from nginx's global default configuration.

**Pre-cut-over gate:** A deployment check performed while the existing release
continues to serve and before the `current` symlink is changed.

### Desired outcome

Before activation, a deployment runs `nginx -t -c /srv/conf/<site>/nginx.conf`.
If that command fails, deployment exits with the nginx diagnostic, removes the
staged release, and leaves `current` unchanged.

### Scope

The deployment preflight command, its regression coverage, and deployment
documentation.

### Constraints

Use the existing Phase A pre-cut-over lifecycle and the centralized path
constants. Do not run end-to-end tests or commit changes.

### Exclusions

This change does not alter nginx service sandboxing, file ownership, AppArmor,
or post-activation service verification.


## Plan

### Current behavior

`commands/deploy/lifecycle.rs` calls `preflight::validate_ready` after a
release is sealed and before `lifecycle::activate::run` repoints `current`.
`release/lifecycle/preflight.rs::run_nginx_test` currently executes bare
`nginx -t`, which selects the host's default nginx configuration. BonesInfra
starts each site's nginx service with `nginx -c /srv/conf/<site>/nginx.conf`.

### Intended behavior

The existing pre-cut-over gate will validate the site nginx configuration used
by the service. A non-zero nginx result will abort before activation through
the existing staged-release cleanup path.

### Approach

Derive the site config path from `DEFAULT_CONF_ROOT_PARENT`, the registered
site identifier, and `NGINX_CONF`. Pass the site identifier to the nginx
preflight function and execute `nginx -t -c <derived path>`. Include that path
in successful and failed diagnostics. Keep the existing web-root check and
preflight failure propagation unchanged.

### Responsibilities and boundaries

`release/lifecycle/preflight.rs` owns pre-cut-over readiness validation and
will own site nginx command construction. `commands/deploy/lifecycle.rs` will
provide the acquired mutation's registered site identifier. `CONTEXT.md` will
describe the deployment guarantee.

### Affected areas

- `crates/bonesremote/src/release/lifecycle/preflight.rs`
- `crates/bonesremote/src/commands/deploy/lifecycle.rs`
- `CONTEXT.md`
- This plan record

### Decisions

Use `nginx -t -c` rather than bare `nginx -t` because the former is exactly the
configuration passed to the per-site systemd service. Derive the path from
shared constants rather than duplicating `/srv/conf` or `nginx.conf`.

### Risks

An incorrect path would reject every deployment before activation. A unit test
must assert the derived per-site config path, and focused crate tests must
prove the preflight module still propagates nginx failures.

### Validation

Run the bonesremote test suite, `cargo fmt`, `cargo clippy`, and `shfmt -w .`.
Review the final diff to confirm no activation occurs before the per-site nginx
test.


## Tasks

### Implementation

- [x] Change the pre-cut-over nginx command to test the derived site nginx configuration.
- [x] Pass the registered site identifier from deployment lifecycle to the preflight command.
- [x] Add regression coverage for the derived per-site nginx configuration path.
- [x] Document the per-site nginx pre-cut-over gate.

### Validation

- [x] Run the bonesremote test suite.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`.

### Completion

- [x] Review the final diff and record validation results.

### Completion notes

Implemented the explicit per-site nginx preflight with the existing Phase A
gate. `cargo test -p bonesremote`, `cargo fmt`, `cargo clippy`, `shfmt -w .`,
and `git diff --check` passed. End-to-end tests were not run.
