# bonesdeploy remote runtime

## Overview

Prompts for a framework template, refreshes the local runtime scaffold, writes `.bones/runtime.yaml`, then asks whether to apply the runtime playbook on the server.

## Command Signature

```bash
bonesdeploy remote runtime
```

## What It Does

- Prompts for the framework template
- Refreshes `.bones/runtime/`
- Writes `.bones/runtime.yaml`
- Reapplies template-specific defaults into `.bones/bones.yaml` when they still match generic or previous-template values
- Passes `.bones/runtime.yaml` to the runtime playbook through `vars_files`
- Prompts `y/N` before running the runtime playbook remotely
- Runs `bonesremote doctor` after playbook completion

## What It Does NOT Do

- Does not handle SSL/TLS configuration (use `bonesdeploy remote ssl` for TLS)
- Does not run certbot or certificate challenges
- Does not pass any SSL-related variables to Ansible

## When to Run

1. After `bonesdeploy init` to choose a framework
2. When switching framework templates
3. After updating framework runtime assets in the repo
