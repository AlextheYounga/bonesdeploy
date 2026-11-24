use bonesdeploy_core::names;

use anyhow::Result;

use super::{RuntimePlan, numeric_version, string_value};
use crate::ProvisioningContext;

pub(super) fn plan(context: &ProvisioningContext) -> Result<RuntimePlan> {
    numeric_version(&string_value(context, names::PHP_VERSION, "8.5")?, 2, names::PHP_VERSION)?;
    Ok(RuntimePlan::Laravel)
}
