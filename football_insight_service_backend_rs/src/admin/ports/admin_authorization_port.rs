use crate::admin::domain::admin_auth::AdminPermission;

pub trait AdminAuthorizationPort: Send + Sync {
    fn is_allowed(&self, role: &str, permission: AdminPermission) -> bool;
}
