# Idea

## Request

Centralize configuration and environment handling without introducing another
local configuration file. Keep the project-root `.env` for BonesDeploy and
BonesInfra inputs, keep `.env.build` for committed build inputs, and keep the
GPG-encrypted local runtime environment as the source for application and
service credentials.

Make remote actions use explicit configuration rather than inferring intent
from remote directories, sockets, or service names. Maintain a sanitized,
root-owned remote `bones.json` at `/srv/conf/<site>/bones.json` as the
synchronized control-plane copy used by remote-only BonesRemote actions.

## Problem

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

## Definitions

### Local environment

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

### Production environment

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

### `.env.build`

The committed, non-secret environment file for build scripts. It is separate
from control-plane configuration and is available in the build context.

### Remote `bones.json`

A sanitized, root-owned JSON control-plane snapshot stored at
`/srv/conf/<site>/bones.json`. The managed `BONES_*` block in local `.env` is
authoritative; the remote file is its synchronized operational copy for
remote-only BonesRemote commands. It contains no application values, secrets,
private keys, or connection credentials.

### Provisioning request

A typed request created by BonesDeploy from the managed local configuration
block and, for service provisioning, the locally decrypted production
environment. It is sent to the local BonesInfra process, which performs PyInfra
operations against the remote host. It is not a persisted configuration file.

Server-only provisioning uses `ServerContext`; site, service, runtime, SSL,
and manifest provisioning uses the full site context. `ServerContext` contains
only the connection values needed to reach the host.

### Deployment descriptor

A typed, sanitized request created from the managed local configuration block
for one BonesRemote deployment or remote validation action. It carries the
runtime and deployment values required by that action, while project paths and
identities remain derived from project identity and connection details remain
local.

The existing `RemoteDeploymentConfig` and `--config-stdin` deployment protocol
are the starting point for this boundary. The change extends and reuses that
typed transport rather than creating a second descriptor format.

### Publish

The explicit `bonesdeploy secrets push` operation that atomically replaces the
remote site `shared/.env` with the decrypted production environment.
Provisioning does not publish this file automatically.

### Framework-specific values

Managed framework scalar settings use `BONES_*` names in the local configuration
block, such as `BONES_PHP_VERSION`, `BONES_RUBY_VERSION`, or
`BONES_IS_STATIC`. Rust projects them into the existing `Runtime.extra` map
with their unprefixed logical names. Application environment keys are not
treated as framework settings.

## Desired outcome

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

## Scope

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

## Constraints

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

## Exclusions

This change does not replace `.env.build`, redesign build-script variables, or
add automatic secret publication. It does not introduce a second remote
application environment file, a new local configuration format, or a general
remote configuration database. It does not change release locking, activation,
rollback, or application deployment semantics beyond their configuration
transport. It does not migrate service credentials or ports from existing
remote sites; existing deployments in scope do not use these managed services.
