# Plan

## Current Behavior

`services/linux/runtime.py` installs Cloudflared for every domainless managed
site, renders and registers `<site>-cloudflared.service` with `<site>.target`,
and starts it during normal runtime startup. Site setup invokes that runtime
flow. SSL removes the tunnel after activating a real-domain route, manifests
expect the service for domainless sites, and release restart contains a special
case to exclude the target-registered tunnel.

The service uses `--url unix:<socket>`. Cloudflared normalizes this to an HTTP
origin whose hostname is `unix`, producing DNS failures and 502 responses.
Cloudflare's Quick Tunnel documentation supports an HTTP localhost origin, and
the installed Cloudflared cannot combine the accountless `--url` dispatcher
with `--unix-socket`.

## Intended Behavior

Normal runtime orchestration manages AppArmor, per-site nginx, real-domain
routing, framework services, and the site target only. It neither installs nor
starts Cloudflared. Domainless setup points users to the optional tunnel command
instead of claiming a tunnel already exists.

`bonesdeploy site tunnel start [--yes]` invokes a dedicated BonesInfra tunnel
provisioning command. It installs Cloudflared, renders a root-owned nginx route
bound to a deterministic project-specific loopback port, proxies that route to
the per-site nginx Unix socket, validates and reloads nginx, renders a hardened
Cloudflared unit using an HTTP loopback URL, enables the unit independently of
the site target, and starts it. The command reports the current URL or explains
that startup is still in progress.

`bonesdeploy site tunnel stop [--yes]` stops and disables the unit, removes the
unit and loopback route, reloads systemd, validates and reloads nginx, and keeps
the host-level package/repository installed. `site tunnel status` reuses the
structured remote site status and clearly reports active URL, active-but-
starting, or stopped state. The general `site status` continues to display an
active optional tunnel.

## Approach

Remove Cloudflared from `services/linux/runtime.py`, SSL provisioning, required
manifest services, and release-target restart filtering. Keep domainless public
router cleanup in runtime because public routing remains domain-owned.

Extend Python deployment paths for the Quick Tunnel nginx route. Derive a stable
unprivileged loopback port from the validated project name with a deterministic
CRC32 mapping. Both templates receive the same computed value from the
Cloudflared service module. The nginx route uses `listen 127.0.0.1:<port>
default_server` and proxies to the per-site Unix socket. A collision or an
unavailable port causes nginx validation/reload to fail rather than silently
routing to another site.

Change the Cloudflared unit to use
`--url http://127.0.0.1:<port>`, depend on both system nginx and per-site nginx,
retain restart-on-failure and sandboxing, and install under
`multi-user.target`. Do not add it to `<site>.target`; explicit startup enables
its independent boot lifecycle, and explicit stop removes it.

Add a private `bonesinfra tunnel start|stop` command group behind the public
Rust site command. Reject tunnel startup for Compose sites without managed nginx
ingress. End each mutating tunnel operation with etckeeper commit, matching
other provisioning flows.

## Responsibilities And Boundaries

- `bonesdeploy::commands::site::tunnel` owns public prompts, invocation, and
  human-readable status.
- `bonesinfra.cli.app` owns private command parsing and typed request loading.
- `bonesinfra.services.linux.cloudflared` owns package installation, port
  derivation, loopback route, service lifecycle, and cleanup.
- Root nginx owns the loopback HTTP-to-Unix proxy; per-site nginx remains the
  application-facing origin.
- `bonesremote::commands::status` remains the read-only observer for service
  state and journal URL discovery.

## Affected Areas

- BonesDeploy site CLI arguments, dispatch, command module, and CLI tests.
- BonesInfra CLI, Cloudflared service, nginx/systemd templates, paths, runtime,
  SSL, manifest, and focused Python tests.
- BonesRemote release service restart behavior and tests.
- Existing Cloudflare tunnel plan/tasks, README, CONTEXT files, and architecture
  command-flow documentation where behavior is described.

## Decisions

- The feature is an explicit `site tunnel` lifecycle, not inferred from an
  empty domain.
- A root-nginx loopback route adapts supported HTTP to the existing Unix socket,
  avoiding direct application exposure and edits to every framework template.
- A deterministic port avoids new persisted allocation state. Port collisions
  fail closed during nginx validation and are an accepted operational limit.
- Explicit start enables reboot startup independently; normal site target
  startup never owns or starts the tunnel.
- Stop removes project configuration but does not uninstall the shared package.
- Status observes journald rather than persisting an ephemeral URL.

## Risks

- A deterministic port can collide with another site or local service; nginx
  validation/reload must expose the conflict without replacing working config.
- The URL may not be present immediately after systemd reports the service
  active; status must represent the starting state.
- Cloudflare package/network failures can prevent startup without affecting the
  normal site runtime.
- Quick Tunnels have no SLA, cap concurrent requests, and do not support SSE.

## Validation

- Python tests prove normal runtime setup/orchestration never call Cloudflared.
- Cloudflared tests prove explicit start installs, renders a loopback-only HTTP
  origin, avoids site-target registration, starts/enables the unit, and cleanup
  removes both unit and route.
- CLI tests prove `site tunnel start|stop|status` are discoverable and parsed.
- Manifest and SSL tests prove neither flow automatically owns/removes a tunnel.
- Existing URL extraction/status tests prove active and starting output.
- Run focused Python and Rust tests, `ruff check .`, `ruff format .`, the Python
  suite, `cargo clippy`, `cargo fmt`, and `shfmt -w .`; do not run E2E tests.
- Review the final diff and verify `tests/cleancode/Cargo.toml` remains untouched.
