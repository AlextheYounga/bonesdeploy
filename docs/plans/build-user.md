### 1. Add Build Identity Helpers
In `crates/shared/src/config.rs`:

```rust
pub fn build_user_for(project_name: &str) -> String {
    format!("{project_name}-build")
}

pub fn build_group_for(project_name: &str) -> String {
    format!("{project_name}-build")
}
```

Do **not** add `build_user` to `runtime.toml`. Keep it derived and boring.

### 2. Move Build Context Out Of `/root`
Current path:

```text
/root/.config/bonesremote/sites/<site>/tmp/build-...
```

New path:

```text
/srv/sites/<site>/tmp/build-...
```

Rationale:
- Runtime/build users cannot traverse `/root`.
- `/srv/sites/<site>` is already part of the deployment model.
- Avoids `/tmp` `noexec` risk.
- Keeps disposable deploy scratch near the project, but outside `releases/` and `shared/`.

Implementation:
- Change `release_checkout::ensure_build_context` to create under `cfg.project_root/tmp`.
- Create the build context as root.
- After checkout, before Podman, recursively `chown` it to `<project>-build:<project>-build`.
- Cleanup removes the unique `build-...` dir after deploy.
- Leaving an empty `/srv/sites/<site>/tmp` is acceptable.

### 3. Keep Build Scripts Root-Owned
No change to build script location:

```text
/root/.config/bonesremote/sites/<site>/deployment/build/
```

Root opens each script and pipes it into Podman stdin.

Important security property:
- `<project>-build` cannot read build scripts from disk.
- `<project>-build` only receives the selected script content over stdin.
- Runtime user still cannot access build scripts.

### 4. Run Podman As Build User
In `crates/bonesremote/src/release/script_runner/mod.rs`, replace direct `podman` execution with:

```text
runuser -u <project>-build -- env \
  HOME=/var/lib/bonesdeploy/users/<project>-build \
  XDG_RUNTIME_DIR=/run/user/<build_uid> \
  podman run ...
```

Preserve existing Podman arguments:
- `--rm`
- `-i`
- `--pull=missing`
- `--security-opt=no-new-privileges`
- `--cap-drop=all`
- `--workdir=/workspace/source`
- `--volume <context>:/workspace/source`
- build env vars
- build image
- `bash -c "umask 0002; exec bash -s"`

### 5. Resolve Build UID
Add a helper in `release_build.rs`:

```rust
fn user_uid(user: &str) -> Result<u32>
```

Use it for:
- `<project>-build` UID, needed for `XDG_RUNTIME_DIR=/run/user/<uid>`
- existing `root_uid()` logic, if convenient

### 6. Keep Promotion Root-Owned
No change to promotion model.

Flow remains:

```text
build user writes build context
root copies context into release dir
root hardens release tree
runtime user reads release tree
```

That avoids direct build-user/runtime-user permission sharing.

### 7. Tests
Update/add Rust tests for:
- `build_user_for("demo") == "demo-build"`
- Podman command starts with `runuser -u demo-build -- env`
- Command sets `HOME=/var/lib/bonesdeploy/users/demo-build`
- Command sets `XDG_RUNTIME_DIR=/run/user/<uid>`
- Command still includes `podman run`, `-i`, security opts, source mount
- Command does not reference `/root/.config/bonesremote`
- UID parsing works

---

## bonesinfra

### 1. Install Rootless Podman Prereqs
`BASE_SYSTEM_PACKAGES` already includes:
- `podman`
- `passt`
- `slirp4netns`
- `aardvark-dns`
- `netavark`

Add if missing:
- `uidmap`
- `fuse-overlayfs`
- `dbus-user-session`

### 2. Create Build User
In setup user provisioning, create:

```text
<project>-build
```

With:
- system user
- `home=/nonexistent`
- `shell=/usr/sbin/nologin`
- `create_home=False`
- no supplementary runtime groups

The build user should **not** be in:
- runtime group
- nginx group
- shared-path groups

### 3. Create Podman Pseudo-Home
Create:

```text
/var/lib/bonesdeploy/users/<project>-build
```

Owned by:

```text
<project>-build:<project>-build
```

Mode:

```text
0700
```

Optionally pre-create:
- `.config`
- `.local/share`
- `.cache`

But Podman can create those itself if the parent is writable.

### 4. Enable Linger
Run:

```text
loginctl enable-linger <project>-build
```

This makes `/run/user/<build_uid>` exist without an interactive login.

### 5. Allocate Unique subuid/subgid Ranges
Do **not** reuse the same range for all build users.

Each build user needs unique ranges in:

```text
/etc/subuid
/etc/subgid
```

Example:

```text
site-a-build:100000:65536
site-a-build:100000:65536

site-b-build:165536:65536
site-b-build:165536:65536
```

Correct behavior:
- If `<project>-build` already has entries, leave them unchanged.
- If missing, allocate the next free `65536` block.
- Ensure `/etc/subuid` and `/etc/subgid` stay aligned for the same build user.
- Avoid overlapping ranges.

Implementation options:
- Prefer distro tooling if it safely allocates unique ranges idempotently.
- Otherwise add a small helper that reads both files, finds the max used range, and appends the next block.

### 6. Validate Rootless Podman
Add a validation command run as `<project>-build`:

```text
HOME=/var/lib/bonesdeploy/users/<project>-build \
XDG_RUNTIME_DIR=/run/user/<build_uid> \
podman info --format '{{.Host.Security.Rootless}}'
```

Expected output:

```text
true
```

This should run during provisioning so deploy failures happen early.

### 7. Tests
Add/update bonesinfra tests for:
- `uidmap`, `fuse-overlayfs`, `dbus-user-session` packages
- build user creation with `/nonexistent` home and nologin shell
- no runtime group membership for build user
- pseudo-home creation under `/var/lib/bonesdeploy/users/<project>-build`
- linger command
- unique subuid/subgid allocation behavior
- rootless Podman validation command

---

## Final Deploy Flow

```text
root
  creates /srv/sites/<site>/tmp/build-...
  exports git revision into it
  chowns build context to <project>-build:<project>-build
  opens root-owned build script
  runs rootless podman as <project>-build, piping script over stdin

<project>-build
  runs podman rootless
  writes build artifacts into mounted build context
  cannot read /root/.config/bonesremote
  cannot read shared/
  cannot read releases/

root
  promotes build context into release dir
  hardens ownership and permissions
  cleans build context

runtime user
  runs app from hardened release
  accesses only release/shared paths
```

**Recommendation**
Use the per-project build user with unique subuid/subgid ranges. This is the cleanest way to satisfy the security goals without giving the runtime user a writable home or build-script access.
