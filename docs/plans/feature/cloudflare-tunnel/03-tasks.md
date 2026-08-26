# Tasks

## Implementation

- [x] Remove live `preview_domain` semantics from `bonesdeploy-core`: delete
  nip.io derivation and the BonesInfra input key, remove the field from the
  runtime `App` model and serializer, and retain an input-only legacy field so
  existing `bones.toml` files containing `preview_domain` still deserialize.
- [x] Remove `preview_domain` from the default `bones.toml` scaffold,
  BonesDeploy-to-BonesInfra runtime data, BonesInfra `DnsConfig`, and template
  context so newly written configuration and provisioning data no longer carry
  a temporary hostname.
- [x] Refactor the Nginx runtime reconciliation so the per-site Nginx service
  and Unix socket are provisioned without requiring a public hostname, while
  the root-managed project router is rendered/enabled only for a configured
  real domain.
- [x] Add no-domain Nginx cleanup that removes both the project router's enabled
  link and available config from prior nip.io runs, validates the remaining
  Nginx configuration, and reloads system Nginx safely.
- [x] Add the BonesInfra `cloudflared` service boundary that idempotently
  configures Cloudflare's Debian/Ubuntu package source, installs `cloudflared`,
  renders `<project>-cloudflared.service`, and registers it with the existing
  site systemd target.
- [x] Add the Quick Tunnel systemd template so it runs as the site runtime user,
  depends on/orders after `<project>-nginx.service`, restarts on failure, and
  proxies `cloudflared tunnel --no-autoupdate --url
  unix:<runtime_nginx_socket>` directly to the per-site Nginx socket.
- [x] Update runtime reconciliation so an empty real domain creates/starts the
  project Quick Tunnel and removes the project public router, while a configured
  real domain removes/stops/unregisters the Quick Tunnel and uses the public
  Nginx router.
- [x] Update the SSL deployment sequence so certificate issuance and the final
  HTTPS Nginx activation complete before the project Quick Tunnel is
  stopped/unregistered/removed.
- [x] Update the BonesInfra manifest so the Quick Tunnel service is runtime
  state expected only for a no-domain site and SSL artifacts are derived only
  from a configured real domain.
- [x] Extend `bonesremote status` with structured temporary-preview state and a
  focused parser that reports the newest current-boot
  `https://*.trycloudflare.com` URL only while the project cloudflared service
  is active.
- [x] Extend `bonesdeploy status` to render temporary-preview state and expose
  its remote-status fetch path for reuse by setup.
- [x] Update `bonesdeploy setup` so no-domain setup surfaces the current preview
  URL and does not recommend `bonesdeploy remote ssl`; preserve existing first
  push, real-domain SSL, and deploy guidance.
- [x] Update `README.md`, `CONTEXT.md`, and
  `crates/bonesinfra/python/CONTEXT.md` to describe Quick Tunnel temporary
  ingress, URL instability across restarts, Cloudflare's preview-only limits,
  and the unchanged real-domain Nginx/Certbot path.

## Validation

- [x] Add and run Rust configuration tests proving legacy `preview_domain`
  input remains readable, nip.io is no longer derived, and serialized config
  omits the legacy key.
- [x] Add and run BonesInfra context tests proving temporary hostnames are no
  longer configuration/template data.
- [x] Add and run cloudflared provisioning tests covering package setup,
  project-scoped unit rendering, runtime user, Unix-socket origin, Nginx
  ordering/dependency, site-target registration, and real-domain removal.
- [x] Add and run Nginx reconciliation tests proving no-domain sites retain
  per-site Nginx while removing public project routing and real-domain sites
  retain the public router.
- [ ] Add and run SSL-flow tests proving the Quick Tunnel is removed only after
  successful permanent HTTPS activation.
- [x] Add and run manifest tests for conditional Quick Tunnel service ownership
  and real-domain-only SSL artifacts.
- [ ] Add and run `bonesremote` status tests for active/inactive tunnel state,
  newest-URL selection, irrelevant journal lines, and changed URLs after a
  simulated service restart.
- [ ] Add and run local status/setup tests proving the current preview URL is
  displayed and no-domain setup does not route the user toward real-domain SSL.
- [ ] Manually smoke-test a no-domain site through its per-site Nginx Unix
  socket and verify the generated `trycloudflare.com` HTTPS URL serves the
  expected application.
- [ ] Manually restart `<project>-cloudflared.service` and verify
  `bonesdeploy status` reports the replacement URL rather than the previous
  hostname.
- [ ] Manually complete a real-domain SSL handoff and verify the real HTTPS
  domain works before the project Quick Tunnel disappears.
- [x] Run `ruff check .`, `ruff format .`, and `uv run pytest` from
  `crates/bonesinfra/python`.
- [x] Run affected Rust tests, `cargo fmt`, `cargo clippy`, and `shfmt -w .`
  without running the end-to-end test suite.

## Completion

- [x] Review the final diff and confirm there is one temporary-ingress model:
  no-domain sites use a project Quick Tunnel, real-domain sites use public
  Nginx/Certbot, and no live nip.io or `preview_domain` behavior remains.
- [x] Confirm no generated Cloudflare hostname is persisted as authoritative
  configuration and no provider/plugin abstraction, account credential flow,
  or parallel release-preview system was introduced.
- [x] Confirm documentation states the Quick Tunnel limitations and that the
  feature is temporary preview infrastructure rather than production ingress.

## Completion notes

Implementation uses the repository's current flat `.env` contract rather than
the older `bones.toml` terminology in this record. Legacy `PREVIEW_DOMAIN` is
discarded during loading and omitted during saving.

After the develop-side config-centralization refactor landed on this branch,
the integration kept that design and re-applied the tunnel semantics on top:

- The typed request transports (`ProvisioningRequest`/`SiteFields`) and the
  Python site parser no longer carry `preview_domain`; the field was removed
  from `transport.rs`, `request.py`, and test fixtures. The old key remains a
  recognized-but-discarded managed input in the Rust parser so existing files
  keep loading.
- `secrets init` composes the first encrypted environment through
  `environment::prepare()` with the two-argument framework examples.
- `bonesdeploy setup` delegates to `server::setup` + `site::setup`; the
  no-domain preview guidance moved into `site::setup::print_next_step`, which
  reports the current Quick Tunnel URL instead of suggesting SSL for
  domain-less sites.
- Nginx router reconciliation stays tunnel-based: public routing only for a
  real domain, plus `remove_project_router` cleanup with tunnel setup.

`cargo fmt`, `cargo clippy`, `shfmt -w .`, focused Rust tests, `ruff format .`,
`ruff check .`, and the full BonesInfra Python test suite pass. The manual
Quick Tunnel smoke test, restart observation, and real-domain handoff remain
for a server with Cloudflare network access. Focused Rust status/setup and
BonesInfra SSL-handoff tests remain unchecked. The E2E harness still routes
through nip.io hosts and needs a follow-up decision for no-domain sites.
