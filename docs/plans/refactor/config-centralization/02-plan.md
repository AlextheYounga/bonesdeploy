# Plan

## Current behavior

`bonesdeploy-core` parses the root `.env` into `config::Bones` and treats every
unrecognized key as a framework runtime value. `save()` replaces the entire
file. Rust commands use that value directly, while `DeployContext.from_files()`
independently parses the same file through Python `read_dotenv()`. The Rust and
Python implementations duplicate keys, defaults, and validation. Python also
mirrors the Rust environment-key vocabulary in `config/keys.py`.

`bonesdeploy deploy` projects selected values into the existing
`RemoteDeploymentConfig` and sends JSON over SSH stdin using
`bonesremote deploy --config-stdin`. BonesRemote combines that descriptor with
site-derived identity and paths for the deployment lifecycle.
`bonesdeploy site doctor` does not send the descriptor. Its remote
doctor reconstructs a minimal configuration from the site name and currently
uses `/run/<site>` as a Docker-runtime heuristic. Native nginx and application
services also create that directory, so the heuristic is invalid.

BonesInfra commands currently receive `--env-file .env` and
`DeployContext.from_files()` parses it independently. The service templates
`configure-postgres-project.sh.j2`, `configure-mysql-project.sh.j2`,
`create-mongodb-project-user.sh.j2`, and `setup-key-value-store.sh.j2` read or
modify remote `shared/.env`; they generate missing values and append them to
that file. MongoDB additionally generates the protected remote
`bonesinfra_admin` credential in its configured admin file.
`bonesdeploy secrets push` atomically replaces the file with the decrypted
`infra/secrets/.env.gpg` content, so generated values can be lost.

`secrets::initialize_defaults()` currently returns immediately when
`infra/secrets/.env.gpg` exists. It therefore cannot add credentials required
by services selected after initial setup. Laravel establishes the manual
secret-entry pattern: the user runs `app:key-generate` locally, then adds the
result through `bonesdeploy secrets`/`secrets edit`. Service values are created
only during first initialization; later additions are user-managed through
`bonesdeploy secrets edit`.

The Rust bridge currently exposes only `bonesinfra::run(&[&str])`; it appends
command arguments, sets the project working directory, and provides no typed
request or secret input channel. This is the process boundary that must be
replaced without putting decrypted values in command arguments.

Remote-only BonesRemote commands derive identity and paths from `--site`, but
there is no synchronized remote control-plane configuration for runtime,
branch, service, or other desired settings.

## Intended behavior

The project-root `.env` is `LocalEnvironment`: the application's local
environment plus a comment-delimited, BonesDeploy-managed `BONES_*` block.
Existing application-owned content and values are preserved. Rust parses it
once, distinguishes `BONES_*` configuration from application keys, and
projects typed requests for each boundary:

- BonesInfra receives a typed provisioning request containing the local
  provisioning context and the service values required for remote setup.
- BonesRemote receives the existing typed `RemoteDeploymentConfig` for deploy
  and site doctor actions through the established stdin JSON protocol.
- A sanitized remote `bones.json` is atomically synchronized from the local
  descriptor and is used by direct remote-only actions.

Project identity is explicit. Paths, users, groups, service names, and target
names are derived from it using shared conventions rather than copied through
every request. Runtime backend, branch, release retention, build settings, and
framework-specific deployment values are explicit configuration values and are
never inferred from remote filesystem state.

On first initialization, BonesDeploy merges missing framework application keys
into `LocalEnvironment` without replacing local values. It derives
`ProductionEnvironment` from the local application key set, strips the managed
`BONES_*` block, applies production defaults, then injects selected service
credentials and framework-native settings. Service values win over framework
defaults in production, while existing local service values are preserved and
missing local service keys are added blank. The validated production environment
is encrypted. If the encrypted file already exists, initialization returns
before reading or modifying it. Users add settings for services selected later
through `bonesdeploy secrets edit`. The local provisioning request carries
required decrypted values to BonesInfra through a secret-safe local process
boundary. BonesInfra uses those values to configure remote services and does
not write application environment files. `secrets push` remains the only
operation that writes remote `shared/.env`.

## Approach

1. Define shared typed projections for local provisioning, remote control-plane
   synchronization, deployment, doctor, and service credential inputs. Use
   stdin JSON for every BonesInfra request, including the connection-only
   `ServerContext`; keep identity-derived fields out of projections when the
   receiver can derive them safely.
2. Make the Rust configuration loader the sole owner of root `.env` parsing and
   validation. Model `LocalEnvironment` and `ProductionEnvironment`, use a
   namespaced, comment-delimited `BONES_*` block for local-only configuration,
   and atomically merge that block without replacing application-owned local
   content. Replace `DeployContext.from_files()`, `ServerContext.from_files()`,
   `read_dotenv()`, and `config/keys.py` with typed provisioning requests.
3. Extend the local BonesInfra bridge with stdin request transport. Decrypted
   production service values stay in the local process boundary and are passed
   to PyInfra operations without shell arguments or persistent plaintext files.
4. Keep the existing-file early return in production-environment initialization.
   During first initialization only, merge missing framework application keys
   into the local environment, derive the production environment without
   `BONES_*` keys, then inject generated service credentials and framework-native
   settings before encryption. Production service values override framework
   defaults; existing local values are retained and absent local service keys
   are added blank. For Laravel PostgreSQL, inject `DB_CONNECTION=pgsql`,
   `DB_HOST=127.0.0.1`, `DB_PORT=5432`, `DB_DATABASE`, `DB_USERNAME`, and
   `DB_PASSWORD`. Leave `APP_KEY` blank for the user to generate locally with
   `app:key-generate` and enter through the secrets workflow. Later service
   additions are user-managed through `bonesdeploy secrets edit`.
5. Change the four service templates and their Python service modules to
   consume supplied credentials. Keep MongoDB's admin credential as protected
   remote machine state, but move project-user credentials into the encrypted
   local runtime environment. Remove remote application credential generation
   and all writes to `shared/.env` from service provisioning.
6. Add remote control-plane synchronization that validates and atomically writes
   the sanitized `/srv/conf/<site>/bones.json` snapshot before config-dependent
   remote actions.
   Remote-only commands read that file; local commands provide the current
   descriptor before invoking those actions.
7. Change site doctor to use the explicit descriptor or synchronized remote
   configuration. Restore exact branch and runtime-backend checks and remove
   `/run/<site>` Docker inference.
8. Update command workflows, documentation, tests, and embedded skill material
   to describe the one-local-file model and explicit `secrets push` step.

## Responsibilities and boundaries

- `bonesdeploy-core` owns `LocalEnvironment` and `ProductionEnvironment`, the
  root `.env` grammar, managed-block merge and filtering rules, schema,
  validation, derived identity/path rules, and typed request projections.
- `bonesdeploy` owns local file access, GPG decryption/encryption, request
  construction, SSH connection details, and explicit remote action sequencing.
- `bonesinfra` owns PyInfra execution and remote host/service provisioning. It
  consumes typed requests and does not parse the root `.env` or own application
  environment publication.
- `bonesinfra` service modules own remote database/service installation and
  configuration from supplied values. They do not generate application
  credentials or write `shared/.env`.
- `bonesremote` owns remote control-plane validation, synchronized
  `bones.json`, release lifecycle behavior, and remote-only state inspection.
  It derives convention-based identity and paths from project identity.
- `secrets push` owns the atomic replacement of remote `shared/.env` from the
  encrypted local runtime environment.
- `.env.build` remains the source for committed build inputs and is outside the
  control-plane transport. Root `.env` remains a local application file; only
  its managed `BONES_*` block is local control-plane configuration.
- `ServerContext` remains limited to host, SSH user, and port. Full site
  provisioning values are available only to site-scoped requests.

## Affected areas

- `crates/bonesdeploy-core/src/config/` and related tests for the canonical
  parser, local/production environment merge and filtering, validation,
  projections, `RemoteDeploymentConfig`, and service credential values.
- `crates/bonesdeploy/src/commands/secrets/`, provisioning commands, SSH bridge,
  `frameworks/laravel.rs`, E2E secret-generation helper, and command
  descriptors.
- `crates/bonesinfra/src/lib.rs`, `config/context.py`, `config/keys.py`, and
  Python CLI modules for typed provisioning input and secret-safe transport.
- `crates/bonesinfra/python/src/bonesinfra/services/runtime/`, including
  PostgreSQL, MySQL/MariaDB, MongoDB, Redis, and Valkey modules, plus
  `configure-postgres-project.sh.j2`, `configure-mysql-project.sh.j2`,
  `create-mongodb-project-user.sh.j2`, and `setup-key-value-store.sh.j2`.
- `crates/bonesremote/src/commands/doctor/`, remote CLI dispatch, control-plane
  state, and config-dependent release/service commands.
- Existing Rust and Python tests at parser, transport, provisioning, service,
  doctor, and remote-state boundaries.
- `CONTEXT.md`, `crates/bonesinfra/python/CONTEXT.md`, `README.md`,
  `docs/ARCHITECTURE.md`, `docs/architecture/reference.md`, and embedded skill
  documentation.

## Decisions

- Root `.env` is `LocalEnvironment`, not a control-plane-only file. Its
  `BONES_*` block is the sole local control-plane configuration; its remaining
  keys are application-local values. A local TOML format is explicitly rejected
  because it duplicates the existing working `.env` input and creates another
  synchronization burden.
- `infra/secrets/.env.gpg` is `ProductionEnvironment` at rest; its decrypted
  content is the sole production application environment and excludes every
  `BONES_*` key.
- Init preserves existing local application values and comments, adds missing
  framework and blank local service keys, and atomically replaces only the
  comment-delimited managed `BONES_*` block.
- `.env.build` remains a separate committed file because build inputs have a
  different trust and lifecycle boundary from local control-plane values.
- The encrypted local runtime environment is the sole source for application
  and service credentials. Remote provisioning consumes values from it but does
  not create or mutate application environment files.
- `shared/.env` remains one file and one writer: explicit `secrets push`.
- A remote `/srv/conf/<site>/bones.json` is an operational synchronized copy,
  not an additional local source of truth. It is sanitized, root-owned,
  atomically replaced, and refreshed before local config-dependent remote
  actions.
- Project-derived values remain derived. Explicit transport carries policy and
  settings that cannot be reconstructed safely, such as runtime backend and
  branch.
- Framework-specific scalar values continue to use `Runtime.extra`; they are
  not promoted into new top-level configuration files or duplicated Python
  schemas.
- The canonical branch default is `main` across Rust and Python projections;
  the existing `App::default()` `master` inconsistency is corrected as part of
  schema consolidation.
- The existing `RemoteDeploymentConfig` and `--config-stdin` protocol are
  reused for deploy and extended to doctor and control-plane synchronization.
- MongoDB's `bonesinfra_admin` credential remains remote host provisioning
  state; project database credentials are application runtime values and are
  generated locally into the encrypted environment.
- BonesInfra receives stdin typed requests, including the connection-only
  server request, rather than independently parsing the root `.env`, preventing
  Rust/Python schema drift.
- Runtime and service credentials do not enter command arguments, build inputs,
  Git, or ordinary output.
- Redis and Valkey request port `6379` by default. Provisioning validates that
  the requested port is available and fails if it is occupied; it does not
  allocate a different port.
- Existing remote service values are not migrated. The change assumes existing
  deployments use the default service stacks and have no managed services to
  import.
- First initialization injects service values before encryption. An existing
  encrypted environment is never read or modified by later initialization.
- Laravel `APP_KEY` remains a manual value: the user runs `app:key-generate`
  locally and enters the result through `bonesdeploy secrets`.
- Build environment allowlist redesign is excluded; `.env.build` and current
  build behavior remain otherwise unchanged.

## Risks

- A request projection that omits a runtime or service value can make
  provisioning or deployment diverge from local configuration. Projection tests
  must cover every consumer-owned field.
- Remote `bones.json` can become stale when a direct remote-only action runs
  without a preceding local synchronization. Remote commands must validate
  presence and schema, and local commands must synchronize before relying on
  current local intent.
- Secret values can leak through PyInfra operation rendering, process arguments,
  logs, or temporary files. Tests and output review must enforce the chosen
  secret-safe transport.
- Users who add services after first initialization must add the required
  application settings through `bonesdeploy secrets edit`; setup must report
  missing values before remote service changes.
- Removing service writes to `shared/.env` can expose an incomplete encrypted
  environment. Site setup must validate required service keys before remote
  service configuration.
- Incorrect managed-block parsing can replace application-owned local content
  or publish `BONES_*` keys. Merge and production-filter tests must preserve
  comments and values outside the block and prove no local-only key reaches the
  encrypted or remote environment.
- Existing remote sites may lack `bones.json`. Synchronization must create it
  from an explicit local request without changing release or application data.
- Redis and Valkey use the default port `6379` unless the user changes the
  encrypted environment. A port collision stops provisioning and requires a
  user-selected replacement; remote scanning and fallback are prohibited.

## Validation

- Unit-test root `.env` parsing, managed-block replacement, local application
  content preservation, production filtering, typed projections, derived
  identity/path rules, existing `RemoteDeploymentConfig`/`--config-stdin`
  round trips, and native/Docker backends.
- Test that BonesInfra receives typed provisioning values without parsing the
  root `.env`, and that service requests contain required credentials without
  exposing them in command arguments or ordinary output.
- Test first-init local/production environment construction: framework keys and
  blank missing service keys merge into the local file without overwriting its
  values; production strips `BONES_*` keys and uses generated service values;
  Laravel `APP_KEY` remains blank; and an existing encrypted environment is
  untouched on later initialization.
- Test PostgreSQL, MySQL/MariaDB, MongoDB, Redis, and Valkey provisioning uses
  supplied values and does not write `shared/.env`.
- Test remote `bones.json` validation, atomic replacement, site-derived paths,
  and remote-only command reads.
- Test site doctor uses explicit runtime backend and branch values and never
  treats `/run/<site>` alone as evidence of Docker.
- Run `cargo fmt`, `cargo clippy`, `cargo test --workspace --exclude e2e`,
  `ruff format .`, `ruff check .`, `uv run pytest`, and `shfmt -w .` as required
  by repository instructions. Do not run E2E tests.
- Review documentation and the final diff for stale independent Python root
  `.env` parsing, remote environment mutation, runtime inference, or references
  to a local replacement configuration file.
