pub(crate) mod deploy;
pub(crate) mod doctor;
pub(crate) mod init;
pub(crate) mod init_config;
pub(crate) mod manage;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod remote_runtime;
pub(crate) mod remote_setup;
pub(crate) mod remote_ssl;
pub(crate) mod rollback;
pub(crate) mod update;
pub(crate) mod update_release;
pub(crate) mod version;

// Compatibility re-exports — CLI types now live in crate::cli
pub use crate::cli::args::Cli;
pub use crate::cli::dispatch::run;
