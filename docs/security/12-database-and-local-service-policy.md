# BonesDeploy Database and Local Service Policy

## Purpose

Databases and local daemons should be isolated per project and should not be exposed more broadly than required.

## SQLite

If SQLite is used, database files should be project-specific and protected:

```text
/srv/sites/<project>/shared/database.sqlite
```

Permissions:

```text
0600 <service-user>:<service-group>
```

or, if deploy or backup group access is needed:

```text
0640 <service-user>:<restricted-group>
```

Other service users should not be able to read SQLite files.

## Network Databases

Postgres, MySQL, and MariaDB should bind only to localhost or private interfaces unless external access is intentionally required.
Each project should have a distinct database user with least privilege.

Bad:

```text
all projects use root DB user
all projects share the same database credentials
database listens publicly
```

Good:

```text
app1 has app1_db_user
app2 has app2_db_user
database listens on localhost or a private network
```

## Redis and Memcached

Redis and Memcached should not be publicly exposed.
If shared, logical separation should be used where possible, but stronger isolation comes from separate instances or separate credentials and namespaces where supported.

## Findings

The agent or operator should flag:

- SQLite files readable by unrelated service users
- databases bound publicly
- multiple projects sharing one database user
- Redis or Memcached exposed on a public interface
