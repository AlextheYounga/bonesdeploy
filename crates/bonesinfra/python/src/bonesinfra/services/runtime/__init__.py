import sys

from bonesinfra.services.runtime.mariadb import SERVICE as _mariadb
from bonesinfra.services.runtime.mongodb import SERVICE as _mongodb
from bonesinfra.services.runtime.mysql import SERVICE as _mysql
from bonesinfra.services.runtime.postgres import SERVICE as _postgres
from bonesinfra.services.runtime.redis import SERVICE as _redis
from bonesinfra.services.runtime.valkey import SERVICE as _valkey

SERVICES = {
    "mariadb": _mariadb,
    "mongodb": _mongodb,
    "mysql": _mysql,
    "postgres": _postgres,
    "redis": _redis,
    "valkey": _valkey,
}


def get_service(name):
    svc = SERVICES.get(name)
    if svc is None:
        print(f"Unknown service: {name}. Available: {', '.join(sorted(SERVICES))}", file=sys.stderr)
        sys.exit(1)
    return svc
