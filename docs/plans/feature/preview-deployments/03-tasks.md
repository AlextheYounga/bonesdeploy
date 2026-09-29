# Tasks

## Implementation

- [ ] Add canonical `Preview` configuration with validated base domain and
  14-day retention default plus 1-through-365 validation; update local `.env`, remote deployment transport,
  BonesInfra provisioning transport, and mirrored Rust/Python path definitions.
- [ ] Add validated preview identifier, alias, hostname, and revision-request
  boundaries so only resolved commit prefixes reach paths, service names, or
  Nginx templates; aliases cannot occupy the 12-to-40-character hexadecimal
  commit-ID namespace; and prefix collisions lengthen without changing existing
  preview URLs.
- [ ] Add the local `preview` command group with setup, deploy, list, status,
  logs, remove, and prune argument contracts and thin dispatch handlers.
- [ ] Extend the SSH adapter with a PTY-backed interactive command for manual
  Certbot while preserving fixed-stdin streaming for typed preview deploy
  requests.
- [ ] Implement `preview setup` input collection, public/untrusted-code warning,
  production/cross-project Nginx-name collision validation, wildcard
  A/AAAA/CNAME guidance, random hostname resolution, privileged exact-SAN
  certificate inspection, 30-day renewal decision, and safe obsolete-TXT
  guidance that preserves unrelated active challenge values.
- [ ] Add the interactive Certbot DNS-01 invocation for the exact wildcard SAN
  and deterministic project lineage, ensuring ACME tokens remain transient and
  a failed/interrupted challenge performs no Nginx mutation.
- [ ] Add a privileged stage-only route reconciliation used by setup to
  regenerate the include from `SiteState` without reload, writing an empty
  projection only on first setup and preserving ready/publishing routes on
  rerun.
- [ ] Add BonesInfra preview provisioning for the stable wildcard TLS gateway,
  route-include parent, preview directories, trusted native systemd/AppArmor
  templates, and root-owned preview runtime descriptor without replacing
  state-derived route content.
- [ ] Make preview provisioning transactional and convergent: snapshot every
  preview-owned file/link, stage and atomically install candidates, validate,
  reload and verify Nginx, restore and validate/reload the old set on failure,
  and retain named recovery artifacts with critical instructions if restoration
  fails, without editing production routes.
- [ ] Extend the BonesInfra manifest and each supported native framework's
  runtime contract with release-scoped preview app/Nginx commands, sockets,
  mutable path declarations, and AppArmor inputs; reject Docker previews.
- [ ] Persist preview configuration only after successful remote provisioning,
  synchronize the remote descriptor, and implement rerun reconciliation for a
  matching orphaned certificate/gateway, local write failure, descriptor-sync
  failure, and local/remote domain disagreement.
- [ ] Rename existing Quick Tunnel status fields, types, labels, and tests so
  `preview` exclusively means commit-specific deployment and Quick Tunnel output
  remains explicitly named.
- [ ] Upgrade `SiteState` to schema version 2 with default-empty preview records,
  preserving active, staged, decommissioning, and tombstone fields; reject
  future/malformed versions through existing fail-closed recovery and add
  phase/process, alias, timestamp, resource, error, and `SiteMutation` behavior
  without another state file or lock.
- [ ] Refactor release preparation into one production/preview-capable sequence
  for stage, source export, build, promotion, prepare, and sealing while leaving
  production activation, rollback, and maintenance semantics unchanged.
- [ ] Add explicit release environment wiring: production continues using
  production shared paths, while previews atomically receive an empty or
  supplied environment and only preview-local mutable paths from the trusted
  runtime descriptor.
- [ ] Run preview prepare scripts in a constrained transient systemd unit with
  preview `PROJECT_ROOT`, clean environment, exact AppArmor policy,
  production/other-preview paths inaccessible, no service-control path, fixed
  resource limits, and a release-specific preview log root; retain the current
  production prepare runner.
- [ ] Add the exact sudoers allowlist entry and typed stdin request for
  `bonesremote preview deploy --site <site>` so routine preview deployment uses
  the existing deploy identity without optional privileged command arguments.
- [ ] Implement the remote preview deploy coordinator: resolve the commit,
  choose/reuse its identifier, persist every phase, run shared preparation,
  create the preview release link and trusted instance files, start and verify
  the preview target/socket, and preserve failed/interrupted diagnostics.
- [ ] Implement the state-derived route reconciler with fixed exact-host server
  blocks, wildcard unknown-host denial, atomic route replacement, `nginx -t`,
  reload verification, previous-file restoration, and crash recovery from
  `Publishing`/`Removing` records.
- [ ] Publish commit routes only after the local origin is ready and implement
  alias movement with one authoritative state snapshot and one Nginx reload;
  make publication, existing-preview alias moves, remove, and prune converge
  from every crash point without rebuilding or extending expiration.
- [ ] Protect every release referenced by a preview record from normal
  production release pruning.
- [ ] Implement machine-readable remote list and status reports with revision,
  release, phase, URLs, aliases, timestamps, expiration, DNS, wildcard
  certificate/SAN/expiry, route agreement, units, socket, and interrupted or
  cleanup diagnostics; render those reports in the local CLI.
- [ ] Implement preview log streaming from a release-specific preview log root,
  followed by only that preview's systemd journals, and remove logs with the
  record/resources.
- [ ] Implement idempotent remove: persist `Removing`, withdraw routes and
  aliases, stop the preview target, remove only validated preview instance
  resources and unreferenced release data, then remove state or retain
  `CleanupPending` on failure.
- [ ] Implement prune using persisted UTC expiration, removing expired ready or
  failed previews, retaining non-expired failed diagnostics, and retrying every
  `CleanupPending` record without a retry limit, timer, or production access;
  retention changes affect only future records.
- [ ] Update command, architecture, security, configuration, and operations
  documentation to distinguish preview deployments from Quick Tunnels, explain
  manual wildcard DNS/TLS renewal, state the public/trusted-code policy, and
  document native-only and command-driven retention limits.

## Validation

- [ ] Add and run core integration tests for preview config serialization,
  defaults, domains, aliases, hostnames, identifiers, collision extension,
  canonical paths, and malicious/traversal input rejection.
- [ ] Add and run local CLI integration tests for every preview command, setup
  confirmation and DNS guidance, deploy JSON/stdin transport, environment-file
  behavior, output rendering, and explicit Quick Tunnel terminology.
- [ ] Add and run SSH adapter tests for PTY command construction and terminal
  forwarding without persisting or logging ACME tokens and preview secrets.
- [ ] Add and run state tests for version-1 compatibility, version-2 preservation
  of decommission/tombstone fields, future-version rejection, atomic preview
  transitions, unique alias movement, interruption, malformed-state failure,
  and recovery markers.
- [ ] Add and run shared lifecycle regression tests proving preview and
  production use one preparation path, preview prepare/runtime policy cannot
  read production current/shared/config/socket paths, preview publication never
  moves `current`, and production deploy/rollback/cancellation remains unchanged.
- [ ] Add and run release-pruning tests proving every retained preview release
  is protected and removal of one preview cannot delete production or another
  preview's release.
- [ ] Add and run preview runtime tests for exact release paths, unit names,
  AppArmor plus inaccessible-path confinement, fixed cgroup/file-descriptor
  limits, environment isolation, unique Unix sockets, readiness checks, and
  idempotent allowlisted cleanup.
- [ ] Add and run route tests for commit and alias host blocks, default denial,
  alias/commit collision rejection, exact/wildcard precedence over existing
  regex servers, one-reload alias replacement, convergence at every
  publish/move/remove crash boundary, validation-before-reload, recovery after
  validation/reload/restore failure, and unchanged production configs.
- [ ] Add and run BonesInfra pytest coverage for convergent setup, certificate
  reuse/renewal decisions, stable gateway ownership, native framework runtime
  descriptors, systemd/AppArmor templates, manifests, and Docker rejection.
- [ ] Add and run list/status/log/remove/prune tests covering healthy, failed,
  interrupted, expired, cleanup-pending, DNS-mismatched, route-drifted,
  certificate-expiring, inactive-unit, and missing-socket observations.
- [ ] Manually validate on a disposable server the wildcard DNS and TXT
  instructions, interrupted/rerun setup, two concurrent previews beside
  production, immutable commit URLs, alias movement, logs, remove, prune, manual
  renewal warning, and production availability through forced Nginx failures.
- [ ] Run `ruff check .`, `ruff format .`, and `uv run pytest` from
  `crates/bonesinfra/python`; run affected Rust tests, `cargo fmt`, `cargo
  clippy`, and `shfmt -w .`; do not run the end-to-end suite.

## Completion

- [ ] Review the final diff for one shared release preparation pipeline, one
  authoritative `SiteState`, one existing site lock, canonical path ownership,
  and no production activation or shared-secret coupling.
- [ ] Confirm no DNS-provider abstraction, provider credential, named
  Cloudflare Tunnel, ngrok integration, automatic certificate renewal, webhook,
  hosted control plane, database clone, access-policy framework, or Docker
  preview was introduced.
- [ ] Confirm setup/removal/pruning failure paths preserve production routing
  and leave enough state and logs for a repeat command to converge safely.
- [ ] Confirm documentation and status output state that URLs are public,
  revisions must be trusted, certificate renewal and pruning are manual, and
  Quick Tunnels remain a separate current-site ingress feature.
- [ ] Record validation results, material approved-plan deviations, and any
  deliberately unfinished manual checks in the completion notes.

## Completion notes

Planning complete. Implementation is blocked on human review and approval of
`01-idea.md`, `02-plan.md`, and this task list.
