You are working on the BonesDeploy repository.

Repository branch: feat/python

Goal:
Migrate BonesDeploy toward a Python-owned infra/runtime system and deliberately remove old legacy runtime behavior instead of preserving fallbacks.

Context:
The latest commit on this branch is mostly a structural move toward the desired shape. Please inspect that commit and the current repository layout before making changes, especially infra/*.

The intended direction is:

- kit files now live under crates/bonesdeploy/kit/
- infra files now live under infra/
- runtime-specific infra files now live under infra/src/runtimes/
- infra/main.py exists and should become the Python entrypoint
- Rust should call Python entrypoints instead of embedding/scanning runtime templates

Core principle:
Do not support old and new behavior side by side. For each subsystem, remove the old implementation, create the new implementation, fix all callsites, and add tests or validation so stale legacy behavior fails loudly.

Do not create compatibility fallbacks such as:
- old embeds/runtimes fallback
- old operations.py fallback
- old runtime discovery fallback
- old service_user/group aliases
- pyinfra CLI fallback if the new programmatic runner fails

Rust should remain the durable CLI/orchestrator.
Python should own infra/runtime logic.

Target division:

Rust owns:
- CLI UX
- prompting
- writing .bones project config
- git hook setup
- syncing .bones to remote
- calling Python entrypoints
- bonesremote release lifecycle

Python owns:
- runtime discovery
- runtime-specific questions
- runtime defaults
- framework-specific infra logic
- pyinfra operations
- template rendering
- runtime doctor checks

Migration tasks:

1. Fix Rust embed paths and remove the old runtime embed model.

- In crates/bonesdeploy/src/embedded.rs, stop embedding ./embeds/kit and ./embeds/runtimes.
- Embed kit from ./kit.
- Embed infra from ../../infra.
- Remove the Runtimes embed entirely.
- Delete or replace functions that depend on embedded runtime templates:
  - scaffold_runtime_template
  - read_template_runtime_config
  - available_templates if it depends on embedded runtime folders
- Runtime discovery should move to Python.

2. Implement infra/main.py.

- infra/main.py should become the single Python entrypoint for infra-related commands.
- It should be executable from the scaffolded .bones/infra/main.py location.
- Do not preserve direct pyinfra deploy-file execution as the main command path.
- Commands that are not implemented yet should fail loudly with clear errors.

3. Create a Python runtime registry.

- Implement infra/src/runtimes/__init__.py.
- It should expose:
  - list_runtimes() -> list[str]
  - get_runtime(name: str) -> module
- Prefer an explicit controlled registry over loose filesystem scanning unless there is a strong reason otherwise.
- Runtime discovery should come from this Python registry, not Rust embedded folders.

4. Add initial Python CLI commands.

Implement these first:

- python .bones/infra/main.py runtime list --json
- python .bones/infra/main.py runtime questions <runtime> --json
- python .bones/infra/main.py runtime defaults <runtime> --json

Also reserve these command shapes for later implementation:

- python .bones/infra/main.py setup apply --config .bones/bones.yaml
- python .bones/infra/main.py runtime apply --config .bones/bones.yaml --runtime-config .bones/runtime.yaml
- python .bones/infra/main.py ssl apply --config .bones/bones.yaml

Unimplemented commands should fail loudly. Do not fall back to old behavior.

5. Move runtime logic into Python modules.

Runtime modules should expose:

- questions()
- defaults()
- shared_paths(ctx)
- apply(ctx), even if initially not implemented

Start with Laravel.

Example shape:

def questions():
    return [
        {
            "key": "php_version",
            "type": "choice",
            "label": "PHP version",
            "choices": ["8.2", "8.3", "8.4"],
            "default": "8.3",
        },
        {
            "key": "install_queue_worker",
            "type": "bool",
            "label": "Install Laravel queue worker?",
            "default": False,
        },
    ]

def defaults():
    return {
        "php_version": "8.3",
        "install_queue_worker": False,
        "shared_paths": [
            {"path": ".env", "type": "file", "required": True},
            {"path": "storage", "type": "dir", "required": True},
        ],
    }

def shared_paths(ctx):
    return defaults()["shared_paths"]

def apply(ctx):
    raise NotImplementedError("laravel apply is not migrated yet")

6. Keep framework-specific values out of bones.yaml.

bones.yaml should remain focused on project identity and deploy mechanics, such as:

- project_name
- host
- port
- repo_path
- project_root
- web_root
- branch
- deploy_user
- runtime_user
- runtime_group
- release_group

Selected runtime answers should live separately in .bones/runtime.yaml.

runtime.yaml should store selected runtime configuration/answers. Python defines the runtime logic. The data file stores the user’s selected values.

Do not keep runtime-specific logic in YAML.

7. Make Rust call Python for runtime discovery and questions.

Replace Rust runtime discovery based on embedded runtime folders.

Rust should call:

- .bones/infra/main.py runtime list --json
- .bones/infra/main.py runtime questions <runtime> --json
- .bones/infra/main.py runtime defaults <runtime> --json

Rust should:
- parse stdout JSON
- render prompts
- write selected runtime answers to .bones/runtime.yaml
- avoid knowing framework-specific details like PHP-FPM, Gunicorn, Node, Composer, etc.

8. Refactor setup/runtime/ssl toward Python entrypoint execution.

The eventual direction is:

- infra/main.py is the only infra command entrypoint
- setup/runtime/ssl behavior is invoked through main.py
- runtime-specific behavior is delegated to Python runtime modules
- pyinfra is run programmatically from Python

Do not implement fallback execution through the old pyinfra CLI path.

9. Add legacy tripwires.

Add validation/tests so old paths or old config concepts fail loudly instead of silently being supported.

Reject or fail on stale concepts such as:

- crates/bonesdeploy/embeds/runtimes
- Rust runtime template scanning
- operations.py as runtime entrypoint
- service_user old config key
- old group alias behavior
- permissions: old config section
- releases.shared_files
- releases.shared_dirs
- scaffold_runtime_template
- read_template_runtime_config

Do not silently ignore these if they can remain hidden and cause confusion.

10. Update docs.

Update README/docs to describe only the new model.

Docs should describe:
- the new .bones/infra/main.py entrypoint
- Python-owned runtime discovery/questions/defaults
- Rust calling Python for runtime info
- runtime answers stored separately from bones.yaml
- no old embedded runtime template model

Avoid documenting old behavior except as a breaking-change warning if needed.

Important design rules:

- Follow the file shape introduced in the latest commit on feat/python.
- Prefer breaking loudly over preserving hidden legacy behavior.
- No fallback loaders.
- No dual old/new runtime discovery paths.
- No compatibility aliases unless removed in the same commit.
- Python defines runtime logic.
- Data files store selected values and project identity.
- Rust should not know framework-specific infra details.
- Runtime-specific questions must live in Python modules.
- Every old subsystem removed should have a test or validation proving it is gone.

Suggested stage breakdown:

1. Fix Rust embed paths for the new layout.
2. Remove old runtime embed/scanning model.
3. Implement infra/main.py.
4. Add Python runtime registry.
5. Add runtime list/questions/defaults CLI commands.
6. Move Laravel runtime questions/defaults into Python.
7. Make Rust use Python for runtime list/questions/defaults.
8. Persist selected runtime answers to .bones/runtime.yaml.
9. Refactor setup/runtime/ssl toward main.py execution.
10. Move pyinfra execution behind the Python entrypoint.
11. Add legacy tripwire validation.
12. Update tests and docs.

Acceptance criteria:

- cargo test passes.
- bonesdeploy init scaffolds the new .bones layout.
- .bones/infra/main.py is executable.
- python .bones/infra/main.py runtime list --json returns available runtimes.
- python .bones/infra/main.py runtime questions laravel --json returns a php_version question.
- python .bones/infra/main.py runtime defaults laravel --json returns Laravel defaults.
- Rust no longer embeds or scans old runtime templates.
- No code references crates/bonesdeploy/embeds/runtimes.
- No code depends on old runtime template discovery.
- Runtime-specific logic is in Python modules.
- Old legacy config keys fail loudly.
- The documented path uses Python entrypoints, not old pyinfra CLI deploy files.
