# bonesinfra

Hidden Python provisioning engine embedded in the BonesDeploy monorepo.

It handles pyinfra-based setup, runtime, and SSL provisioning. BonesDeploy owns TOML creation and uses this package as the private execution layer.

The Rust `bonesinfra` crate embeds this package into the `bonesdeploy` binary.
For Python-only development, work from `crates/bonesinfra/python`; production
execution uses the embedded copy.

## Interface

- `bonesinfra setup apply --config <bones.toml>`
- `bonesinfra runtime apply --config <bones.toml>`
- `bonesinfra ssl apply --config <bones.toml>`
- `bonesinfra helpers apply --config <bones.toml>`
- `bonesinfra services apply --config <bones.toml>`
- `bonesinfra manifest show --config <bones.toml>`

Framework template questions are defined by the Rust CLI under
`crates/bonesdeploy/src/frameworks/`. BonesInfra reads the resulting
`bones.toml` and applies infrastructure; it does not prompt for those
settings.

## Project infrastructure

`runtime apply` loads `.bones/infra/runtime.py` from the project (resolved
relative to `bones.toml`), imports it as a package alongside
`infra/__init__.py` and `infra/manifest.py`, and calls its `deploy(ctx)`.
The manifest declares framework-owned artifacts, services, and mode for
`manifest show`. `.bones/infra/` and its local `templates/` are generated
by `bonesdeploy init` from the kit and named-framework snapshots embedded
in the Rust binary.

## Notes

- `bonesinfra` reads one `bones.toml` file.
- It does not create those files.
- It does not own deployment scripts.