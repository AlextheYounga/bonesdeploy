# bonesdeploy secrets

## Overview

Manages encrypted project environment secrets under `.bones/secrets/` and uploads the decrypted `.env` to the remote shared directory when requested.

## Command Signature

```bash
bonesdeploy secrets init
bonesdeploy secrets edit
bonesdeploy secrets push
```

## Subcommands

### `secrets init`

- Requires `.bones/` to already exist; run `bonesdeploy init` first.
- Requires `gpg`.
- Creates `.bones/secrets/`.
- Creates or reuses a project-specific GPG key in the BonesDeploy GPG home.
- Fails if the removed legacy `.bones/secrets.toml` file exists.

### `secrets edit`

- Requires `gpg` and `$EDITOR`.
- Decrypts `.bones/secrets/.env.gpg` to a temporary file when it already exists.
- Opens the temporary plaintext file in `$EDITOR`.
- Re-encrypts it to `.bones/secrets/.env.gpg` for the project key.
- Removes the temporary plaintext file after editing.

### `secrets push`

- Requires `.bones/secrets/.env.gpg`.
- Loads `.bones/bones.toml` and `.bones/runtime.toml`.
- Connects to the configured host as `ssh_user` or `BONES_BOOTSTRAP_SSH_USER`.
- Decrypts the local secret and writes it to `<project_root>/shared/.env`.
- Sets ownership to `root:<runtime_group>` and mode `0640`.

## Related Commands

- `bonesdeploy init` - Initialize `.bones/` before secrets setup.
- `bonesdeploy remote runtime` - Defines runtime identity values used by `secrets push`.
- `bonesdeploy deploy` - Deploys code that can consume the remote shared `.env`.
