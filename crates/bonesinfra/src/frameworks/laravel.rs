use bonesdeploy_core::state;

use anyhow::Result;

use super::{RuntimePlan, numeric_version, string_value};
use crate::ProvisioningContext;

pub(super) fn plan(context: &ProvisioningContext) -> Result<RuntimePlan> {
    numeric_version(&string_value(context, state::PHP_VERSION, "8.5")?, 2, state::PHP_VERSION)?;
    Ok(RuntimePlan::Laravel)
}
