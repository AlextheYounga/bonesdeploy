use super::RuntimePlan;
use crate::ProvisioningContext;

pub(super) fn plan(_context: &ProvisioningContext) -> RuntimePlan {
    RuntimePlan::Custom
}
