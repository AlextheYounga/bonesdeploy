# Coward's Code Audit

Defensive branches that attempt to "recover" from edge cases that are
extremely unlikely or impossible given the actual control flow. These
usually hide bugs rather than prevent them.

## High Confidence — Delete or Crash Deliberately

### 1. `release_prepare.rs:43–50` · `release_build.rs:49–55, 91–97`

```rust
load_runtime(&paths::bonesremote_site_root(site)).unwrap_or_else(|_| Runtime {
    web_root: default_web_root(),
    build_image: String::new(),
    ..
});
```

`load_runtime` already returns a default `Runtime` when the file is
missing ([config.rs:166-173](../shared/src/config.rs)). If it *fails*
(parse error, read error), silently replacing the error with defaults
hides the broken config. A missing runtime.toml is handled; a corrupt
one should fail the deploy.

**Fix:** let the error propagate (`load_runtime(...)?`).

---

### 2. `release_state.rs:91–100`

```rust
current_link.parent().unwrap_or_else(|| Path::new("/"))
```

`current_link` is `{project_root}/current` — a two-component path that
always has a parent. The only way `.parent()` returns `None` is a root
path, which project_root can never be. This is dead code that would
produce pathologically wrong results if it ever ran.

**Fix:** use `.parent().context(...)?`.

---

### 3. `remote_data.rs:63–66, 94–99`

```rust
Path::new(&cfg.repo_path).parent().unwrap_or(Path::new(paths::DEFAULT_REPO_PARENT))
Path::new(project_root).parent().unwrap_or(Path::new(paths::DEFAULT_PROJECT_ROOT_PARENT))
```

A repo_path or project_root without a parent means it's a root path
(e.g. `/srv`), which is a misconfigured project. Silently falling back
to the conventional parent hides the misconfiguration.

**Fix:** let it crash — `.parent().context(...)?`.

---

### 4. `doctor_site.rs:119–123`

```rust
match current_path.parent() {
    Some(parent) if parent.is_dir() => {}
    Some(parent) => issues.push(...),
    None => issues.push(format!("current path has no parent: {current_path:?}")),
}
```

`current_path` is `{project_root}/current` — always has a parent.
The `None` arm is unreachable dead code.

**Fix:** drop the `None` arm.

---

### 5. `config.rs:28–30` (bonesdeploy)

```rust
Ok(cwd.file_name().map_or_else(|| String::from("project"), |n| n.to_string_lossy().to_string()))
```

Only triggers if `bonesdeploy init` is run from a filesystem root or on
a platform where `current_dir()` has no filename — neither of which is
a realistic use case. The `"project"` fallback provides a wrong name
instead of failing fast.

**Fix:** `cwd.file_name().context("current directory has no name")?.to_string_lossy().to_string()`

---

### 6. `shared/src/config.rs:21, 44–63` — `#[serde(default)]` on entire `Bones` struct

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]    // <-- applies to the whole struct
pub struct Bones { ... }
```

Any missing field in `bones.toml` silently gets `Default::default()` rather
than failing to parse. Fields like `remote_name`, `project_name`, and `host`
are required for the tool to function. If a user typos a field name or
forgets one, there is zero indication — the tool proceeds with empty strings
and fails cryptically later.

**Fix:** Remove `#[serde(default)]` from struct level. Mark individual
optional fields with `#[serde(default)]` only where the default is genuinely
safe. The `Default` impl should be removed (its only non-test use is
`..Default::default()` spread in builder calls that already set every field).

---

### 7. `shared/src/config.rs:65` — Empty `impl Bones {}` block

```rust
impl Bones {}
```

Dead code. Serves no purpose.

**Fix:** Delete it.

---

### 8. `shared/src/config.rs:138–174` — `#[serde(default)]` on all `Runtime` fields

```rust
pub struct Runtime {
    #[serde(default)]
    pub build_image: String,       // default "" for a field that
    #[serde(default)]              // most runtimes require
    pub runtime_user: String,
    #[serde(default)]
    pub runtime_group: String,
    #[serde(default)]
    pub release_group: String,
}
```

Combined with `load_runtime` returning a full-default `Runtime` when the
file doesn't exist, a missing or mis-typed `runtime.toml` is silently
accepted with zero configuration. If a runtime requires `build_image`
(most do), the first indicator is a cryptic error deep in `release_build`.

**Fix:** Remove `#[serde(default)]` from `build_image`. In `load_runtime`,
error or at minimum require explicit configuration when the file is absent.

---

### 9. `deploy.rs:32–75 (×7)` — `.ok()` on `drop_failed_release::run()` in error recovery

```rust
if let Err(error) = release_checkout::run(site, ...) {
    cleanup(site, Some(&context_dir));
    drop_failed_release::run(site).ok();   // <-- error swallowed
    return Err(error);
}
```

Repeated 7 times in `run_full`. If `drop_failed_release` itself fails,
the error is silently discarded. The deployment system may be left in a
state where a failed release was not properly cleaned up, and the user
never learns that cleanup also broke.

**Fix:** At minimum, `eprintln!` the cleanup error. Better: collect both.

---

### 10. `deploy.rs:81–85` — `.ok()` on `cleanup_build_context`

```rust
fn cleanup(site: &str, context: Option<&Path>) {
    if let Some(context) = context {
        release_checkout::cleanup_build_context(site, context).ok();
    }
}
```

Same issue — build-context cleanup error is invisible.

**Fix:** Log to stderr on failure.

---

### 11. `site.rs:134–138` — `.ok()` on backup-restore after failed activation

```rust
if let Err(error) = fs::rename(staging_dir, &site_root) {
    if had_existing {
        fs::rename(&backup_dir, &site_root).ok();  // <-- error swallowed
    }
    return Err(error)...;
}
```

If the main `rename` fails and the restore `rename` *also* fails, the
site could be left broken (staging_dir orphaned, backup gone, site_root
missing). The user has no indication the restore attempt failed.

**Fix:** Print a warning to stderr if the restore rename fails.

---

### 12. `drop_failed_release.rs:19–20` — `.ok()` on `clear_staged_release`

```rust
let Ok(release_name) = release_state::read_staged_release(site) else {
    release_state::clear_staged_release(site).ok();  // <-- error swallowed
    ...
};
```

If the staged release file is unreadable and the subsequent clear also
fails (e.g. permission error), we proceed as if cleanup succeeded.

**Fix:** `eprintln!` the error.

---

### 13. `pull_state.rs:61` — Parsed git user overwritten with hardcoded `"root"`

```rust
let details = git::infer_remote_connection_details(&remote_name)?...;

Ok(git::RemoteConnectionDetails { user: String::from("root"), ..details })
```

When falling back to inferring connection details from the git remote URL,
the parsed user (e.g. `"git"` from `ssh://git@example.com/repo.git`) is
unconditionally overwritten with `"root"`. The inferred data is thrown away.

**Fix:** Use the inferred user, or at minimum pass it through
`bootstrap_ssh::resolve`.

---

### 14. `ui/prompts.rs:26` — `unwrap_or_default()` on runtime questions contract

```rust
let questions = questions.as_array().cloned().unwrap_or_default();
```

If `bonesinfra` returns a malformed questions value (string, object instead
of array), this silently converts it to zero questions. The user is told
configuration succeeded but answered nothing.

**Fix:** `bail!` or propagate an error. A broken data contract from
bonesinfra is a programmer error.

---

### 15. `ui/prompts.rs:29, 33–34` — `unwrap_or("")` on question metadata

```rust
let key = question["key"].as_str().unwrap_or("");
if key.is_empty() { continue; }            // silently skip
let question_type = question["type"].as_str().unwrap_or("text");  // silently default
```

Questions with missing keys are silently skipped. A wrong question type
silently becomes a text input. Mask contract violations from bonesinfra.

**Fix:** `bail!` when metadata is missing or invalid.

---

### 16. `ui/prompts.rs:49` — `unwrap_or(Value::Null)` for empty choice questions

```rust
if choices.is_empty() {
    default.unwrap_or(serde_json::Value::Null)
}
```

A runtime choice question with zero valid choices silently sets the answer
to `Null`. The configuration is accepted without complaint.

**Fix:** `bail!` with a descriptive error.

---

### 17. `doctor.rs:13` — `.ok()` on config load in the diagnostic tool

```rust
let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML)).ok();
```

A corrupted `bones.toml` silently becomes `None`, so all remote checks are
skipped without the user learning that their config is broken. Doctor's
purpose is to diagnose — this hides the diagnosis.

**Fix:** Report the parse error as a doctor issue instead of skipping it.

---

### 18. `doctor.rs:110, 135` — `.ok()?` silently skipping checks on I/O error

```rust
let entries = fs::read_dir(&scripts_dir).ok()?;
let target = fs::read_link(link).ok()?;
```

If `read_dir` or `read_link` fails due to permissions or filesystem error,
the entire check silently returns `None`. Doctor says "all good" without
checking.

**Fix:** Push the error into the issues list instead of skipping it.

---

### 19. `doctor.rs:147, 180` — `filter_map(Result::ok)` silencing per-entry errors

```rust
let profile_files: Vec<String> = profiles
    .filter_map(Result::ok)                                     // line 147
    .filter_map(|entry| entry.file_name().into_string().ok())   // line 148
    ...
let unit_entries: HashMap<String, PathBuf> = units
    .filter_map(Result::ok)                                     // line 180
    ...
```

If `read_dir` yields an I/O error for an individual entry (permission
denied, corrupt filesystem), or a filename is non-UTF-8 in AppArmor dirs,
the entry is silently skipped. A diagnostic tool should report these, not
hide them.

**Fix:** Collect errors into the issues list instead of discarding them.

---

### 20. `doctor.rs:147, 161` — `let _ = session.close().await` swallowing SSH close errors

```rust
let _ = session.close().await;
```

If the remote SSH session fails to flush or close cleanly, doctor reports
"all checks passed" anyway.

**Fix:** Report close errors as issues.

---

### 21. `site.rs:168` — `map_or(0_u128, ...)` guarding a pre-1970 clock

```rust
let stamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0_u128, |duration| duration.as_nanos());
```

`duration_since(UNIX_EPOCH)` returns `Err` only when the system clock is
set before Jan 1, 1970. The `0` fallback produces a collision-prone path
component that is *worse* than crashing. Same pattern in
`release_checkout.rs:84`, `post_deploy.rs:69`, `scripts.rs:217`,
and test helpers.

**Fix:** `.expect("system clock is before UNIX epoch")`. This is a trust
boundary where the assertion is justified.

Affected files:
- `crates/bonesremote/src/commands/site.rs:168`
- `crates/bonesremote/src/commands/release_checkout.rs:84`
- `crates/bonesremote/src/commands/post_deploy.rs:69`
- `crates/bonesremote/src/release/scripts.rs:217`
- `crates/bonesdeploy/src/commands/secrets.rs:264`
- `crates/bonesremote/src/release_state.rs:176` (test helper)
- `crates/bonesdeploy/src/config.rs:65` (test helper)

---

### 22. `update.rs:132–133` — `.ok()?` silently downgrading on corrupt runtime config

```rust
fn selected_runtime_template(runtime_toml: &Path) -> Option<String> {
    let content = fs::read_to_string(runtime_toml).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;
    value.get("template")?.as_str().map(String::from)
}
```

During `bonesdeploy update`, a corrupted `runtime.toml` silently causes
fallback to the generic kit deployment scripts instead of the
runtime-specific ones. No warning, no error.

**Fix:** Propagate the error instead of silently returning `None`.

---

## Borderline — Acceptable but Softening

### 23. `shared/src/config.rs:83–94` — `validate_host` silently accepts empty string

```rust
pub fn validate_host(host: &str) -> Result<()> {
    let host = host.trim();
    if host.is_empty() {
        return Ok(());    // empty host accepted as valid
    }
    ...
}
```

An empty host is invalid for any real use case. If there is a legitimate
reason to accept empty hosts, the check should be at the call site, not in
the validator.

---

### 24. `status.rs:44–47` (bonesdeploy)

```rust
remote_status(cfg).await.unwrap_or_else(|_| fallback_remote_status(cfg))
```

Converts an SSH failure into a locally-guessed status. Reasonable for
the *status* command (best-effort), but it means the output lies
silently. Consider printing a warning.

---

### 25. `status.rs:52–56, 115–120` (bonesremote)
### 26. `update_release.rs:16–32`

`"unknown"` fallbacks for status probes and version checks.

Fine for diagnostics, but the string is indistinguishable from a real
"unknown" state. If these are ever consumed programmatically, the error
should be surfaced as an absent field instead.

---

## Justified — Keep

- **Host / build-image validation** (`shared/src/config.rs`)
- **Symlink escape checks** (`release_build.rs`)
- **SSH / systemd / passwd / group probes** (doctor commands)
- **Trust-boundary input validation** (`validate_host`, `validate_site_name`)
- **Missing file handling in init** (`init_project.rs`) — these handle
  genuine first-run states

These guard actual trust boundaries or real OS conditions, not
imaginary edge cases.
