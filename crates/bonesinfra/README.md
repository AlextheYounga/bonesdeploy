# bonesinfra

Hidden Python provisioning engine for BonesDeploy.

It handles pyinfra-based setup, runtime, and SSL provisioning. BonesDeploy owns TOML creation and uses this package as the private execution layer.

## Interface

- `bonesinfra runtime list`
- `bonesinfra runtime questions <runtime>`
- `bonesinfra setup apply --config <bones.toml>`
- `bonesinfra runtime apply --config <bones.toml>`
- `bonesinfra ssl apply --config <bones.toml>`

## Runtime metadata

Runtime modules expose `questions()` so BonesDeploy can prompt users and write the `[runtime]` section in `bones.toml`.

## Notes

- `bonesinfra` reads one `bones.toml` file.
- It does not create those files.
- It does not own deployment scripts.
