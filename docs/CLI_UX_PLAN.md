# CLI UX Plan

This plan captures the agreed usability changes for the deployment CLI.

## Goals

- Keep the setup pieces internally separated, but give users a simpler first-run path.
- Make command output tell users what to do next, without long explanatory blurbs.
- Add a read-only status command so users can see whether a deployment is healthy.
- Improve doctor output so each check is visible and actionable.
- Make every command usable by coding agents without forced interactive prompts.

## Command Shape

The happy path should be:

1. `bonesdeploy init`
2. `bonesdeploy setup`
3. `bonesdeploy remote ssl`
4. `bonesdeploy deploy`

`init` should stay mostly local. It should create local `.bones` config/scaffolding, collect project/server config, prepare local repo integration, and tell the user the next command.

`setup` should be the first-run orchestrator. It should run:

1. remote bootstrap
2. remote runtime
3. push
4. doctor

The underlying commands should stay separated. The combined setup flow is a user-facing convenience, not a replacement for the lower-level pieces.

`remote ssl` should stay separate. After setup finishes, the CLI should guide the user toward SSL setup as a clear next step.

## Remote Bootstrap

Rename `bonesdeploy remote setup` to `bonesdeploy remote bootstrap`.

`bootstrap` is the foundation step that provisions the base server/project infrastructure. This name is clearer once top-level `bonesdeploy setup` exists.

Preferred command shape:

```txt
bonesdeploy setup
bonesdeploy remote bootstrap
bonesdeploy remote runtime
bonesdeploy remote ssl
```

If compatibility matters, keep `bonesdeploy remote setup` as a hidden or deprecated alias for `bonesdeploy remote bootstrap`, but docs and help should prefer `remote bootstrap`.

## User Guidance

Replace vague or long text blurbs with short next-step guidance.

Good output should answer: "What should I do next?"

Examples:

```txt
Next: run bonesdeploy setup.
```

```txt
Next: run bonesdeploy remote ssl to set up HTTPS.
```

Avoid long explanations during normal flows. Detailed background belongs in documentation, not routine CLI output.

## Prompt-Free Command Contract

Every command should be runnable without interactive prompts when the needed values are supplied by flags or existing config.

A forced `y/N` prompt is a CLI bug for this project. Coding agents and automation need a way to run the same command paths that humans run.

Rules:

- Any command that asks for confirmation must have a flag that bypasses that confirmation.
- Any command that asks for required input must allow that input through flags or config.
- Prompt-free commands should fail with a clear error when required values are missing.
- Prompt-free errors should say exactly which flag or config value is needed.

Examples:

```txt
bonesdeploy init --yes --project-name lawsnipe --host deploy.example.com
```

```txt
bonesdeploy setup --yes
```

```txt
bonesdeploy remote ssl --yes --domain app.example.com --email ops@example.com
```

```txt
bonesdeploy remote runtime --yes
```

## Status Command

Add a `bonesdeploy status` command.

The status command should be read-only and should help users understand whether the remote deployment is healthy.

It should report the most useful deployment facts first, such as:

- project
- host
- current release
- configured branch
- service state
- SSL state when available

## Agent Guidance Command

Add a read-only command that helps coding agents decide what to run next.

Preferred shape:

```txt
bonesdeploy guide
```

The command should inspect the current repository and print a short, prompt-free next step.

Example human-readable output:

```txt
Project: lawsnipe
State: initialized, runtime missing

Next: bonesdeploy setup --yes
Then: bonesdeploy remote ssl --yes --domain app.example.com --email ops@example.com
Then: bonesdeploy deploy
```

Agent-friendly output should also be available:

```txt
bonesdeploy guide --format json
```

The JSON output should include:

- detected state
- missing requirements
- recommended next command
- whether the recommended command is destructive
- whether the recommended command may contact the remote host
- the exact prompt-free command to run

This command should never mutate local or remote state. It should only guide the next action.

## Doctor Output

Improve `bonesdeploy doctor` so checks are shown individually.

Use per-check status markers:

```txt
✓ check passed
✗ check failed
⚠ check warning
```

Doctor output should make failures actionable. When a check fails, include the next command the user should run when there is an obvious fix.

Example:

```txt
✗ .bones is not synced to the remote
  Next: run bonesdeploy push
```

## Non-Goal

Do not change the wall-of-text deploy output right now.

`bonesdeploy deploy` should keep streaming the remote deploy output as it currently does. Changing this output is likely to introduce unnecessary problems at this stage.
