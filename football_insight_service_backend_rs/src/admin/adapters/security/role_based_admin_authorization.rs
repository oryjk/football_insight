use crate::admin::{
    domain::admin_auth::AdminPermission, ports::admin_authorization_port::AdminAuthorizationPort,
};

#[derive(Clone, Default)]
pub struct RoleBasedAdminAuthorization;

impl AdminAuthorizationPort for RoleBasedAdminAuthorization {
    fn is_allowed(&self, role: &str, permission: AdminPermission) -> bool {
        match role {
            "owner" => true,
            "admin" => matches!(
                permission,
                AdminPermission::ManageUsers
                    | AdminPermission::ManageProducts
                    | AdminPermission::ViewAuditLogs
            ),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_management_is_granted_only_to_owner_and_admin() {
        let authorization = RoleBasedAdminAuthorization;
        assert!(authorization.is_allowed("owner", AdminPermission::ManageProducts));
        assert!(authorization.is_allowed("admin", AdminPermission::ManageProducts));
        assert!(!authorization.is_allowed("viewer", AdminPermission::ManageProducts));
    }

    #[test]
    fn unknown_roles_have_no_permissions() {
        let authorization = RoleBasedAdminAuthorization;
        assert!(!authorization.is_allowed("viewer", AdminPermission::ManageUsers));
    }
}
