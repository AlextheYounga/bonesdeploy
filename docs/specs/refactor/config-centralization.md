# Config Centralization

<!-- Merged from refactor/config-centralization -->

## Idea

### Request

Centralize configuration and environment handling without introducing another
local configuration file. Keep the project-root `.env` for BonesDeploy and
BonesInfra inputs, keep `.env.build` for committed build inputs, and keep the
GPG-encrypted local runtime environment as the source for application and
service credentials.

Make remote actions use explicit configuration rather than inferring intent
from remote directories, sockets, or service names. Maintain a sanitized,
root-owned remote `bones.json` at `/srv/conf/<site>/bones.json` as the
synchronized control-plane copy used by remote-only BonesRemote actions.

### Problem

Configuration is currently represented in several incompatible ways. Rust and
Python independently parse the project-root `.env`; each command forwards a
different subset of its values; deployment sends a temporary descriptor while
doctor reconstructs runtime intent from remote filesystem state; and remote
commands do not have one persisted control-plane configuration to validate.

Application environment ownership is also split incorrectly. Database and
other service provisioning scripts generate or append credentials directly to
remote `shared/.env`, while `secrets push` later replaces that file from the
encrypted local environment. This can erase generated credentials and makes
remote provisioning the accidental source of application secrets.

These boundaries produce failures such as a native Vue site being treated as a
Docker Laravel site because `/run/<project>` exists for native systemd
services. They also make configuration drift and command behavior difficult to
understand.

### Definitions

#### Local environment

`LocalEnvironment` is the project-root, gitignored `.env`, conceptually
`.env.local`. It is the application's local environment and may contain blank
or development-specific values. It also contains one BonesDeploy-managed,
comment-delimited `BONES_*` block with the project identity, connection
settings, provisioning choices, deployment settings, runtime selection,
service selection, and framework-specific scalar values required locally by
BonesDeploy and BonesInfra.

The managed block is excluded from the production environment. Its `BONES_*`
namespace makes this separation machine-verifiable; the comment delimiters make
ownership clear to users. Application-owned values outside the block are never
replaced by initialization.

#### Production environment

`ProductionEnvironment` is the application runtime environment, conceptually
`.env.production`. Its encrypted local representation is
`infra/secrets/.env.gpg`; its published remote representation is `shared/.env`.
It contains application runtime keys only, including generated service
credentials, and never contains the local `BONES_*` configuration block.

On first initialization it has the same application key set as the local
environment, with framework production defaults and generated service values
taking precedence. Values may differ from the local environment. Later
initialization never reads or modifies an existing encrypted production
environment.

#### `.env.build`

The committed, non-secret environment file for build scripts. It is separate
from control-plane configuration and is available in the build context.

#### Remote `bones.json`

A sanitized, root-owned JSON control-plane snapshot stored at
`/srv/conf/<site>/bones.json`. The managed `BONES_*` block in local `.env` is
authoritative; the remote file is its synchronized operational copy for
remote-only BonesRemote commands. It contains no application values, secrets,
private keys, or connection credentials.

#### Provisioning request

A typed request created by BonesDeploy from the managed local configuration
block and, for service provisioning, the locally decrypted production
environment. It is sent to the local BonesInfra process, which performs PyInfra
operations against the remote host. It is not a persisted configuration file.

Server-only provisioning uses `ServerContext`; site, service, runtime, SSL,
and manifest provisioning uses the full site context. `ServerContext` contains
only the connection values needed to reach the host.

#### Deployment descriptor

A typed, sanitized request created from the managed local configuration block
for one BonesRemote deployment or remote validation action. It carries the
runtime and deployment values required by that action, while project paths and
identities remain derived from project identity and connection details remain
local.

The existing `RemoteDeploymentConfig` and `--config-stdin` deployment protocol
are the starting point for this boundary. The change extends and reuses that
typed transport rather than creating a second descriptor format.

#### Publish

The explicit `bonesdeploy secrets push` operation that atomically replaces the
remote site `shared/.env` with the decrypted production environment.
Provisioning does not publish this file automatically.

#### Framework-specific values

Managed framework scalar settings use `BONES_*` names in the local configuration
block, such as `BONES_PHP_VERSION`, `BONES_RUBY_VERSION`, or
`BONES_IS_STATIC`. Rust projects them into the existing `Runtime.extra` map
with their unprefixed logical names. Application environment keys are not
treated as framework settings.

### Desired outcome

Rust is the sole parser and validator for the project-root `.env`. BonesInfra
receives typed provisioning requests instead of independently parsing the root
`.env`.

Every config-dependent BonesRemote action receives or reads the same
sanitized, validated control-plane shape. Local BonesDeploy commands update the
remote `bones.json` atomically before remote-only actions rely on it. Remote
doctor validates the explicit runtime backend and deployment settings instead
of inferring them from `/run/<project>`.

During first initialization, framework keys are merged into the existing local
environment without replacing existing application values. A production
environment is then derived from its application keys, excluding the managed
`BONES_*` block. Service credentials and framework-native service settings are
added to that production environment before it is encrypted. Production service
values override conflicting framework defaults; local service values are
preserved when present and added blank when absent.
For Laravel with PostgreSQL, this includes `DB_CONNECTION=pgsql`,
`DB_HOST=127.0.0.1`, `DB_PORT=5432`, `DB_DATABASE`, `DB_USERNAME`, and
`DB_PASSWORD`. `APP_KEY` remains blank for the user to generate locally and
enter through `bonesdeploy secrets`.

If `infra/secrets/.env.gpg` already exists, initialization returns without
reading or changing it. Users add service settings later through
`bonesdeploy secrets edit`. BonesInfra configures remote services from values
supplied by the local provisioning request and never generates, appends, or
replaces application environment values on the remote host. `shared/.env` is
written only by explicit `secrets push`.

Existing projects retain the root `.env`, `.env.build`, and
`infra/secrets/.env.gpg` layout. No local `bones.json` or other replacement
configuration file is created.

### Scope

This change includes:

- One Rust-owned parser boundary that separates the local application
  environment from the managed `BONES_*` configuration block in root `.env`.
- Typed provisioning requests from BonesDeploy to BonesInfra.
- Typed deployment and doctor descriptors for BonesRemote.
- Atomic creation and update of sanitized remote `bones.json` state.
- Remote-only BonesRemote commands reading the synchronized remote control-plane
  configuration.
- First-init generation and encrypted storage of database and service
  credentials plus framework-native service settings.
- Passing required decrypted service values to BonesInfra without command-line
  arguments, persistent plaintext configuration files, or log output.
- Service provisioning that consumes supplied credentials and never modifies
  `shared/.env`.
- Regression coverage for native and Docker runtime selection, descriptor
  validation, remote config synchronization, and secret ownership.
- Documentation describing the resulting configuration boundaries.

### Constraints

- Do not introduce any replacement local configuration file.
- Keep root `.env` as the local application environment plus its managed local
  BonesDeploy block, and keep `.env.build` as the committed build input.
- Preserve application-owned root `.env` content and values during init. Update
  only the explicitly delimited `BONES_*` block, atomically.
- Do not include `BONES_*` keys in the production environment or publish them
  to remote `shared/.env`.
- Keep application and service credentials encrypted locally until explicit
  publication or controlled provisioning use.
- Never expose secrets in command arguments, Git, build inputs, or ordinary
  provisioning output.
- `project_name` remains explicit input; paths, users, groups, and unit names
  derived from it continue to use existing conventions.
- `shared/.env` remains the one remote application environment file and is
  replaced only by `secrets push`.
- First initialization renders the framework environment, injects selected
  service defaults, validates it, and encrypts it. Later initialization never
  reads or modifies an existing encrypted environment.
- Redis and Valkey use port `6379` by default. Provisioning fails when the
  requested port is occupied; it never selects a replacement port.
- Do not run E2E tests during implementation validation.

### Exclusions

This change does not replace `.env.build`, redesign build-script variables, or
add automatic secret publication. It does not introduce a second remote
application environment file, a new local configuration format, or a general
remote configuration database. It does not change release locking, activation,
rollback, or application deployment semantics beyond their configuration
transport. It does not migrate service credentials or ports from existing
remote sites; existing deployments in scope do not use these managed services.


## Plan

### Current behavior

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

### Intended behavior

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

### Approach

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

### Responsibilities and boundaries

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

### Affected areas

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

### Decisions

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

### Risks

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

### Validation

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


## Tasks

### Implementation

- [x] Define canonical stdin typed projections for server and site provisioning,
  remote control-plane synchronization, deployment/doctor, and service
  credential inputs in `bonesdeploy-core`, reusing `RemoteDeploymentConfig`.
- [x] Preserve the `ServerContext` boundary for server-only provisioning and
  retain framework-specific values in validated `Runtime.extra` projections.
- [x] Model `LocalEnvironment` and `ProductionEnvironment` in Rust. Parse the
  root `.env` once, separate the comment-delimited `BONES_*` block from local
  application keys, and atomically merge that block without replacing
  application-owned content.
- [x] Remove `DeployContext.from_files()`, `ServerContext.from_files()`,
  `read_dotenv()`, and duplicated `config/keys.py` root configuration parsing,
  making every BonesInfra command entrypoint consume a typed stdin request from
  BonesDeploy.
- [x] Add secret-safe local transport from decrypted `infra/secrets/.env.gpg`
  values into BonesInfra, replacing the current `bonesinfra::run(&[&str])`
  `--env-file`-only boundary without command-line or persistent plaintext
  exposure.
- [x] Extend first-time environment rendering to preserve existing local
  application values, add missing framework and blank local service keys, then
  derive a production environment without `BONES_*` keys. Inject generated
  service credentials and framework-native production settings before
  encryption. For Laravel PostgreSQL, inject `DB_CONNECTION`, `DB_HOST`,
  `DB_PORT`, `DB_DATABASE`, `DB_USERNAME`, and `DB_PASSWORD`; leave `APP_KEY`
  blank. Preserve the existing-file early return so later initialization never
  modifies the encrypted production environment.
- [x] Change PostgreSQL, MySQL/MariaDB, MongoDB project-user, Redis, and Valkey
  provisioning to consume supplied credentials and stop generating or modifying
  remote `shared/.env`; retain MongoDB admin credentials as protected remote
  machine state.
- [x] Implement sanitized remote `/srv/conf/<site>/bones.json` creation,
  validation, atomic replacement, and loading for remote-only BonesRemote
  actions.
- [x] Synchronize the remote control-plane descriptor before local
  config-dependent remote actions and update direct remote command handling to
  use the synchronized file, reusing the existing `--config-stdin` transport.
- [x] Update site doctor and related remote checks to use explicit runtime and
  deployment configuration, including exact backend and branch validation.
- [x] Remove `/run/<site>` runtime-backend inference and stale configuration
  fallback paths.
- [x] Unify the branch default to `main` across `App::default()`, dotenv
  loading, and Python projections.
- [x] Update command workflows, architecture documentation, context files,
  README guidance, and embedded skill documentation for the resulting
  configuration boundaries.

### Validation

- [x] Add Rust tests covering managed `BONES_*` block replacement, local
  application-content preservation, production-environment filtering, parser
  projections, native/Docker descriptor round trips, derived values, and remote
  control-plane serialization.
- [x] Add Python tests covering typed provisioning input and service operations
  that never write `shared/.env`.
- [x] Add secret-flow tests covering first-init local/production environment
  construction, service-value precedence in production, blank local service
  additions, Laravel native PostgreSQL values, blank `APP_KEY`, no published
  `BONES_*` values, and untouched existing encrypted files.
- [x] Add remote doctor tests proving native sites do not run Docker checks and
  Docker sites do run them from explicit configuration.
- [x] Add remote state tests covering atomic `bones.json` replacement, schema
  validation, and direct remote command reads.
- [x] Run `cargo fmt` and `cargo clippy`.
- [x] Run `cargo test --workspace --exclude e2e`.
- [x] Run `ruff format .`, `ruff check .`, and `uv run pytest`.
- [x] Run `shfmt -w .` and review generated changes.
- [x] Confirm E2E tests were not run.

### Completion

- [x] Remove obsolete Python root `.env` readers, remote environment writers,
  `/run/<site>` inference, legacy `--env-file` transport, and dead transport
  code.
- [x] Review `CONTEXT.md`, `crates/bonesinfra/python/CONTEXT.md`, `README.md`,
  `docs/ARCHITECTURE.md`, and `docs/architecture/reference.md` for consistency
  with the finalized configuration model.
- [x] Review the final diff for secret exposure, duplicate sources of truth,
  accidental local application-value replacement, published `BONES_*` keys,
  stale local TOML references, and unresolved configuration ownership.

### Completion notes

Implemented across three waves:

- `bonesdeploy-core`: managed `BONES_*` block grammar (`local_environment.rs`),
  atomic block-preserving writer replacing whole-file save (`write_local_environment`),
  `ParsedDotEnv`/`LoadedLocal`/`load_local`, `production_application_keys`
  filtering, typed stdin payloads (`requests.rs`: `ProvisioningRequest`,
  `ServerConnection`, `SiteFields`, `ServicesRequest`), branch default unified
  to `main`, and `RemoteDeploymentConfig.services`.
- `bonesinfra`: Rust bridge gained `run_with_request` (JSON on stdin); Python
  CLI commands take `--request-stdin`; `DeployContext.from_request`/
  `ServerContext.from_request` in new `config/request.py`;
  `from_files()`/`read_dotenv()`/`config/keys.py` deleted; PostgreSQL, MySQL,
  MongoDB, Valkey, and Redis templates consume supplied credentials and never
  write `shared/.env`; port conflicts fail on the requested port.
- `bonesremote`: `config sync --site <site> --config-stdin` validates and
  atomically installs root-owned `/srv/conf/<site>/bones.json`; doctor loads it,
  runs Docker checks only for an explicit Docker backend (the `/run/<site>`
  heuristic is removed) and validates the exact configured branch ref.
- `bonesdeploy`: all BonesInfra invocations send typed requests over stdin;
  deploy and site doctor synchronize the control-plane snapshot before remote
  actions; first initialization merges framework/blank service keys into the
  local environment without touching application values, derives the production
  environment excluding every `BONES_*` key, injects generated service
  credentials plus Laravel-native PostgreSQL settings with a blank `APP_KEY`,
  keeps the existing-encrypted-file early return; SSL rewrite preserves local
  application content.

Validation executed: `cargo fmt`, `cargo clippy --workspace --all-targets
--exclude e2e` (zero warnings), `cargo test --workspace --exclude e2e` (green),
`uv run pytest` (417 passed), `ruff format`/`ruff check`, `shfmt -w .`. E2E
tests were not run.


## Clarification

### Trigger

The service environment contract was clarified: Redis and Valkey use the
normal default port, service values are injected only during first
initialization, and Laravel must receive framework-native database settings
before its encrypted environment is created.

### Decision

Use port `6379` for Redis and Valkey by default. Provisioning fails when the
requested port is occupied and never selects a replacement automatically.

On first initialization, BonesDeploy renders the framework environment,
injects values for services selected in the root `.env`, validates the complete
environment, and encrypts it. For Laravel with PostgreSQL, the injected values
include `DB_CONNECTION=pgsql`, `DB_HOST=127.0.0.1`, `DB_PORT=5432`,
`DB_DATABASE`, `DB_USERNAME`, and generated `DB_PASSWORD`. `APP_KEY` remains
blank; the user runs `app:key-generate` locally and enters the result through
`bonesdeploy secrets`.

If `infra/secrets/.env.gpg` already exists, initialization returns without
reading or changing it. Later service additions are handled manually through
`bonesdeploy secrets edit`. Existing remote service values are not migrated.

Store the sanitized remote control-plane copy as validated JSON at
`/srv/conf/<site>/bones.json`, atomically replaced by BonesDeploy and read by
remote-only BonesRemote actions.

### Supersedes

This replaces the earlier plan for encrypted-environment reconciliation, the
unsettled Redis/Valkey port allocation, and the `bones.toml` filename and
location. It adds the framework-native Laravel PostgreSQL injection contract.

### Effect on the record

`01-idea.md` now defines first-init-only injection, port `6379`, no service
migration, Laravel-native PostgreSQL values, and remote `bones.json`.

`02-plan.md` now specifies the single first-init flow, fail-on-port-conflict
behavior, and `/srv/conf/<site>/bones.json` synchronization.

`03-tasks.md` now tracks first-init injection and `bones.json` validation
instead of reconciliation and `bones.toml` work.


## Clarification

### Trigger

The root `.env` must also serve as the application's local environment. It
must not be replaced when it already exists. The encrypted runtime environment
is conceptually the production environment, and local-only setup values must
not be included in it.

### Decision

Model two explicit concepts in Rust and related code:

- `LocalEnvironment` is root `.env`, conceptually `.env.local`. It contains
  application-local values plus a comment-delimited, BonesDeploy-managed
  `BONES_*` block.
- `ProductionEnvironment` is the plaintext represented by
  `infra/secrets/.env.gpg`, conceptually `.env.production`, and published as
  remote `shared/.env`. It contains application values only.

Initialization preserves all application-owned local text and values. It adds
missing framework keys and blank local service keys, then atomically replaces
the managed `BONES_*` block. The production environment is derived from the
local application key set after removing `BONES_*` keys. Generated service
values override framework defaults in production; existing local values are
never overwritten.

The `BONES_*` namespace is the machine-enforced production exclusion boundary.
The comment delimiters make managed ownership clear to users. Rust remains the
sole root `.env` parser, and BonesInfra receives stdin typed requests rather
than a file path.

### Supersedes

This replaces the earlier description of root `.env` as a control-plane-only
file. It refines first-init injection so only production service values override
framework defaults; local application values remain user-owned.

### Effect on the record

`01-idea.md` now defines local and production environments, the managed
`BONES_*` block, and the no-overwrite local merge rule.

`02-plan.md` now requires Rust-owned environment separation, atomic managed
block replacement, stdin requests for all BonesInfra commands, and production
filtering of `BONES_*` values.

`03-tasks.md` now tracks environment modeling, managed-block merge behavior,
legacy file-transport removal, and local/production boundary regression tests.
