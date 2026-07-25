# Prepare Scripts

Scripts in this directory run after the build is promoted into a release.

## Environment

- Runs as the site's runtime user (e.g. `<project>`, not root and not `<project>-build`).
- Working directory: the promoted release directory.
- **Has access** to `.env`, `shared/` symlinks (storage, bootstrap/cache, database.sqlite), and the database.

## Contract

- Scripts run in lexical order by filename.
- Non-zero exit code fails the deploy.
- This is where migrations, cache/optimize, and runtime-state commands belong.

## Typical Laravel Commands

```sh
php artisan key:generate --force
php artisan storage:link --force
php artisan migrate --force
php artisan optimize:clear
php artisan optimize
php artisan package:discover --ansi
php artisan queue:restart
php artisan up
```

## Adding Scripts

Name them with a numbered prefix so the order is clear:

```text
01_prepare_laravel.sh
02_custom_migration.sh
```
