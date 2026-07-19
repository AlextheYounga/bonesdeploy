# BonesDeploy runtime templates

Seven templates ship in the binary. Each one installs a framework-specific
runtime on the server: nginx router, systemd service, AppArmor profile, and
whatever language runtime the framework needs. You pick one at `init` time.

## Picking a template

Interactive:

```
bonesdeploy init
```

You'll get a menu. Pick one.

Non-interactive (agents, CI):

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template laravel --runtime-var php_version=8.5
```

`--template none` or omitting the flag means "build from scratch" — no
framework runtime, you wire your own. Most projects pick a template.

## The templates

### laravel

PHP + PHP-FPM.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `php_version` | choice | 8.2, 8.3, 8.4, 8.5 | 8.5 |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template laravel --runtime-var php_version=8.5
```

### django

Python + Gunicorn.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `wsgi_module` | text | — | `config.wsgi:application` |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template django --runtime-var wsgi_module=project.wsgi:application
```

### next

Next.js. Static export or Node server.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `is_static` | bool | — | true |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template next --runtime-var is_static=false
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
  --template nuxt --runtime-var is_static=false
```

### rails

Ruby + Puma.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `rails_env` | text | — | `production` |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template rails --runtime-var rails_env=production
```

### sveltekit

SvelteKit. Node server. No runtime vars.

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template sveltekit
```

### vue

Vue. Static export. No runtime vars.

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template vue
```

## Validation

`--runtime-var` answers are validated against the template's schema before
they reach `bones.toml`. Unknown keys, wrong types, and out-of-range choices
are rejected. You can't typo `php_verison` and ship a broken config.
