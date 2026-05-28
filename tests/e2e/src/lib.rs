#![forbid(unsafe_code)]

#[cfg(test)]
mod e2e_cmd_deploy;

#[cfg(test)]
mod e2e_cmd_doctor;

#[cfg(test)]
mod e2e_cmd_init;

#[cfg(test)]
mod e2e_cmd_push;

#[cfg(test)]
mod e2e_cmd_pull;

#[cfg(test)]
mod e2e_cmd_rollback;

#[cfg(test)]
mod e2e_cmd_remote_ssl;

#[cfg(test)]
mod e2e_smoke;

#[cfg(test)]
mod support;
