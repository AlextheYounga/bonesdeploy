# Project-Local Infrastructure Refactor

## Request

Move project-owned deployment infrastructure into the application repository
under `infra/`, remove the separate `.bones` configuration repository and its
transport, and make an application Git revision the unit of deployment.

Remove the general-purpose `bones.toml` configuration model rather than
relocating it. Keep only information that has a real consumer in an existing
channel: convention, local `.env`, committed `.env.build`, committed
`infra/` files or infrastructure code, or existing remote machine state.

Deployments remain explicitly user-triggered:

```text
git push
bonesdeploy deploy
```

## Problem

The current project configuration is a symlink from `.bones` to a separate
Git repository under the user's configuration directory. `bonesdeploy push`
publishes that repository to a second remote repository, where BonesRemote
imports it as mutable site state. Local commands, provisioning, and remote
release stages then depend on that separately synchronized copy.

This duplicates Git history, separates application code from the instructions
that build and deploy it, and allows a deployment to combine an application
revision with infrastructure from another revision. It also treats storage
location as a secret boundary even though the encrypted secret mechanism is
the actual protection required.

## Definitions

**Project identity** is the explicit site name supplied to BonesDeploy and
BonesRemote, such as `checkmyslop`. It is used to derive the canonical bare
repository, site root, runtime identity, build identity, and systemd target. It
is not inferred from an incidental directory name and is not duplicated in a
remote project configuration file.

**Project infrastructure** is committed, non-machine-specific behavior and
assets under `infra/`, including deployment scripts, templates, and project
infrastructure Python modules. It is part of normal application Git history.

**Managed provisioning** is the project-facing provisioning implementation
under `infra/provision/core/`. BonesDeploy supplies and maintains it, and an
explicit BonesDeploy update may refresh it. It is visible and committed in the
application repository, but users should customize `custom/` instead.

**Custom provisioning** is project-owned provisioning code under
`infra/provision/custom/`. BonesDeploy updates preserve it and never silently
overwrite it. Core provisioning runs before custom provisioning through
ordinary explicit Python composition; no general plugin registry is required.

**Provisioning input** is information consumed only by the local
`bonesdeploy` process while it invokes BonesInfra to change a remote machine.
Connection, domain, TLS, and similar values are local `.env` inputs and are
not required by remote deployment.

**Deployment revision** is the full commit SHA resolved by BonesRemote from
the requested branch or revision in the application bare repository. The
revision supplies the application source, `infra/` deployment instructions,
and `.env.build` values used for that deployment.

**Machine state** is durable state created or maintained on the server:
repositories, releases, shared runtime data, locks, deployment records, logs,
caches, identities, service configuration, and runtime secrets. It is not a
second copy of project infrastructure.

**Encrypted secret material** is the existing GPG-encrypted project secret
file. It may be committed under `infra/secrets/`; plaintext runtime secrets
remain protected server-side in shared state and are never supplied to builds.

**Local key material** is the machine-local GPG keyring, including private
keys, trust data, and other decryption authority. It is never stored in the
application repository, `infra/`, or remote Git repositories. The refactor
preserves the BonesDeploy local data/configuration roots needed to store this
keyring and other non-project application state.

**Managed shared state** is the server-side `shared/` directory used by
releases. BonesDeploy always creates and wires `shared/.env`; it may create
framework-declared directories referenced by local environment values. It does
not create application data files such as SQLite databases.

## Desired outcome

A project contains ordinary committed infrastructure content:

```text
project/
├── infra/
│   ├── provision/
│   │   ├── core/
│   │   └── custom/
│   ├── deployment/
│   ├── templates/
│   ├── secrets/
│   └── infrastructure code and assets
├── .env
├── .env.build
└── application source
```

There is no `.bones` symlink, nested infrastructure Git repository,
per-project configuration copy under `~/.config/bonesdeploy/projects`,
infrastructure remote repository, infrastructure-specific Git hook, or separate
`bonesdeploy push` step.

For `bonesdeploy deploy`, BonesRemote resolves one immutable deployment
revision, reads the deployment instructions and scalar build inputs from that
revision, and runs the existing staged release lifecycle. It does not reload
project configuration from mutable remote site state.

The build context contains only source and build-safe inputs. It excludes
`infra/secrets/`, plaintext runtime secrets, decryption keys, `shared/.env`,
and other runtime credentials. The existing GPG workflow remains in place;
this refactor changes its project path only where required by `infra/`.

## Scope

- Remove the `.bones` local and remote configuration repositories and their
  synchronization commands, hooks, and site import/receive transport.
- Audit every current `Bones` and `bones.toml` consumer and eliminate or move
  each value to convention, local `.env`, `.env.build`, committed `infra/`, or
  machine state.
- Make `init`, setup, doctor, manifest, runtime provisioning, services,
  SSL, secrets, deploy, rollback, status, and update use the new boundaries.
- Make project-facing provisioning visible in `infra/provision/core/` and
  `infra/provision/custom/`, with explicit ownership and update behavior.
- Make BonesRemote obtain deployment-specific instructions from the requested
  application Git revision while preserving the existing release safety,
  locking, cancellation, and activation behavior.
- Remove deploy-on-push behavior and its application post-receive trigger.
- Provide a safe migration from an existing `.bones` tree to `infra/` without
  creating commits or exposing plaintext secrets.
- Keep project-local infrastructure code and assets in `infra/` and retain
  the existing BonesInfra framework behavior behind that boundary.

## Constraints

- The application Git revision is the only project revision deployed.
- Remote deployment may use only the target revision, existing machine state,
  and explicit command arguments after local control ends.
- `.env.build` is the single committed scalar channel for values needed after
  control moves to the remote machine; `BONES_*` names remain reserved for
  derived/internal values.
- Local provisioning values remain uncommitted `.env` inputs.
- Runtime secrets never enter the build environment, and plaintext secrets are
  never committed.
- `shared/.env` is the only shared file created and release-wired by
  BonesDeploy. Other application files are created by the application.
- GPG private keys and other decryption authority remain machine-local. Do not
  delete or relocate the local BonesDeploy data/configuration roots solely
  because the old per-project configuration repository is removed.
- Project identity remains explicit and is validated at trust boundaries.
- `infra/provision/core/` is BonesDeploy-managed and updateable. An update must
  detect modified managed files and refuse to overwrite them silently.
- `infra/provision/custom/` is project-owned and preserved by every
  BonesDeploy update.
- Existing rollback behavior remains conceptually unchanged and gains no new
  metadata unless an existing feature demonstrably requires it.
- Preserve current runtime permission behavior; `[runtime.permissions]` is a
  separate follow-up.
- Do not run e2e tests during ordinary implementation validation.
- No backwards compatibility with old `.bones`-based project layouts. Commands
  fail with a clear message if they encounter the old structure; `bonesdeploy
  update` is the single versioned bridge.
- BonesDeploy creates only framework-declared shared directories from local
  environment values. It does not infer or create arbitrary shared files;
  application data files remain application-owned.

## Exclusions

- Redesigning GPG encryption or introducing git-crypt, SOPS, age, or another
  secret system.
- Copying the entire BonesInfra execution engine or its packaging/runtime
  machinery into every project.
- Introducing a general plugin framework or broad ownership subdivisions where
  no managed/project-owned boundary exists.
- Automatically pushing application changes from `bonesdeploy deploy`.
- Redesigning deploy-on-push; it is removed for now.
- Adding a replacement `deploy.toml`, `infra.toml`, or equivalent declarative
  project configuration layer.
- Stripping `infra/` from final runtime release artifacts; that is a later
  hardening change.
- Reworking unrelated deployment behavior or making runtime permissions
  functional as part of this refactor.
- Inferring arbitrary shared files from path-like environment values or adding
  a generic shared-path permission model.
