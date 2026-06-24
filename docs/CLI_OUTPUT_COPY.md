# CLI Output Copy

This document defines the target user-facing text for bonesdeploy CLI output.

## Output Rule

Normal command output should only show:

- what is happening
- whether it worked
- what to run next

Use short, direct lines. Put background explanation in docs or help text.

Preferred shape:

```txt
Applying runtime...
Runtime applied.

Next: run bonesdeploy push.
```

Avoid:

```txt
This applies per-site runtime configurations to the server.

It will:
  - Ensure runtime-specific packages are installed.
  - Provision runtime-specific services...
```

Implementation details like exact paths, symlinks, scaffold internals, hook cleanup, and source versions should not appear in normal output unless they are needed to fix a problem.

## Prompts

### Runtime Template

Use:

```txt
Runtime template:
```

Options should include the available runtimes plus:

```txt
Build from scratch
```

Help text, if needed:

```txt
Choose the app runtime to configure.
```

### Template Selection

Use:

```txt
Template:
```

Do not add help text unless a specific template needs clarification.

### Deployment Remote

Use:

```txt
Deployment remote:
```

Help text:

```txt
Choose the VPS remote, not your code host.
```

If `origin` is selected:

```txt
Warning: origin usually points to your code host, not your VPS.
```

Confirmation:

```txt
Use 'origin' anyway?
```

Help text:

```txt
Choose No unless origin points to your VPS.
```

Abort text:

```txt
Choose a deployment remote that points to your VPS.
```

### Host And Port

Use:

```txt
Server host or IP:
```

Help text:

```txt
e.g. deploy.example.com or 203.0.113.10
```

Use:

```txt
SSH port:
```

### Deployment Remote Name

Use:

```txt
Deployment remote name:
```

Help text:

```txt
Created if missing.
```

### SSL

Use:

```txt
Domain:
```

Help text:

```txt
e.g. app.example.com
```

Use:

```txt
Let's Encrypt email:
```

Help text:

```txt
e.g. ops@example.com
```

## Confirmations

### Remote Bootstrap

Use:

```txt
Remote bootstrap prepares the VPS for this project.
```

Prompt:

```txt
Bootstrap remote server?
```

### Remote Runtime

Use:

```txt
Runtime setup installs app services for this project.
```

Prompt:

```txt
Apply runtime setup?
```

### Remote SSL

Use:

```txt
HTTPS requires DNS to point at this server.
```

Prompt:

```txt
Configure HTTPS?
```

## Init

Normal output:

```txt
Initializing bonesdeploy...
bonesdeploy initialized.

Next: run bonesdeploy setup.
```

If existing config was updated:

```txt
Initializing bonesdeploy...
bonesdeploy config updated.

Next: run bonesdeploy setup.
```

If `.bones` already exists:

```txt
Using existing .bones config.
```

Runtime template output:

```txt
Runtime template: {template_name}
```

For a custom runtime:

```txt
Runtime template: custom
```

Move these implementation details out of normal output:

```txt
Preparing local bonesinfra...
Local bonesinfra ready.
Creating {}...
Symlinked .bones -> {}
Saved config to .bones/bones.toml
Saved runtime config to .bones/runtime.toml
Created config .gitignore at {}
Added .bones to .gitignore
Symlinked .git/hooks/pre-push -> .bones/hooks/pre-push
Configured local git remote {} -> {}
```

## Setup

Use for the new `bonesdeploy setup` command:

```txt
Setting up deployment...
Bootstrapping remote server...
Applying runtime...
Syncing .bones...
Checking deployment...

Setup complete.

Next: run bonesdeploy remote ssl to configure HTTPS.
```

If SSL is already configured:

```txt
Setup complete.

Next: run bonesdeploy deploy.
```

If setup fails:

```txt
Setup failed while {step}.

Next: fix the error above, then run bonesdeploy setup again.
```

## Remote Bootstrap

Direct command output:

```txt
Bootstrapping remote server...
Remote bootstrap complete.

Next: run bonesdeploy remote runtime.
```

## Remote Runtime

Success:

```txt
Applying runtime...
Runtime applied.

Next: run bonesdeploy push.
```

Skipped:

```txt
Skipped runtime setup.

Next: run bonesdeploy remote runtime when ready.
```

Missing runtime config:

```txt
Missing .bones/runtime.toml.

Next: run bonesdeploy init.
```

## Remote SSL

Success:

```txt
Configuring HTTPS for {domain}...
HTTPS configured.

Next: run bonesdeploy deploy.
```

Skipped:

```txt
Skipped HTTPS setup.

Next: run bonesdeploy remote ssl when DNS is ready.
```

Missing domain:

```txt
Missing domain.

Next: pass --domain or set domain in .bones/bones.toml.
```

Missing email:

```txt
Missing Let's Encrypt email.

Next: pass --email or set email in .bones/bones.toml.
```

## Push

Success:

```txt
Syncing .bones...
.bones synced.
```

If run directly:

```txt
Syncing .bones...
.bones synced.

Next: run bonesdeploy doctor.
```

Failure:

```txt
Failed to sync .bones.

Next: check SSH access and run bonesdeploy push again.
```

Move these implementation details out of normal output:

```txt
Cleaning sample hooks from remote...
Symlinking hooks...
```

## Pull

Success:

```txt
Pulling .bones...
.bones pulled.
```

If run directly:

```txt
Pulling .bones...
.bones pulled.

Next: run bonesdeploy doctor.
```

## Doctor

Success:

```txt
Checking deployment...

✓ .bones config
✓ deployment scripts
✓ remote SSH
✓ bonesremote
✓ .bones sync

All checks passed.
```

Failure:

```txt
Checking deployment...

✓ .bones config
✗ .bones sync
  Next: run bonesdeploy push
✓ deployment scripts
✓ remote SSH

Doctor found 1 issue.
```

Use these issue lines when applicable:

```txt
✗ Missing .bones config
  Next: run bonesdeploy init
```

```txt
✗ .bones is not managed by bonesdeploy
  Next: run bonesdeploy init
```

```txt
✗ Missing .bones/bones.toml
  Next: run bonesdeploy init
```

```txt
✗ Deployment script is not ordered: {name}
  Next: rename it with a numeric prefix, like 01_build.sh
```

```txt
✗ pre-push hook is not installed
  Next: run bonesdeploy init
```

```txt
✗ Cannot connect to remote
  {error}
  Next: check host, port, and SSH access.
```

```txt
✗ bonesremote is missing
  Next: run bonesdeploy remote bootstrap
```

```txt
✗ .bones is not synced to the remote
  Next: run bonesdeploy push
```

```txt
✗ .bones has local changes
  Next: run bonesdeploy push
```

## Deploy

Keep the remote deploy stream unchanged for now.

Only clean the local wrapper lines:

```txt
Deploying {project} to {host}...
Running remote deploy...
Deployment complete.
```

## Rollback

Use:

```txt
Rolling back {project} on {host}...
Rollback complete.

Next: run bonesdeploy status.
```

## Secrets

Init:

```txt
Secrets initialized.

Next: run bonesdeploy secrets edit.
```

Edit:

```txt
Secrets updated.

Next: run bonesdeploy secrets push.
```

Push:

```txt
Secrets pushed.
```

Temporary file warning:

```txt
Warning: could not remove temporary secret file: {path}
```

Missing `.bones`:

```txt
Missing .bones config.

Next: run bonesdeploy init.
```

Missing encrypted secrets:

```txt
Missing encrypted secrets.

Next: run bonesdeploy secrets edit.
```

Missing gpg:

```txt
gpg is required.

Next: install gpg and try again.
```

## Update

Normal output:

```txt
Checking for updates...
```

If current:

```txt
Already up to date.
```

If updates are needed:

```txt
Updating bonesdeploy...
Updating bonesremote...
Update complete.
```

Move these details out of normal output:

```txt
Current local version: ...
Current remote version: ...
Source branch: master
Checking master version from ...
Master bonesdeploy version: ...
Master bonesremote version: ...
Refreshing local .bones scaffold...
```

## Manage

If keeping `manage`, use:

```txt
Opening remote manage session...
```

Failure:

```txt
Could not open remote manage session.

Next: run bonesdeploy status or check SSH access.
```

## Guide

Human-readable output:

```txt
State: initialized, setup not complete.

Next: bonesdeploy setup --yes
```

JSON output shape:

```json
{
  "state": "initialized_setup_missing",
  "next": {
    "command": "bonesdeploy setup --yes",
    "mutates": true,
    "contacts_remote": true
  },
  "missing": ["remote_bootstrap", "runtime", "bones_sync", "doctor_pass"]
}
```
