"""Canonical keys for the flat project .env contract."""

BRANCH = "BRANCH"
DOMAIN = "DOMAIN"
EMAIL = "EMAIL"
HOST = "HOST"
PORT = "PORT"
PREVIEW_DOMAIN = "PREVIEW_DOMAIN"
PROJECT_NAME = "PROJECT_NAME"
RUNTIME_BACKEND = "RUNTIME_BACKEND"
SERVICES = "SERVICES"
SSH_USER = "SSH_USER"
SSL_ENABLED = "SSL_ENABLED"
TEMPLATE = "TEMPLATE"
WEB_ROOT = "WEB_ROOT"

SUPPORTED_DATABASE_SERVICES = frozenset({"postgres", "mariadb", "mysql", "mongodb", "valkey", "redis"})

APP_KEYS = frozenset(
    {
        BRANCH,
        DOMAIN,
        EMAIL,
        HOST,
        PORT,
        PREVIEW_DOMAIN,
        PROJECT_NAME,
        RUNTIME_BACKEND,
        SERVICES,
        SSH_USER,
        SSL_ENABLED,
        TEMPLATE,
        WEB_ROOT,
    }
)
