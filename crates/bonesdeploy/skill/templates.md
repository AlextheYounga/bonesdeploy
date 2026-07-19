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
  --template laravel --runtime-var php_version=8.5 --runtime-var install_queue_worker=true
```

`--template none` or omitting the flag means "build from scratch" — no
framework runtime, you wire your own. Most projects pick a template.

## The templates

### laravel

PHP + PHP-FPM. Ships the Laravel queue worker optionally.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `php_version` | choice | 8.2, 8.3, 8.4, 8.5 | 8.5 |
| `install_queue_worker` | bool | — | false |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template laravel --runtime-var php_version=8.5 --runtime-var install_queue_worker=true
```

### django

Python + Gunicorn.

| Key | Type | Choices | Default |
|-----|------|---------|---------|
| `wsgi_module` | text | — | `config.wsgi:application` |
| `python_version` | choice | 3.11, 3.12, 3.13 | 3.12 |
| `install_postgres` | bool | — | false |
| `static_root` | text | — | `staticfiles` |
| `media_root` | text | — | `media` |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template django --runtime-var python_version=3.13 --runtime-var install_postgres=true
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
| `ruby_version` | choice | 3.2, 3.3, 3.4 | 3.3 |
| `install_postgres` | bool | — | false |
| `install_redis` | bool | — | false |
| `rails_env` | text | — | `production` |

```
bonesdeploy init --non-interactive --project-name atlas --host deploy.example.com \
  --template rails --runtime-var ruby_version=3.4 --runtime-var install_postgres=true
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

## Changing templates later

Already provisioned and want to switch framework? Run:

```
bonesdeploy remote runtime --yes
```

You'll be prompted again. This is the one case where `remote runtime` runs
on its own, outside of `setup`.
