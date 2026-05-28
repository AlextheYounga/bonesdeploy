# apparmor role

Installs and validates per-project AppArmor confinement for per-site nginx.

## What it does

1. Installs AppArmor packages.
2. Ensures `apparmor` service is enabled and running.
3. Verifies kernel AppArmor is enabled via `/sys/module/apparmor/parameters/enabled`.
4. Deploys `/etc/apparmor.d/{{ apparmor_profile_name }}` from template.
5. Reloads profile with `apparmor_parser -r`.
6. Enforces profile with `aa-enforce`.
7. Verifies profile is loaded and in enforce mode with `aa-status`.
8. Limits per-site nginx network allowance to unix stream sockets by default.

## Expected profile name

Default profile name:

```text
bonesdeploy-{{ project_name }}-nginx
```

## Linux verification commands

```bash
systemctl is-active apparmor
cat /sys/module/apparmor/parameters/enabled
aa-status | awk '/profiles are in enforce mode:/{flag=1; next} /profiles are in complain mode:/{flag=0} flag' | grep 'bonesdeploy-<project>-nginx'
systemctl cat <project>-nginx.service | grep -E 'AppArmorProfile|After=|Requires='
```

Expected:

- `apparmor` is `active`
- enabled parameter is `y/yes/1`
- profile is in enforce mode
- per-site service unit is bound to AppArmor profile and ordered after/requires `apparmor.service`
