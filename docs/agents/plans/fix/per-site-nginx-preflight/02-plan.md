# Plan

## Current behavior

`commands/deploy/lifecycle.rs` calls `preflight::validate_ready` after a
release is sealed and before `lifecycle::activate::run` repoints `current`.
`release/lifecycle/preflight.rs::run_nginx_test` currently executes bare
`nginx -t`, which selects the host's default nginx configuration. BonesInfra
starts each site's nginx service with `nginx -c /srv/conf/<site>/nginx.conf`.

## Intended behavior

The existing pre-cut-over gate will validate the site nginx configuration used
by the service. A non-zero nginx result will abort before activation through
the existing staged-release cleanup path.

## Approach

Derive the site config path from `DEFAULT_CONF_ROOT_PARENT`, the registered
site identifier, and `NGINX_CONF`. Pass the site identifier to the nginx
preflight function and execute `nginx -t -c <derived path>`. Include that path
in successful and failed diagnostics. Keep the existing web-root check and
preflight failure propagation unchanged.

## Responsibilities and boundaries

`release/lifecycle/preflight.rs` owns pre-cut-over readiness validation and
will own site nginx command construction. `commands/deploy/lifecycle.rs` will
provide the acquired mutation's registered site identifier. `CONTEXT.md` will
describe the deployment guarantee.

## Affected areas

- `crates/bonesremote/src/release/lifecycle/preflight.rs`
- `crates/bonesremote/src/commands/deploy/lifecycle.rs`
- `CONTEXT.md`
- This plan record

## Decisions

Use `nginx -t -c` rather than bare `nginx -t` because the former is exactly the
configuration passed to the per-site systemd service. Derive the path from
shared constants rather than duplicating `/srv/conf` or `nginx.conf`.

## Risks

An incorrect path would reject every deployment before activation. A unit test
must assert the derived per-site config path, and focused crate tests must
prove the preflight module still propagates nginx failures.

## Validation

Run the bonesremote test suite, `cargo fmt`, `cargo clippy`, and `shfmt -w .`.
Review the final diff to confirm no activation occurs before the per-site nginx
test.
