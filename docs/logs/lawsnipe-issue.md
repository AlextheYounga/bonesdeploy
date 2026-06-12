# Lawsnipe Deployment Diagnosis

## Issue 1: 403 Forbidden — Root Cause

**File:** `/srv/conf/lawsnipe/nginx.conf`, line 20

```
index index.php;
```

The `public/` directory only contains `index.html` and no `index.php`. When nginx processes `try_files $uri $uri/ /index.php?$query_string`:

1. `$uri` = `/` — this is a directory, so nginx moves on
2. `$uri/` — nginx tries to serve a directory listing with its index directive
3. Index directive only lists `index.php` — no match
4. No `autoindex on;` — nginx returns 403 Forbidden

The `try_files` never reaches `/index.php?$query_string` because nginx still attempts to locate an index file before falling through. The error log confirms:

```
directory index of "/srv/deployments/lawsnipe/current/public/" is forbidden
```

---

## Issue 2: PHP-FPM Service Fails to Start (exit code 78)

**Config file:** `/srv/conf/lawsnipe/php-fpm.conf`, line 2

```
error_log = /proc/self/fd/2
```

The service (`lawsnipe-php-fpm.service`) exits immediately with code 78 (PHP-FPM configuration error). The error:

```
ERROR: failed to open error_log (/proc/self/fd/2): No such device or address (6)
ERROR: failed to post process the configuration
ERROR: FPM initialization failed
```

**Why:** The service unit has `PrivateDevices=yes` and uses an AppArmor profile (`bonesdeploy-lawsnipe-php-fpm`). Under these sandboxing restrictions, `/proc/self/fd/2` is not accessible as a valid log target from the global error_log directive. This happens even when running interactively outside systemd, so it's an inherent combination of the sandbox restrictions and the config value.

**Impact:** No `/run/lawsnipe/php-fpm.sock` socket is ever created. The internal nginx cannot proxy PHP requests — any `.php` request would return a 502. Currently this isn't blocking the 403 since no `.php` file exists, but it will block any dynamic functionality.

**Needs:** Change `error_log = /proc/self/fd/2` to something that works under the sandbox, e.g.:
- `error_log = /dev/stderr` (if `/dev` is allowed)
- `error_log = /run/lawsnipe/php-fpm.log` (if a log file in the ReadWritePaths is acceptable)
- `error_log = syslog` (route through syslog)

---

## Issue 3: Release Timestamp is Unix Epoch

**Path:** `/srv/deployments/lawsnipe/releases/19700101_000000`

The release directory is named `19700101_000000` — that's midnight January 1, 1970 UTC (Unix epoch zero). The `current` symlink points to this release. This indicates the deployment system is failing to capture or format the actual deployment timestamp, resulting in a default/zero value.

**Severity:** Medium — deployment system bug, doesn't break serving but indicates something is wrong upstream.

---

## Issue 4: ACME Challenge Path May Break Cert Renewal

In `/etc/nginx/sites-available/lawsnipe.conf`, lines 14-16:

```
location ^~ /.well-known/acme-challenge/ {
    root /srv/deployments/lawsnipe/current/public;
    try_files $uri =404;
}
```

This location is handled by the **outer** system nginx, which runs as `www-data`. However, files in the `public/` directory are owned by `lawsnipe:root` with mode `640` (example: `index.html`). The `www-data` user cannot read these files. When Let's Encrypt tries to place and serve challenge files here, they will be unreadable by the outer nginx, causing certificate renewal failures.

**Severity:** Medium — cert renewal will fail.

---

## Issue 5: AppArmor Profiles May Add Restrictions

The service units reference AppArmor profiles:

- `lawsnipe-nginx.service`: `AppArmorProfile=bonesdeploy-lawsnipe-nginx`
- `lawsnipe-php-fpm.service`: `AppArmorProfile=bonesdeploy-lawsnipe-php-fpm`

If these profiles are more restrictive than the systemd sandbox settings, they could introduce additional failures. The PHP-FPM error could be partly or fully caused by AppArmor rather than (or in addition to) `PrivateDevices=yes`. If the AppArmor profile blocks access to `/proc/self/fd/2`, the fix for issue 2 must also ensure the chosen error_log path is allowed by the profile.

**Severity:** Low — verify if the profiles exist and what they restrict.

---

## Summary Table

| # | Problem | File | Severity |
|---|---------|------|----------|
| 1 | `index index.php;` but only `index.html` exists | `/srv/conf/lawsnipe/nginx.conf:20` | **Blocking** — causes 403 |
| 2 | `error_log = /proc/self/fd/2` fails under sandbox | `/srv/conf/lawsnipe/php-fpm.conf:2` | **Blocking** — PHP-FPM won't start |
| 3 | Release directory is epoch timestamp | `/srv/deployments/lawsnipe/releases/19700101_000000` | Medium — deployment system bug |
| 4 | Outer nginx (www-data) can't read lawsnipe files | `/etc/nginx/sites-available/lawsnipe.conf:15` | Medium — cert renewal will fail |
| 5 | AppArmor profiles may add restrictions | Service unit files & `/etc/apparmor.d/` | Low — verify if profiles exist |
