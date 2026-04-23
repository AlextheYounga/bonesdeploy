# BonesDeploy TODO

## Phase 1: Workspace & Scaffolding
- [x] Convert to Cargo workspace with `crates/bonesdeploy` and `crates/bonesremote`
- [x] Set up clap CLI skeleton for both binaries (subcommands, help text)
- [x] Wire up `version` command for both binaries

## Phase 2: Config & Embedded Assets
- [x] Define `bones.toml` serde structs in `bonesdeploy/src/config.rs`
- [x] Implement load/save for local config (`.bones/bones.toml`)
- [x] Set up `rust-embed` pointing at `kit/` in `bonesdeploy/src/embedded.rs`
- [x] Write scaffold extraction: create `.bones/` directory tree from embedded assets
- [x] Write the kit hook scripts (pre-push, pre-receive, pre-deploy, post-receive, deploy, post-deploy)

## Phase 3: bonesdeploy init
- [x] Implement `prompts.rs` using inquire (collect all bones.toml fields with defaults)
- [x] Implement `git.rs` (read remote URL from git2, validate repo state)
- [x] Implement init command orchestration:
  - [x] Inform user that a remote git URL must already be set, explain what will happen, confirm with user
  - [x] Fail if no git remote URL is set for the configured remote name
  - [x] Extract scaffold to `.bones/`
  - [x] Update `.gitignore` to include `.bones`
  - [x] Load existing config or run prompts for new config
  - [x] Save config to `.bones/bones.toml`
- [x] Symlink `.bones/hooks/pre-push` to `.git/hooks/pre-push`

## Phase 4: SSH & Remote Setup (bonesdeploy init, continued)
- [x] Implement `ssh.rs` (openssh session from host/port/deploy_user in config)
- [x] Create bare repo on remote if it doesn't exist
- [x] Upload post-receive hook to remote bare repo

## Phase 5: bonesdeploy push
- [x] Implement rsync of `.bones/` to `{git_dir}/bones/` on remote
- [x] Delete sample hooks from remote bare repo `{git_dir}/hooks/`
- [x] Symlink `{git_dir}/bones/hooks/*` to `{git_dir}/hooks/` on remote

## Phase 6: bonesdeploy doctor
- [x] Local checks:
  - [x] `.bones/` folder structure is valid
  - [x] Deployment scripts follow naming convention
  - [x] `pre-push` hook is symlinked to `.git/hooks/pre-push`
- [x] Remote checks (over SSH, skipped with `--local`):
  - [x] `bonesremote` is globally available on remote
  - [x] `{git_dir}/bones/` exists on remote
  - [x] `{git_dir}/bones/hooks/` entries match `{git_dir}/hooks/` symlinks
- [x] Implement `--local` flag (pre-push hook uses this since remote is validated independently by bonesremote doctor)

## Phase 7: bonesremote init
- [x] Define `bones.toml` serde structs in `bonesremote/src/config.rs`
- [x] Check that command is run as root/sudo
- [x] Write `/etc/sudoers.d/bonesdeploy` drop-in file
- [x] Validate with `visudo -c`

## Phase 8: bonesremote doctor
- [x] Check `bonesremote` can run without password (sudo -n)
- [x] Check `bonesremote` is globally available (which/command -v)

## Phase 9: bonesremote pre-deploy & post-deploy
- [x] Implement `config.rs` for remote (discover `bones.toml` relative to bare repo)
- [x] `pre-deploy`: chown worktree to deploy user
- [x] `post-deploy`: implement `permissions.rs`
  - [x] Apply default ownership (service_user:service_group)
  - [x] Apply default dir_mode and file_mode
  - [x] Apply path overrides (recursive, type=dir, type=file)

## Phase 10: End-to-end testing
- [ ] Full flow: init -> push -> git push -> deploy cycle
- [ ] Verify permissions are correct after post-deploy
- [ ] Test with non-default port and host
- [ ] Test re-running init on an existing project (idempotency)
- [ ] Test doctor catches misconfigurations
