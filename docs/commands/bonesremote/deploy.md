# bonesremote deploy

## Overview

Runs the full server-side deployment lifecycle. This is the primary remote entrypoint used by both `bonesdeploy deploy` and the `post-receive` hook.

## Command Signature

```bash
bonesremote deploy --config <path> [--revision <rev>]
```

## Behavior

`bonesremote deploy` orchestrates:

1. `doctor` — checks the server environment.
2. `release stage` — creates staged release state and the release directory.
3. `post_receive` — checks out the configured branch or exact `--revision` into `build/workspace`.
4. `release wire` — symlinks shared runtime paths into the workspace.
5. Deploy scripts — runs scripts from `<repo_path>/bones/deployment/` in sorted order.
6. `release activate` — publishes the staged release by switching `current`.
7. `service restart` — runs `sudo bonesremote service restart --config <path>`.
8. `post_deploy` — prunes old releases beyond `releases`.

If a failure occurs after staging, `release drop-failed` clears staged release state and removes the failed release directory.

## Flags

- `--config <path>`: Path to the remote `bones.toml`.
- `--revision <rev>`: Exact commit to deploy. When omitted, the configured branch is used.

## Related Commands

- `bonesdeploy deploy` - SSHes to the host and runs this command.
- `bonesremote release stage` - Individual lifecycle step.
- `bonesremote service restart` - Restarts the per-site nginx service.
