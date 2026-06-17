You are helping me refactor BonesDeploy.

Repositories:
- Rust/main repo: https://github.com/AlextheYounga/bonesdeploy
- Python infra repo: https://github.com/AlextheYounga/bonesinfra

Goal:
Create a simpler architecture for BonesDeploy. Do not overbuild. Do not preserve legacy behavior unless explicitly necessary. I want directional cleanup, not compatibility layers.

Current direction:
BonesDeploy should stop being framed primarily as a “git push deployment tool.” It should become a remote release deployment tool.

Git should remain supported, but only as one possible source/trigger. Git should not drive the architecture.

The desired conceptual model:

- bonesdeploy:
  Local user-facing Rust CLI.
  Owns public UX, project init, config prompting, local setup, installing/updating bonesremote, installing/updating hidden bonesinfra, and core kit/hook assets.

- bonesremote:
  Remote Rust release lifecycle executor.
  Owns staging, checkout/build workspace, shared-path wiring, deploy script execution, release publishing, activation, rollback, pruning, service restart, and failure cleanup.

- bonesinfra:
  Hidden Python infra/provisioning engine.
  Owns pyinfra API usage, runtime-specific provisioning, runtime questions/defaults, framework-specific operations, nginx/PHP/Python/Node/etc. templates, runtime doctor checks, and setup/runtime/ssl apply internals.

- git:
  Optional source/trigger mechanism.
  Should not own deployment lifecycle.

Important user-facing rule:
The user should only interact with `bonesdeploy`. The user should not need to know `bonesinfra` exists.

Public UX should remain shaped around commands like:

    bonesdeploy init
    bonesdeploy remote setup
    bonesdeploy remote runtime
    bonesdeploy remote ssl
    bonesdeploy deploy
    bonesdeploy push

Do not expose `bonesinfra` as a public product command.

Key architectural concern:
The current git hook system is too complex. Deployment lifecycle is spread across pre-receive, post-receive, shell hook libraries, and multiple bonesremote commands. This should be collapsed.

Desired future direction:
Create one high-level remote deployment command, conceptually:

    bonesremote deploy --config bones/bones.toml --revision <sha>

That command should own the full remote deployment sequence internally.

Conceptual deploy sequence:

1. load config
2. run doctor/preflight
3. stage release/build workspace
4. checkout revision
5. wire shared paths
6. run deployment/build scripts
7. publish release tree
8. activate release
9. restart services
10. prune old releases
11. on failure, drop failed staged release / leave current release unchanged

Existing lower-level bonesremote commands/modules can remain internally useful, but shell hooks should not coordinate the lifecycle.

Git hook simplification:
For git-based deployments, reduce hooks to a thin adapter.

Prefer one post-receive hook that:
1. resolves whether the configured branch was updated
2. gets the new revision
3. calls:

    bonesremote deploy --config "$GIT_DIR/bones/bones.toml" --revision "$NEWREV"

Avoid pre-receive deployment behavior.

Accept this tradeoff:
Deployment failure should no longer reject the git push. A failed deploy should mean:
- source revision exists on the server
- current release remains unchanged
- deployment failed visibly
- next deploy can retry or rollback

This is cleaner than tying source upload success to deployment success.

Kit ownership:
The core kit belongs in `bonesdeploy`, not `bonesinfra`.

Reason:
The kit contains hook scripts and lifecycle glue that call `bonesdeploy` and `bonesremote`. That is Rust/product lifecycle behavior, not Python/pyinfra behavior.

Python ownership:
`bonesinfra` should own pyinfra-related operations and assets only. It should not own the public CLI. It should not own git hook orchestration. It should not own bonesremote release lifecycle.

Rust/Python boundary:
Rust should not call the pyinfra CLI directly.
Rust should not know pyinfra internals.
Rust should eventually call a hidden Python engine through a small wrapper/manager.

Conceptual Rust-side wrapper:

    ensure_bonesinfra_available()
    bonesinfra_runtime_list()
    bonesinfra_runtime_questions(runtime)
    bonesinfra_runtime_defaults(runtime)
    bonesinfra_setup_apply(...)
    bonesinfra_runtime_apply(...)
    bonesinfra_ssl_apply(...)

For packaging the bonesinfra, we will download the Python git repo from https://github.com/AlextheYounga/bonesinfra.

Important separation:
Provisioning and deployment are different concerns.

Provisioning:
- install packages
- create users
- configure nginx
- configure AppArmor
- create services
- install runtime dependencies
- SSL setup

This mostly belongs to `bonesinfra` and should largely not be a concern of bonesdeploy except for doctor commands.

Deployment:
- move code/revision to server
- build
- stage release
- wire shared paths
- activate release
- restart services
- rollback/prune

This belongs to `bonesremote`.

Config:
Use config files as the shared contract between Rust, Python, and remote execution. (bones.toml and runtime.toml)
Avoid hidden assumptions through shell globals, folder layout, or hook state.

Do not spend time changing config format unless directly required.
The important rule is:
- data/config is shared
- logic belongs to the correct engine

Suggested staged plan:

Stage 1: Ownership cleanup
- Keep kit in bonesdeploy.
- Keep Python infra in bonesinfra.
- Rename bonesinfra package metadata clearly if needed.
- Remove Rust assumptions that Python infra source lives inside the bonesdeploy workspace.

Stage 2: Python wrapper boundary
- Add a Rust-side bonesinfra manager/wrapper.
- Let Rust download/find/install bonesinfra.
- Keep bonesinfra hidden from users.
- Stop embedding Python source into the Rust binary.

Stage 3: Single remote deploy command
- Add or promote `bonesremote deploy`.
- Move lifecycle sequencing into Rust.
- Reuse existing stage/checkout/wire/deploy/activate/prune modules internally where useful.

Stage 4: Hook collapse
- Remove pre-receive deployment behavior.
- Replace multi-step shell orchestration with one thin post-receive hook.
- Hook only resolves revision and calls `bonesremote deploy`.

Stage 5: Product reframing
- Docs should stop presenting git hooks as the center.
- Git deployment becomes one supported mode.
- `bonesdeploy deploy` becomes the conceptual center.

Things to avoid:
- Do not preserve both old and new orchestration paths indefinitely.
- Do not keep pre-receive and post-receive both coordinating deploy state.
- Do not let shell scripts own release lifecycle.
- Do not make Rust embed Python infra long-term.
- Do not make Rust know pyinfra details.
- Do not make Python own bonesremote release lifecycle.
- Do not require users to call bonesinfra directly.
- Do not let git push remain the only mental model for deployment.
- Do not add compatibility fallbacks unless they are explicitly requested.

Guiding principle:

    One public CLI.
    One remote deploy lifecycle.
    One hidden Python infra engine.
    Git is optional plumbing.

Your task:
Analyze the current repos and propose the smallest practical refactor plan that moves the codebase toward this architecture.

Do not implement a giant migration all at once.
Prefer a staged plan with narrow commits.
Call out files/areas that should change, but avoid unnecessary speculative detail.
When uncertain, leave details intentionally vague rather than inventing abstractions.