/// Managed-key vocabulary for the root `.env` grammar.
pub(crate) const MANAGED_PREFIX: &str = "BONES_";

pub(super) const BACKUP_SCHEDULE: &str = "BACKUP_SCHEDULE";
pub(super) const BACKUP_RETENTION_DAYS: &str = "BACKUP_RETENTION_DAYS";
pub(super) const BORG_PASSPHRASE: &str = "BORG_PASSPHRASE";
pub(super) use crate::config::variables::{PROJECT_NAME, WEB_ROOT};
pub(super) const SSH_USER: &str = "SSH_USER";
pub(super) const HOST: &str = "HOST";
pub(super) const PORT: &str = "PORT";
pub(super) const BRANCH: &str = "BRANCH";
pub(super) const DOMAIN: &str = "DOMAIN";
pub(super) const EMAIL: &str = "EMAIL";
pub(super) const SSL_ENABLED: &str = "SSL_ENABLED";
pub(super) const TEMPLATE: &str = "TEMPLATE";
pub(super) const RUNTIME_BACKEND: &str = "RUNTIME_BACKEND";
pub(super) const NODE_VERSION: &str = "NODE_VERSION";
pub(super) const COMPOSE_PORT: &str = "COMPOSE_PORT";
pub(super) const COMPOSE_WAIT_TIMEOUT: &str = "COMPOSE_WAIT_TIMEOUT";
pub(super) const PHP_VERSION: &str = "PHP_VERSION";
pub(super) const PYTHON_VERSION: &str = "PYTHON_VERSION";
pub(super) const RUBY_VERSION: &str = "RUBY_VERSION";
pub(super) const IS_STATIC: &str = "IS_STATIC";
pub(super) const INTERNAL_PORT: &str = "INTERNAL_PORT";

pub(super) const FRAMEWORK_KEYS: &[&str] = &[PHP_VERSION, PYTHON_VERSION, RUBY_VERSION, IS_STATIC, INTERNAL_PORT];

pub(super) const MANAGED: &[&str] = &[
    PROJECT_NAME,
    SSH_USER,
    HOST,
    PORT,
    BRANCH,
    DOMAIN,
    EMAIL,
    SSL_ENABLED,
    TEMPLATE,
    RUNTIME_BACKEND,
    WEB_ROOT,
    NODE_VERSION,
    COMPOSE_PORT,
    COMPOSE_WAIT_TIMEOUT,
    BACKUP_SCHEDULE,
    BACKUP_RETENTION_DAYS,
    BORG_PASSPHRASE,
    PHP_VERSION,
    PYTHON_VERSION,
    RUBY_VERSION,
    IS_STATIC,
    INTERNAL_PORT,
];
