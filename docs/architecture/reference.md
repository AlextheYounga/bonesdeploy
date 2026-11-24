# Architecture reference

This is the concrete map of the four Rust components. See
[`docs/ARCHITECTURE.md`](../ARCHITECTURE.md) for dependency and placement rules.

## `bonesdeploy-core`

- `config/model.rs`: canonical `Bones`, `Runtime`, `Backup`, build settings,
  runtime backend, defaults, and validation.
- `config/local_env.rs`: sole parser/writer for the project `.env`.
- `config/transport.rs`: the narrow backend-specific descriptor synchronized
  to BonesRemote. Provisioning does not use this transport.
- `paths.rs` and `paths/deployment.rs`: shared constants and the complete typed
  server-side `DeploymentPaths` representation.

## `bonesdeploy`

- `cli/`: Clap arguments and async dispatch.
- `commands/server/`: server setup, helpers, and server doctor orchestration.
- `commands/site/`: site setup, runtime, manifest, deletion, SSL, Quick Tunnel,
  status, doctor, import, export, and releases.
- `commands/update/`: binary updates, template refresh, and native
  version-gated patch calls.
- `infra/ssh.rs`: local-to-server SSH transport for BonesRemote and read-only
  command integrations. It is not used to recreate provisioning operations.
- `infra/git.rs`: local source provenance and Git operations.
- `build/`: local native/Compose artifact production.

Provisioning handlers load `Bones` and call `bonesinfra` functions directly.
There is no provisioning request JSON or child process.

## `bonesinfra`

### Foundation

- `context.rs`: validated `ServerContext` and `ProvisioningContext` derived
  directly from `Bones`, plus structured template data.
- `frameworks/<name>.rs`: validated declarative runtime plans for built-in
  frameworks.
- `execution.rs`: RustInfra inventory construction, SSH execution, and stable
  provisioning reports.
- `templates.rs`: traversal-safe resolution against project-local
  `infra/templates/` only.
- `lib.rs`: embedded template snapshot materialization, framework pruning, and
  safe rejection of legacy `infra/custom/`.

### Frameworks

`src/frameworks/` contains Angular, custom/no-op, Django, Laravel, Next.js,
Nuxt, Rails, SvelteKit, and Vue modules. A module validates its typed extra
values and returns the framework runtime plan used by both runtime provisioning
and manifest declarations.

Framework plans identify static roots or application services, language
runtimes, placeholders, proxy shape, AppArmor execution/writable paths, shared
directories, and validation commands. Docker backend selection bypasses native
framework provisioning and uses the Compose plan.

### Provisioning

- `provisioning/server.rs`: supported-host fact validation, baseline packages,
  etckeeper, AppArmor, firewall, fail2ban, unattended upgrades, deploy identity,
  SSH hardening, BonesRemote installation, and sudoers.
- `provisioning/helpers.rs`: optional administrator helper packages.
- `provisioning/site.rs`: runtime identity, canonical directories, and
  placeholder release.
- `provisioning/backup.rs`: encrypted Borg repository, passphrase, and backup
  cron provisioning.
- `provisioning/operations.rs`: shared named operation registration, shell
  registration, and argument quoting.
- `provisioning/runtime/`: native framework and Compose orchestration, language
  runtimes, AppArmor, nginx, systemd targets/services, placeholders, and
  validation.
- `provisioning/ssl.rs`: Certbot webroot issuance and staged TLS router update.
- `provisioning/tunnel.rs`: explicit Cloudflared install/start/stop lifecycle,
  independent of the normal site target.
- `provisioning/deletion.rs`: validated service-first teardown.

Every mutating plan uses RustInfra operations and finishes with etckeeper where
the previous provisioning behavior did. The backup passphrase uses a
domain-specific stateful operation that uploads in-memory content without
  placing the secret in a workstation file or shell argument.

Public provisioning APIs use `prepare_*` for offline plan construction and
`apply_*` for execution. Shared path composition belongs to
`bonesdeploy-core::paths`; cross-process config/resource names belong to
`bonesdeploy-core::names`.

### Manifest

- `manifest/declarations.rs`: common, backend, service, and framework-derived
  artifact/service inventory.
- `manifest/inspection.rs`: read-only RustInfra filesystem, command, and
  systemd facts; typed strategy/artifact/service/Compose reports.
- `manifest.rs`: stable deletion-plan JSON and path/service validation.

Artifact states are `present`, `missing`, or `wrong-kind`. Deletion rejects
relative paths, parent traversal, retained Borg data, unrelated system paths,
and non-project service units. A persisted plan is always revalidated against
the current configuration before execution.

### Patches and templates

- `patches.rs`: semantic-version-gated local migration and remote markers.
- `templates/shared/`: shared Linux templates/scripts.
- `templates/frameworks/<name>/`: framework-owned templates.

Materialization copies these sources to `infra/templates/` atomically and
removes legacy project wheel files. No Python-based BonesInfra process, wheel,
pip environment, or subprocess participates in provisioning. Framework
applications may still use distribution Python or another framework runtime.

## `bonesremote`

- `control_plane.rs`: reads the narrow synchronized runtime descriptor.
- `release/`: staging, artifact verification, sealing, activation, rollback,
  pruning, persisted state, and locking.
- `runtime/`: native systemd and Compose deployment-time control.
- `shared/`: secret/environment installation and transactional shared-data
  import/export behavior.
- `commands/doctor/`: root-owned server/site security and readiness checks.

BonesRemote owns deployment-time mutation and locking. BonesInfra owns
provisioning-time desired state. BonesRemote never invokes BonesInfra.

## Main command flows

### Server setup

```text
bonesdeploy server setup
  → load Bones
  → bonesinfra::provisioning::apply_server_setup
  → validate host facts
  → RustInfra plan + SSH execution
  → server doctor
```

### Site setup/runtime

```text
bonesdeploy site setup
  → server readiness
  → bonesinfra::provisioning::apply_site_setup
  → bonesinfra::provisioning::apply_runtime
  → site doctor
```

### Manifest/delete

```text
site manifest
  → bonesinfra::manifest::inspect
  → read-only RustInfra facts
  → typed text or JSON presentation

site delete
  → resolve and display typed DeletionPlan
  → confirmation
  → serialize, parse, and revalidate plan
  → bonesinfra::provisioning::apply_site_delete
  → BonesRemote registration cleanup
```

### Deploy

```text
bonesdeploy deploy
  → local build and artifact
  → sync narrow control-plane descriptor
  → sudo -n bonesremote deploy --site <site>
  → BonesRemote locked release lifecycle
```
