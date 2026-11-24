use bonesdeploy_core::names;

use super::RuntimePlan;
use crate::ProvisioningContext;

pub(super) fn plan(_context: &ProvisioningContext) -> RuntimePlan {
    RuntimePlan::Static {
        root: names::ANGULAR_BROWSER_DIR,
        nginx_template: "frameworks/angular/nginx/static-site-nginx.conf.j2",
        placeholder_template: names::INDEX_TEMPLATE,
    }
}
