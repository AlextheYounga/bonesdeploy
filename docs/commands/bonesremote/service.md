# bonesremote service

## Overview

Manages server-side services for a deployed site.

## Command Signature

```bash
bonesremote service restart --config <path>
```

## `service restart`

- Requires root privileges.
- Loads the project config from `--config <path>`.
- Validates `project_name` as a safe systemd service name component.
- Restarts `<project_name>-nginx` with `systemctl restart`.

The intended privilege path is the narrow sudoers drop-in installed by `bonesremote init`, allowing only:

```bash
sudo bonesremote service restart --config *
```

## Related Commands

- `bonesremote init` - Installs the sudoers drop-in.
- `bonesremote deploy` - Calls `service restart` after activating a release.
- `bonesdeploy remote runtime` - Creates the per-site nginx systemd service.
