I checked the current Rust structure across the workspace, focusing on the crate boundaries and representative command/orchestration files. The workspace is now four members: `shared`, `bonesdeploy`, `bonesremote`, and `tests/cleancode`.

Your instinct is right: the code is not “bad,” but the **boundaries are mushy**. The main issue is that `commands/*` files are doing too much. They are not just commands; they are CLI routing, app orchestration, filesystem work, subprocess execution, SSH, git, config loading, prompting, and domain decisions all in one place.

## What feels disorganized right now

`bonesdeploy/src/main.rs` imports everything as flat top-level modules: `bonesinfra`, `bootstrap_ssh`, `commands`, `config`, `embedded`, `git`, `prompts`, `python`, `remote_data`, and `ssh`.  That is a smell: there is no obvious distinction between domain logic, infrastructure adapters, UI, and CLI commands.

`bonesdeploy/src/commands/mod.rs` also mixes three concerns: command definition, CLI help text, and command dispatch. It defines all subcommands directly and then calls into the implementation modules from one big match.   That is workable early, but it becomes hard to follow as the product grows.

`init.rs` is the biggest example of boundary collapse. In one flow it ensures a git repo, creates/symlinks `.bones`, scaffolds embedded assets, writes config, collects prompts, asks Python for runtime info, configures local git remotes, symlinks hooks, and may run remote setup.  It also contains config collection helpers, runtime config seeding, `.gitignore` mutation, hook symlinking, and remote creation logic.    That should be split.

`push.rs` is another example. It loads config, builds remote paths, rsyncs `.bones`, opens SSH, deletes sample hooks, symlinks hooks remotely, and prints user output.  The `sync_bones_directory` helper directly shells out to `rsync`.  That should be an adapter, not command logic.

`bonesremote` has the same pattern. It has a flat command list, including lifecycle operations, hook helpers, service operations, and top-level deploy all in the same command enum.  `deploy.rs` mixes the new high-level lifecycle, deployment script execution, service restart, release publish, and cleanup.   That file is now carrying too much conceptual weight.

The `shared` crate is useful, but it is currently a bit of a junk drawer. `shared::paths` contains local `.bones` paths, remote repo paths, nginx paths, systemd paths, AppArmor paths, install paths, binary names, socket names, and release names all in one file.  Then it also defines `DeploymentPaths`, which combines repo, config, release, nginx, systemd, AppArmor, socket, sudoers, and binary paths.  That makes every part of the product feel coupled to every other part.

You already have some clean-code guardrails, like the `tests/cleancode` crate and the 400-line file test.   That is good, but line count alone does not enforce architectural boundaries.

# Proposed organization model

Do not jump straight to a huge “enterprise architecture.” The goal should be **boring, obvious layers**.

## Layer 1: CLI

CLI modules should only do:

```text
parse args
map args to app calls
format command-level errors/help
```

They should not shell out, load files directly, mutate git remotes, open SSH sessions, or know how deployment works.

## Layer 2: App services

App services should orchestrate use cases:

```text
init project
deploy project
push project state
apply remote setup
apply runtime
apply SSL
update installation
rollback
```

These services may call domain logic and infrastructure adapters, but they should be the only place where a user action becomes a sequence of steps.

## Layer 3: Domain

Domain code should describe stable concepts:

```text
ProjectConfig
RemoteTarget
DeployRevision
ReleaseName
ReleaseState
DeploymentPaths
RuntimeSelection
BonesProject
```

This code should avoid subprocesses, SSH, file syncing, prompts, and printing. Ideally it is easy to unit test.

## Layer 4: Infrastructure adapters

Infrastructure modules own side effects:

```text
git command adapter
ssh adapter
rsync adapter
filesystem adapter
process runner
bonesinfra runner
cargo installer
remote command runner
embedded asset writer
```

These modules are allowed to call `std::process::Command`, `fs`, `openssh`, etc. App services call them; domain code does not.

## Layer 5: UI

Prompting and printing should be isolated:

```text
prompts
progress messages
confirmation text
human-readable output
```

Right now prints are spread through orchestration code. That is fine for a CLI prototype, but it makes code hard to test and hard to read.

# Proposed `bonesdeploy` structure

I would move toward this shape:

```text
crates/bonesdeploy/src/
  main.rs
  cli/
    mod.rs
    args.rs
    dispatch.rs

  app/
    init_project.rs
    deploy_project.rs
    push_state.rs
    pull_state.rs
    remote_setup.rs
    remote_runtime.rs
    remote_ssl.rs
    update.rs
    rollback.rs
    doctor.rs

  domain/
    project.rs
    remote.rs
    runtime.rs
    deploy_target.rs

  config/
    mod.rs
    model.rs
    store.rs
    defaults.rs

  infra/
    git.rs
    ssh.rs
    rsync.rs
    process.rs
    filesystem.rs
    bonesinfra.rs
    embedded_kit.rs
    cargo_install.rs

  ui/
    prompts.rs
    output.rs
```

The names can change, but the boundary should not.

For example, `commands/init.rs` should probably become several things:

```text
cli/init.rs
  Parses InitArgs and calls app::init_project.

app/init_project.rs
  Orchestrates init.

config/store.rs
  load/save bones.toml and runtime.toml.

infra/embedded_kit.rs
  writes kit files.

infra/git.rs
  ensures repo, adds remote, infers remote.

ui/prompts.rs
  asks questions.
```

That would make `init` readable again.

# Proposed `bonesremote` structure

`bonesremote` should be even more domain-driven because it is the remote release executor.

```text
crates/bonesremote/src/
  main.rs
  cli/
    mod.rs
    args.rs
    dispatch.rs

  app/
    deploy.rs
    rollback.rs
    doctor.rs
    init.rs
    service.rs

  release/
    lifecycle.rs
    state.rs
    naming.rs
    activation.rs
    cleanup.rs
    publish.rs
    shared_paths.rs
    checkout.rs
    scripts.rs

  infra/
    git.rs
    fs.rs
    process.rs
    privileges.rs
    systemd.rs
    sudo.rs

  config/
    mod.rs
    model.rs
    store.rs
```

The current `deploy::run_full` should probably become:

```text
app/deploy.rs
```

And the lower-level work inside current `deploy.rs`, `post_receive.rs`, `wire_release.rs`, `stage_release.rs`, `activate_release.rs`, `drop_failed_release.rs`, and `post_deploy.rs` should become the `release/` domain/lifecycle area.

The current top-level remote command is good conceptually:

```text
bonesremote deploy --config ... --revision ...
```

But internally it should read like:

```rust
DeployLifecycle::new(config, revision)
    .preflight()
    .stage()
    .checkout()
    .wire_shared_paths()
    .run_scripts()
    .publish()
    .activate()
    .restart_services()
    .prune()
    .cleanup_on_failure()
```

Not necessarily with that exact fluent API, but the code should read that clearly.

# What I would change first

## 1. Rename `commands` mentally to `cli`

Keep the folder for now if renaming is too noisy, but conceptually `commands` should become thin.

Current `commands/mod.rs` has the CLI enum and routing in one file.   I would split that first:

```text
commands/mod.rs       // temporary facade
cli/args.rs           // clap structs/enums
cli/dispatch.rs       // maps args to app services
app/*                 // actual behavior
```

This immediately makes it easier to see where behavior lives.

## 2. Extract app services without changing behavior

Start with `init`, because it is the most tangled.

Create:

```text
app/init_project.rs
```

Move the high-level `run()` sequence there, but do not change the logic yet.

Then peel out helpers:

```text
config/store.rs       // config save/load
infra/git.rs          // git repo/remote actions
infra/embedded_kit.rs // scaffold kit
ui/prompts.rs         // questions/confirmations
```

The first pass can be mostly file movement and renaming.

## 3. Extract `bonesremote` deployment lifecycle

Current `bonesremote deploy::run_full` is the right conceptual center, but it lives beside lower-level deploy script execution.  Split it:

```text
app/deploy.rs
release/scripts.rs
release/publish.rs
release/cleanup.rs
release/checkout.rs
release/shared_paths.rs
```

Then leave old CLI commands as wrappers around the new release modules for now.

## 4. Move all subprocess calls behind adapters

Right now `git.rs`, `push.rs`, `update.rs`, `update_release.rs`, `ssh.rs`, and `bonesinfra.rs` all call processes directly. That is okay at the adapter layer, but not in app services.

Introduce a simple convention:

```text
Only infra/* may call std::process::Command.
Only infra/ssh.rs may open SSH sessions.
Only ui/* may prompt.
Only config/* may serialize/deserialize config files.
```

No need for traits everywhere yet. Start with folder boundaries.

## 5. Split `shared::paths`

`shared::paths` is doing too much. It should become something like:

```text
shared/src/paths/
  mod.rs
  local.rs
  remote.rs
  install.rs
  nginx.rs
  runtime.rs
```

Keep the public names re-exported temporarily so you do not break everything at once.

The important separation:

```text
local .bones paths != remote release paths != nginx/systemd paths != install paths
```

Right now they all live together.

## 6. Normalize config ownership

Both `bonesdeploy` and `bonesremote` define their own `BonesConfig`, and both wrap shared config types.   That is not terrible, but it is drifting.

I would move toward:

```text
shared::config::ProjectConfig
shared::config::Data
shared::config::Releases
shared::config::Ssl maybe optional/local-only
```

Then each binary can have a small `config/store.rs` for file IO, but the actual model should live in one place.

# Boundaries I would enforce with tests

You already have `tests/cleancode`. Add architectural tests that scan source files.

## Suggested tests

```text
1. commands/ or cli/ must not contain std::process::Command
2. commands/ or cli/ must not call fs::write, fs::remove_file, fs::create_dir_all
3. app/ must not call std::process::Command directly
4. domain/ and release/ must not import clap
5. domain/ must not import openssh
6. bonesdeploy must not embed or reference ../../infra
7. hook scripts must not call lower-level bonesremote lifecycle commands
8. shared/paths.rs must stay small or be split
```

The line-count test is good, but architecture tests will catch the kind of mess you actually hate.

# Refactor order I recommend

Do this in small boring commits.

## Pass 1: Name the layers

No behavior changes.

```text
- Add folders: cli/, app/, infra/, ui/
- Move prompts.rs to ui/prompts.rs
- Move git.rs, ssh.rs, bonesinfra.rs, embedded.rs to infra/
- Keep re-export modules if needed
```

## Pass 2: Thin `commands`

```text
- Move clap structs/enums into cli/args.rs
- Move dispatch into cli/dispatch.rs
- commands/mod.rs becomes a compatibility facade or disappears
```

## Pass 3: Extract `app/init_project`

```text
- Move init orchestration out of command file
- Keep helper names mostly the same
- Do not redesign config yet
```

## Pass 4: Extract `app/remote_*`

```text
- remote setup/runtime/ssl become app services
- Python calls go through infra/bonesinfra_runner.rs
- remote_data moves toward domain or app/infra boundary
```

## Pass 5: Reshape `bonesremote` around release lifecycle

```text
- app/deploy.rs owns high-level deploy
- release/* owns release operations
- infra/process.rs owns subprocess execution
- old hook commands become wrappers or internal-only
```

## Pass 6: Split shared paths/config

```text
- shared config becomes the single model
- per-crate config modules become file stores
- shared paths split by local/remote/install/service concerns
```

# The clean-code rule I would use

Every file should be answerable in one sentence.

Examples:

```text
cli/args.rs
  Defines CLI shape.

app/init_project.rs
  Orchestrates local project initialization.

infra/git.rs
  Wraps local git commands.

infra/bonesinfra.rs
  Locates and runs hidden Python infra.

release/lifecycle.rs
  Runs the remote deployment lifecycle.

release/state.rs
  Reads/writes release state.

release/scripts.rs
  Finds and runs deployment scripts.

config/store.rs
  Loads and saves config files.
```

If a file needs a paragraph to explain, it probably owns too much.

# My blunt take

Do **not** start by rewriting logic. Start by moving code into clearer places.

Right now the product direction is getting cleaner, but the source tree still reflects the older “move fast and wire things together” phase. The highest-leverage move is to make the folder structure express the architecture:

```text
CLI -> app service -> domain -> infra adapter
```

Once that exists, the remaining cleanup becomes obvious.
