# bonesremote config

## Overview

Reads a scalar value from a TOML config file on the remote server.

## Command Signature

```bash
bonesremote config --file <path> <key>
```

## Behavior

- Reads the TOML file from `--file <path>`.
- Looks up the top-level `<key>`.
- Prints string, boolean, integer, and float values without extra formatting.
- Fails when the key is missing or when the value is not a supported scalar.

## Examples

```bash
bonesremote config --file /home/git/myapp.git/bones/bones.toml project_name
bonesremote config --file /home/git/myapp.git/bones/bones.toml project_root
```

## Related Commands

- `bonesdeploy config` - Local equivalent.
- `bonesremote deploy` - Consumes the same remote config file.
