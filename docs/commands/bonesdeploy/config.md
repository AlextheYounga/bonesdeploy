# bonesdeploy config

## Overview

Reads a scalar value from a TOML config file. By default this is used with `.bones/bones.toml`.

## Command Signature

```bash
bonesdeploy config [--file <path>] <key>
```

## Behavior

- Reads the TOML file from `--file <path>`.
- Looks up the top-level `<key>`.
- Prints string, boolean, integer, and float values without extra formatting.
- Fails when the key is missing or when the value is not a supported scalar.

## Examples

```bash
bonesdeploy config --file .bones/bones.toml project_name
bonesdeploy config --file .bones/bones.toml deploy_on_push
```

## Related Commands

- `bonesdeploy init` - Writes `.bones/bones.toml`.
- `bonesremote config` - Reads values from a server-side config file.
