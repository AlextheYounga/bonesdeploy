use super::RuntimePlan;
use crate::ProvisioningContext;

pub(super) fn plan(_context: &ProvisioningContext) -> RuntimePlan {
    RuntimePlan::Static {
        root: "dist",
        nginx_template: "frameworks/vue/nginx/static-site-nginx.conf.j2",
        placeholder_template: "frameworks/vue/nginx/index.html.j2",
    }
}
