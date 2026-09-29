# Plan

## Current behavior

### Commands and transport

`crates/bonesdeploy/src/cli/{args,dispatch}.rs` define thin local commands.
Handlers invoke BonesInfra or use `infra/ssh.rs` to execute machine-oriented
BonesRemote commands. Production deploy connects as `git`, synchronizes
`RemoteDeploymentConfig`, and invokes the exact sudo-allowlisted
`bonesremote deploy --site <site>` form. Administrative commands use the
configured privileged SSH identity.

There is no preview command group and no interactive SSH helper. Existing SSH
helpers stream output or send fixed stdin, but cannot request a PTY and bridge
the user's terminal for Certbot's manual DNS prompt.

### Lifecycle and state

`bonesremote::commands::deploy` resolves a commit in the site's bare repository.
Its coordinator stages a release, exports with `git archive`, builds through the
rootless build user, promotes artifacts, links production `shared/.env`, runs
prepare scripts as the runtime user, seals, validates, moves `current`, restarts
the site target, and prunes.

`SiteMutation` combines validated configuration with the per-site
`DeploymentLock`. `SiteState` is the sole runtime-mutated state document, written
by temp-file, fsync, rename, and directory fsync. It stores the production
deployment, staged release, decommissioning, and tombstone state. Normal pruning
protects only `current`.

Preparation is reusable, but the coordinator assumes every success becomes
`current`. `wire_shared` always links production secrets, prepare exposes the
production project root, preflight checks production Nginx, service restart
targets production, and pruning can delete any non-current release. These
assumptions prevent concurrent releases.

### Runtime and ingress

BonesInfra provisions one native production service set per site. App and
per-site Nginx units use `current`, production `runtime.env`, and stable sockets
under `/run/<site>`. Framework runtime provisioning owns systemd, Nginx, and
AppArmor templates. Units use the site UID/GID, root-owned definitions,
AppArmor, `ProtectSystem=strict`, `NoNewPrivileges`, private devices/tmp,
restricted namespaces/address families, and empty capabilities.

Compose is production-scoped through `current`, production `shared/.env`, the
stable `bonesdeploy-<site>` project, and persistent volumes. It cannot safely
host a v1 preview through a path substitution.

BonesInfra's `services/linux/nginx/router.py` owns system Nginx files,
`nginx -t`, and reload. Certbot currently supports one non-interactive HTTP-01
certificate, not wildcard manual DNS-01. The Quick Tunnel separately proxies the
current site's Nginx socket; `bonesremote status` currently misnames that state
`preview`, which would conflate two different features.

## Intended behavior

### Setup and convergence

`bonesdeploy preview setup` executes this order:

1. Load configuration and collect `--domain`, `--email`, and
   `--retention-days`; retention defaults to 14 and accepts 1 through 365.
2. Validate a lowercase DNS base name, rejecting wildcard/URL syntax, a change
   while previews exist, equality with production, and a production hostname one
   label below the preview base. Inspect managed files and `nginx -T` before
   certificate issuance; an exact or wildcard name owned by another site fails
   closed. Generated exact route names and the longest wildcard gateway take
   precedence over regex names under Nginx selection rules; focused config tests
   prove that existing regex servers cannot capture preview hosts.
3. Explain that URLs are public, credentials must be preview-safe, and same-site
   previews are not an untrusted-code sandbox. Require confirmation unless
   `--yes` is supplied.
4. Print `*.preview.example.com A <server IPv4>`, plus AAAA guidance for public
   IPv6 or CNAME guidance for a stable server name. Resolve a random label and
   require it to reach the configured server before remote mutation.
5. Through privileged SSH inspect
   `/etc/letsencrypt/live/bonesdeploy-<site>-preview/{fullchain,privkey}.pem`.
   Require readable regular-file targets, only the exact wildcard SAN, and more
   than 30 days remaining to reuse it.
6. Otherwise bridge a privileged PTY to `certbot certonly --manual
   --preferred-challenges dns --agree-tos --no-eff-email --email <email>
   --cert-name bonesdeploy-<site>-preview -d '*.preview.example.com'`, adding
   `--force-renewal` only for a due valid lineage. Certbot displays and pauses on
   `_acme-challenge.preview.example.com TXT <current token>`. Output says a rerun
   can replace the token: remove completed/abandoned values, preserve unrelated
   active challenges, and publish the current process's token. Tokens are never
   persisted by BonesDeploy.
7. Before gateway changes, invoke the privileged BonesRemote route reconciler in
   stage-only mode with the validated desired domain. It atomically regenerates
   the route include from `SiteState` without reloading; the first setup writes
   an empty projection, while reruns preserve every ready/publishing route.
   Then invoke focused BonesInfra preview provisioning for the stable wildcard
   gateway, route-include parent, directories, native systemd/AppArmor templates,
   and root-owned runtime descriptor. BonesInfra never replaces route content.
   It snapshots the preview-owned available
   file, enabled-link target, and route include; atomically installs candidates;
   runs the existing safety check; reloads; and verifies Nginx. Validation
   failure restores files without reload. Reload failure restores, validates,
   and reloads the old set. Failed restoration keeps named recovery backups,
   reports critical instructions, and does not stop previously loaded workers.
   Production router files are never edited.
8. Atomically persist `PREVIEW_DOMAIN` and `PREVIEW_RETENTION_DAYS`, then sync the
   remote descriptor. ACME tokens and observed certificate state are not config.

Local `.env` becomes authority when written; remote files and descriptor are
projections. Before that, validated command values are desired state. A rerun
adopts deterministic remote files only when they match the requested domain and
reuses matching certificates/gateways left by interruption.

If remote provisioning succeeds but local writing fails, setup reports the
orphan and requires an identical rerun. If descriptor sync fails after local
write, local remains authoritative, deploy refuses stale remote config, and a
rerun skips satisfied certificate/gateway work and retries sync. Domain mismatch
reconciles to local only with no previews; otherwise it fails closed. Interrupted
Certbot never changes Nginx. Setup never deploys a preview.

### Deployment and runtime

Local deploy synchronizes config, reads optional `--env-file` bytes, and sends a
deny-unknown-fields request containing revision, alias, and environment through
stdin to exactly `sudo -n bonesremote preview deploy --site <site>`. No env file
means an empty one. Revision and alias never enter the privileged command line.

BonesRemote acquires `SiteMutation`, reconciles pending preview state, verifies
setup, rejects Compose, and resolves `<revision>^{commit}`. It chooses the
shortest unused commit prefix of at least 12 hex characters; collision lengthens
only the new ID. A ready preview for the same full commit is reused, and an
optional alias is moved without rebuilding or extending expiry. Aliases are
lowercase DNS labels, cannot equal any retained preview ID, and cannot consist
solely of 12 to 40 hex characters, reserving the immutable commit-host namespace.

The release lifecycle is refactored around an explicit environment and log root
so production and preview call the same stage, export, build, promote, prepare,
and seal operations. Production behavior is unchanged. Preview creates
`previews/<id>/`, atomically writes the empty or supplied environment, creates
preview-local mutable framework paths from the trusted descriptor, and wires
only those paths into the candidate. Build input remains secret-free.

Preview prepare uses the same scripts but not unrestricted `runuser`. A
transient systemd unit runs as the site UID with `PROJECT_ROOT=previews/<id>`, a
clean environment, the candidate release read-only, and only preview environment
and mutable paths writable. `InaccessiblePaths` hides production `current`,
`shared/`, `/srv/conf/<site>`, production sockets, and other preview roots; an
exact per-preview AppArmor profile independently enforces that allowlist.
Preview helpers expose no production paths or service control.

After sealing, BonesRemote creates a root-owned `previews/<id>/release` link,
renders exact instance Nginx/AppArmor files from product templates, reloads
AppArmor, and starts `<site>-preview@<id>.target`. Dynamic native apps receive an
app service and release-specific socket. Every preview receives an Nginx service
and socket under `/run/<site>/previews/<id>/`; static releases are their isolated
origin. Units never load production environment files and must be active with an
existing Nginx socket before publication.

Application and prepare workloads use fixed v1 limits: `CPUQuota=50%`,
`MemoryMax=512M`, `TasksMax=128`, and `LimitNOFILE=1024`, plus existing hardening.
Builds keep existing stricter build-user controls. Journals use host policy;
preview release/build logs are deleted with the preview. Per-preview disk quotas
are an explicit v1 limitation.

### Routing transaction

The stable wildcard gateway returns 404 for unknown names. A generated root-owned
include contains exact TLS server blocks for commit hosts and aliases, each
proxying one preview Nginx socket. Candidate route replacement retains the old
file, runs `nginx -t`, reloads only after validation, and restores/reloads the old
file on failure. One reload moves an alias because it appears in one exact block.

Route files are projections of complete `SiteState` snapshots:

- Publish persists the candidate and aliases as `Publishing`, generates routes
  from `Ready` plus `Publishing`, reloads, then marks it `Ready`.
- Alias movement atomically removes the alias from its old record, adds it to the
  target, marks the target `Publishing`, reloads once, then marks it `Ready`.
- Remove persists `Removing`; those hosts are excluded from generated routes.
  Routes reload before service/resource deletion. Prune uses the same workflow.

A live failure restores route and prior state snapshots. After a crash, the next
setup or mutating preview command regenerates from persisted
`Ready`/`Publishing`/`Removing` state, completes valid publication or withdrawal,
then resumes cleanup. Read-only status reports drift rather than mutating. No
second transaction file is introduced. Production `current`, target, and router
are never touched.

### State, inspection, and retention

`SiteState` schema version 2 adds a default-empty preview map while preserving
`active`, `staged_release`, `decommissioning`, and `tombstone`. Version 1 upgrades
in memory and writes as version 2 on the next mutation; newer versions fail
closed. Malformed preview data wedges reads and uses whole-state quarantine.

Each `PreviewRecord` stores ID, full commit, release, phase, process identity,
temporary context, error, immutable commit host, aliases, creation/expiry, log
root, expected target/units, and socket. Persisted phases are:

```text
Created -> SourceExported -> Built -> Promoted -> Prepared -> Sealed
        -> Publishing -> Ready
        -> Failed | CleanupPending
Ready   -> Removing -> removed
```

Every transition is atomic. Pre-publication failure withdraws new routes, stops
instances, removes preview config/runtime and unreferenced release, and retains
a `Failed` record with logs; incomplete cleanup becomes `CleanupPending`. A
crash leaves PID/start-tick evidence. Remove or prune resumes idempotently after
the lock becomes available.

`preview list` lock-free reads one state snapshot and shows revision, phase,
commit URL, aliases, age, expiry, and service summary. `preview status <preview>`
accepts ID, commit hostname, or alias and adds current DNS, certificate
SAN/expiry, route agreement, unit state, socket state, and interruption/cleanup
diagnostics. At 30 days certificate status warns to rerun setup.

`preview logs` reads
`<bonesremote-site-logs>/previews/<id>/<release>/`, then only that preview's
target/app/Nginx journals. Explicit log roots prevent sequential overwrite.
Remove/prune delete logs; failed records retain them until expiry/removal.

`preview remove` marks `Removing`, withdraws routes, stops the target, removes
only validated instance/AppArmor/Nginx/runtime resources, removes a release only
when no state references it, and removes the record. Missing resources are
success. Failure keeps `CleanupPending`. Production paths are outside cleanup's
allowlist. Production prune protects every preview-referenced release.

Retention accepts 1 through 365 days and defaults to 14. Every record keeps
`expires_at = created_at + retention`; config changes affect later records only,
reuse does not extend expiry, and a failed commit must be removed before retry.
Prune removes expired `Ready`/`Failed` records and retries all `CleanupPending`
records regardless of age. Successful early cleanup leaves a `Failed` diagnostic
until expiry; repeated failure retains combined errors with no retry limit. No
timer is installed.

### Security policy

No preview environment means an empty file. Runtime and prepare systemd policy
plus exact AppArmor profiles hide production environment, shared, current,
control-plane, socket, and other-preview paths. Preview mutable paths are
separate. Existing rootless builds, sealed ownership, site identity, hardening,
Unix-socket ingress, and deployment lock remain in force.

This is not a hostile-code sandbox. Native previews share the site's UID and
host network, so malicious code may attack same-site availability or services
whose credentials the operator supplies. Only trusted revisions are supported;
URLs are public; Docker and fork automation are rejected.

## Approach

1. Add canonical preview config, validation/value types, transports, and mirrored
   Rust/Python paths.
2. Add local/remote preview command groups and a PTY SSH boundary; keep local
   prompting/rendering and remote JSON reports.
3. Add state-derived route staging plus convergent BonesInfra gateway/runtime
   provisioning and manifest ownership.
4. Add schema-v2 preview records and operations through `SiteMutation`.
5. Extract shared release preparation with explicit environment, prepare-runner,
   and log policies; retain separate production activation and preview publish.
6. Add focused preview runtime and route reconcilers using validated IDs,
   product-owned templates, and state-derived hosts.
7. Protect preview releases; add idempotent remove and age-based prune.
8. Rename Quick Tunnel status terminology and update focused tests/docs.

## Responsibilities and boundaries

- `bonesdeploy-core` owns config, defaults, value validation, transport, and
  canonical paths; Python `DeploymentPaths` mirrors server layout.
- Local `commands::preview` owns interaction, DNS guidance, environment input,
  SSH selection, and rendering; `infra::ssh` owns PTY/fixed-stdin transport.
- BonesInfra owns stable gateway files, trusted systemd/AppArmor/runtime assets,
  directories, manifests, and safe provisioning reconciliation.
- Certbot owns ACME interaction, lineage, key permissions, and token lifetime.
  BonesDeploy starts manual mode but never edits DNS or persists tokens.
- Remote `commands::preview` owns workflows/reports and never calls BonesInfra.
- Existing lifecycle stages own release preparation; preview supplies explicit
  policies and a post-seal publication, not another build pipeline.
- `SiteState` owns preview identity/lifecycle/aliases/retention/resources.
  Systemd, sockets, DNS, certificates, and Nginx are observed projections.
- `SiteMutation` is the only mutation guard. Preview runtime and route
  reconcilers own dynamic instances and state-to-Nginx projection respectively.

## Affected areas

- `bonesdeploy-core` config/transport/validation/path modules and tests.
- Local CLI args/dispatch, new focused `commands/preview/`, SSH, and transport.
- Remote CLI/dispatch, `commands/preview/`, lifecycle/state/mutation/prune, and
  trusted preview runtime/routing assets.
- BonesInfra request/context/paths, preview command, Nginx/systemd/AppArmor
  assets, framework runtime descriptors, manifests, and pytest suites.
- Global sudoers, adding only exact `preview deploy --site <site>` stdin input.
- Existing status models/tests for explicit Quick Tunnel naming.
- README, context, architecture, security, command, and operations documentation.

## Decisions

- Use wildcard DNS, manual Certbot DNS-01, and separate system Nginx preview
  gateway; no named tunnel, provider credentials, or production-router changes.
- Use one local Nginx socket per preview and trusted template instances. Remote
  code never executes unit/config definitions from the deployed repository.
- Reuse one release-preparation implementation with explicit policies.
- Keep previews in `SiteState`, aliases on target records, IDs at a collision-
  lengthened minimum 12 hex characters, and the commit-ID namespace reserved.
- Default environment to empty; `--env-file` is explicit preview-safe input.
- Support native only; Compose needs distinct project/volume/network policy.
- Treat public access and trusted revisions as explicit v1 product policy.
- Default expiry to 14 days, certificate warning/renewal to 30 days, and require
  command-driven prune and renewal.
- Use the restricted deploy identity for exact stdin preview deploy; setup and
  administration use the existing privileged SSH identity.

## Risks

- Shared-lifecycle refactoring can regress production deployment/rollback.
- Trusted prepare scripts can cause external side effects even without implicit
  production credentials.
- Same-site UID/network means no hostile-code or same-site availability claim.
- Rust/Python framework descriptor drift can prevent runtime startup.
- State and Nginx cannot share one rename; phase-driven recovery is essential.
- Host-wide Nginx reload requires fixed templates, full validation, restoration,
  recovery artifacts, and manual validation of forced failure paths.
- DNS cache/propagation can be stale; Certbot remains challenge authority.
- Manual certificates can expire despite warnings.
- Quick Tunnel status-field rename may affect internal JSON consumers.
- Multiple previews consume disk; v1 has retention but no disk quota.

## Validation

- Core tests: config/schema migration, value/path/unit validation, malicious
  inputs, alias/commit namespace, prefix collision, and state quarantine.
- CLI/SSH tests: command grammar, exact remote forms, stdin requests,
  environment defaults, rendering, PTY construction, and Quick Tunnel wording.
- Lifecycle tests: one preparation pipeline, confined preview prepare paths,
  explicit logs/environment, unchanged production activation/rollback/cancel,
  no `current` mutation, and preview release protection.
- Runtime tests: exact paths/units/profiles, production/other-preview denial,
  fixed resource limits, socket readiness, and allowlisted cleanup.
- Routing tests: exact and wildcard precedence over regex servers, unknown-host
  404, alias/commit collision rejection, one-reload alias moves, all crash
  boundaries, validation/reload/restore failures, and no production edits.
- BonesInfra tests: convergent setup, orphan adoption, local/sync failure reruns,
  certificate exact-SAN reuse/renewal, framework descriptors, manifests, Nginx
  backup/recovery, and Compose rejection.
- Inspection tests: every phase, expiry rule, DNS/certificate/route/unit/socket
  drift, per-preview logs, removal, cleanup retry, and prune.
- Manual disposable-server validation: wildcard A/AAAA/CNAME and TXT flow,
  interrupted setup, two previews beside production, alias movement, expiry,
  logs/remove/prune, renewal warning, and forced Nginx failures while production
  remains reachable.
- Run affected Rust/Python tests, `ruff check .`, `ruff format .`,
  `uv run pytest`, `cargo fmt`, `cargo clippy`, and `shfmt -w .`; never run e2e.
- Review for one state model/lock/preparation pipeline, no provider credentials,
  no production-secret fallback, no Quick Tunnel coupling, and no scope creep.
