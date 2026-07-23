# bonesinfra

Hidden Python provisioning engine embedded in the BonesDeploy monorepo.

It handles pyinfra-based setup, runtime, and SSL provisioning. BonesDeploy owns TOML creation and uses this package as the private execution layer.

The Rust `bonesinfra` crate embeds this package into the `bonesdeploy` binary.
For Python-only development, work from `crates/bonesinfra/python`; production
execution uses the embedded copy.

## Interface

- `bonesinfra runtime list`
- `bonesinfra setup apply --config <bones.toml>`
- `bonesinfra runtime apply --config <bones.toml>`
- `bonesinfra ssl apply --config <bones.toml>`

Runtime questions are defined by the Rust CLI under
`crates/bonesdeploy/src/runtimes/`. BonesInfra reads the resulting
`bones.toml` and applies infrastructure; it does not prompt for runtime
settings.

## Notes

- `bonesinfra` reads one `bones.toml` file.
- It does not create those files.
- It does not own deployment scripts.
