//! Configuration module: canonical model, local environment grammar, build
//! environment parsing, validation, variable vocabulary, and typed transports
//! shared by the BonesDeploy, BonesInfra, and BonesRemote boundaries.

mod atomic_write;
mod backup;
mod local_env;
mod model;
mod transport;

pub mod build_env;
pub mod validation;
pub mod variables;

pub use backup::{
    BACKUP_RETENTION_DAYS_DEFAULT, BACKUP_SCHEDULE_DEFAULT, Backup, ensure_backup_passphrase, validate_backup,
    validate_cron_expression,
};

pub use local_env::{
    LoadedLocal, ParsedDotEnv, load, load_local, parse_dotenv, production_application_keys, validate_dotenv,
    write_local_environment,
};
pub use model::{
    App, BUILD_TIMEOUT_SECONDS_DEFAULT, Bones, Build, DATABASE_SERVICES, LARAVEL_TEMPLATE, PROJECT_SETUP_ERROR,
    RUNTIME_PYTHON_VERSION, RUNTIME_RUBY_VERSION, Runtime, RuntimeBackend, Services, apply_derived_defaults,
    build_group_for, build_timeout_seconds, build_user_for, default_deploy_user, default_node_version,
    default_repo_path_for, parse_port, runtime_group_for, runtime_user_for, validate_database_services, validate_host,
    validate_runtime,
};
pub use transport::{
    BackupFields, KeyValueCredentials, ProvisioningRequest, RemoteDeploymentConfig, ServerConnection,
    ServiceCredentials, ServicesRequest, SiteFields,
};
pub use validation::{is_numbered_shell_script, validate_project_name, validate_site_name};
