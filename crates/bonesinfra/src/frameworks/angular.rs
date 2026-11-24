use bonesdeploy_core::state;

use super::RuntimePlan;
use crate::ProvisioningContext;

pub(super) fn plan(_context: &ProvisioningContext) -> RuntimePlan {
    RuntimePlan::Static {
        root: state::ANGULAR_BROWSER_DIR,
        nginx_template: "frameworks/angular/nginx/static-site-nginx.conf.j2",
        placeholder_template: state::INDEX_TEMPLATE,
    }
}
