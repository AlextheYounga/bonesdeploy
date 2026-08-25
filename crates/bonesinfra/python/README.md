# bonesinfra

Hidden Python provisioning engine embedded in the BonesDeploy monorepo.

It handles pyinfra-based setup, runtime, and SSL provisioning. BonesDeploy owns
project input and uses this package as the private execution layer.

The Rust `bonesinfra` crate embeds this package into the `bonesdeploy` binary.
For Python-only development, work from `crates/bonesinfra/python`; production
execution uses the embedded copy.

## Interface

- `bonesinfra server apply --env-file <.env> --bonesremote-version <version>`
- `bonesinfra site apply --env-file <.env>`
- `bonesinfra runtime apply --env-file <.env>`
- `bonesinfra ssl apply --env-file <.env>`
- `bonesinfra helpers apply --env-file <.env>`
- `bonesinfra services apply --env-file <.env>`
- `bonesinfra manifest show --env-file <.env>`

Framework template questions are defined by the Rust CLI under
`crates/bonesdeploy/src/frameworks/`. BonesInfra reads the resulting
the project environment and applies infrastructure; it does not prompt for
those settings.

## Project infrastructure

`runtime apply` loads the project-local `infra/` package, imports it alongside
`infra/__init__.py` and `infra/manifest.py`, and calls its `deploy(ctx)`.
The manifest declares framework-owned artifacts, services, and mode for
`manifest show`. `infra/` and its local `templates/` are copied by
`bonesdeploy init` from the canonical framework package maintained and embedded
by BonesInfra. A local package takes precedence as a whole; when it is absent,
BonesInfra loads the selected built-in package.

## Notes

- `bonesinfra` reads the project `.env` and committed `infra/` files.
- It does not create or manage a project configuration file.
- It does not own deployment scripts.
