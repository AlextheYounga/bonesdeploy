# bonesremote release wire

## Overview

Creates a symlink from the build workspace to the shared directory for `.env`. Called internally by `bonesremote deploy --config <path>` (the recommended unified command) after git checkout.

Use this subcommand directly when composing a custom pipeline from individual building blocks.

## Command Signature

```bash
bonesremote release wire --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.toml` configuration file (required)

---

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `wire_release.rs`

Loads deployment configuration to get:
- Staged release name
- Directory locations

---

### 2. Define Paths

**Paths:**
- `shared_env`: `<project_root>/shared/.env`
- `workspace_env`: `<build_root>/.env`
- `workspace_env_example`: `<build_root>/.env.example`

---

### 3. Link .env

**Algorithm:**

1. If `shared/.env` exists:
   - Remove `build/workspace/.env` if it exists (build workspace is disposable)
   - Symlink `build/workspace/.env` -> `shared/.env`
   - Print: `Linked .env: build/workspace/.env -> shared/.env`

2. If `shared/.env` does not exist but `build/workspace/.env.example` exists:
   - Create `shared/` directory if needed
   - Copy `build/workspace/.env.example` to `shared/.env`
   - Set conservative permissions (600) on `shared/.env`
   - Remove `build/workspace/.env` if it exists
   - Symlink `build/workspace/.env` -> `shared/.env`
   - Print: `Created shared .env from .env.example and linked it:`

3. If neither `shared/.env` nor `build/workspace/.env.example` exist:
   - Do not fail
   - Print: `.env not found and .env.example not found; skipping .env link. Use bonesdeploy secrets to provide .env.`

**Rules:**
- Never overwrite existing `shared/.env`.
- Never truncate existing `shared/.env`.
- It is safe to remove `build/workspace/.env` because `build/workspace` is disposable.
- Storage, cache, uploads, and database paths are NOT symlinked.
- Storage, cache, uploads, and database paths are NOT created.

---

## Directory Structure After Wiring

```
/srv/sites/myapp/
├── build/
│   └── workspace/
│       ├── .env -> ../../shared/.env           # Symlink
│       └── (other files from git checkout)
├── releases/
│   └── 20260507_150432/    # (empty, waiting for build)
├── shared/
│   └── .env                # Actual file, runtime-user-owned
└── current -> releases/20260507_140000/
```

---

## Runtime State Paths

BonesDeploy no longer manages arbitrary shared symlinks. Only `.env` is linked into releases.

Runtime state paths belong in `.env`. Frameworks create their own storage/cache/uploads/database files under `project_root/shared`.

For Laravel SQLite, configure via `.env`:

```env
DB_CONNECTION=sqlite
DB_DATABASE=/srv/sites/<project>/shared/database.sqlite

STORAGE_PATH=/srv/sites/<project>/shared/storage
CACHE_PATH=/srv/sites/<project>/shared/cache
UPLOADS_PATH=/srv/sites/<project>/shared/uploads
```

Do not symlink database or storage directories.

---

## Typical Workflow

```bash
# 1. Stage release
bonesremote release stage --config /home/git/myapp.git/bones/bones.toml

# 2. Check out code (done by post-receive hook)
git --work-tree=/srv/sites/myapp/build/workspace \
    --git-dir=/home/git/myapp.git \
    checkout -f master

# 3. Wire .env
bonesremote release wire --config /home/git/myapp.git/bones/bones.toml

# 4. Build and deploy
# (deployment scripts run in build/workspace with symlink active)

# 5. Activate release
bonesremote release activate --config /home/git/myapp.git/bones/bones.toml
```

---

## Secrets Workflow

1. `bonesdeploy init` copies runtime secrets examples to `.bones/secrets/`
2. User edits/copies `.bones/secrets/.env.prod.example`
3. User creates/encrypts real env through `bonesdeploy secrets`
4. `bonesdeploy secrets push` writes remote `project_root/shared/.env`
5. Deploy links `build/workspace/.env` to `project_root/shared/.env`

---

## Related Commands

- `bonesremote release stage` - Stage a new release
- `bonesremote release activate` - Activate the release
- `bonesremote hooks post-receive` - Orchestrates staging and wiring
- `bonesremote hooks deploy` - Runs deployment scripts
