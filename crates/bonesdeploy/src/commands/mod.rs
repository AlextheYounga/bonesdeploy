pub(crate) mod config;

pub mod deploy;
pub mod doctor;
pub mod init;
pub mod manifest;
pub mod push_state;
pub mod releases;
pub mod remote;
pub mod rollback;
pub mod secrets;
pub mod setup;
pub mod skill;
pub mod status;
pub mod update;
pub mod version;

pub use crate::cli::args::Cli;
pub use crate::cli::dispatch::run;
