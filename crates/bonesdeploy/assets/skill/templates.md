# BonesDeploy framework templates

Seven templates ship in the binary. Each one provisions the runtime on the
server: nginx router, systemd service, AppArmor profile, and
whichever language runtime the framework needs. You pick one at `init` time.

## Picking a template

Interactive:

```
bonesdeploy init
```

You'll get a menu. Pick one.

Non-interactive (agents, CI):

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template laravel --framework-var php_version=8.5
```

`--template none` or omitting the flag means "build from scratch" — no
runtime is provisioned; you wire your own. Most projects pick a template.

## The templates

### laravel

PHP + PHP-FPM. Set `install_queue_worker=true` to provision an opt-in Laravel
`queue:work` systemd service that consumes the application's configured queue.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `php_version` | choice | 8.2, 8.3, 8.4, 8.5 | 8.5 |
| `install_queue_worker` | bool | — | false |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template laravel --framework-var php_version=8.5
```

Enable the queue worker with `--framework-var install_queue_worker=true`.

### django

Python + Gunicorn.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `python_version` | choice | 3.12, 3.13, 3.14 | 3.14 |
| `wsgi_module` | text | — | `config.wsgi:application` |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template django --framework-var wsgi_module=project.wsgi:application
```

### next

Next.js. Static export or Node server.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `is_static` | bool | — | true |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template next --framework-var is_static=false
```

Static Next serves from `out/`. Server Next runs the standalone server on
an internal port behind nginx.

### nuxt

Nuxt. Static or Node server.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `is_static` | bool | — | true |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template nuxt --framework-var is_static=false
```

### rails

Ruby + Puma.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `rails_env` | text | — | `production` |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template rails --framework-var rails_env=production
```

### sveltekit

SvelteKit. Node server. No framework vars.

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template sveltekit
```

### vue

Vue. Static export. No framework vars.

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template vue
```

## Validation

`--framework-var` answers are validated against the template's schema before
they reach `bones.toml`. Unknown keys, wrong types, and out-of-range choices
are rejected. You can't typo `php_verison` and ship a broken config.
