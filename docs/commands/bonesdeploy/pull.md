# bonesdeploy pull

## Overview

Synchronizes the remote bare repository's `bones/` directory back into the local `.bones/` directory using rsync, then restores the local pre-push hook symlink. This is the reverse of `bonesdeploy push` and lets you recover the deployment scaffold on another machine with SSH access to the deployment server.

## Detailed Execution Steps

### 1. Determine Sync Target

The command uses the local `.bones/bones.yaml` when it exists. Otherwise it falls back to the configured Git remote URL, preferring a remote named `production` when multiple remotes are present.

### 2. Create Local Scaffold Directory

Ensures `.bones/` exists before syncing.

### 3. Sync Remote `bones/` Back Locally

Uses `rsync -av --delete` over SSH to copy:

```bash
<user>@<host>:<repo_path>/bones/
```

into local `.bones/`.

### 4. Restore Local Hook Symlink

Recreates `.git/hooks/pre-push` as a symlink to `../../.bones/hooks/pre-push`.

### 5. Print Success Message

Reports `.bones/ pulled from remote.` when complete.

## Related Commands

- `bonesdeploy push` - Syncs local `.bones/` to the remote bare repo
- `bonesdeploy init` - Creates the initial local scaffold
