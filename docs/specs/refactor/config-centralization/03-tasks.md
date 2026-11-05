# Tasks

## Implementation

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

## Validation

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

## Completion

- [x] Remove obsolete Python root `.env` readers, remote environment writers,
  `/run/<site>` inference, legacy `--env-file` transport, and dead transport
  code.
- [x] Review `CONTEXT.md`, `crates/bonesinfra/python/CONTEXT.md`, `README.md`,
  `docs/ARCHITECTURE.md`, and `docs/architecture/reference.md` for consistency
  with the finalized configuration model.
- [x] Review the final diff for secret exposure, duplicate sources of truth,
  accidental local application-value replacement, published `BONES_*` keys,
  stale local TOML references, and unresolved configuration ownership.

## Completion notes

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
