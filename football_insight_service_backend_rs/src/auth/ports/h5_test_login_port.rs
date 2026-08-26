use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::domain::user::AuthUser;

/// H5 测试登录的用户查询端口：按 UUID 取活跃账号。
/// 独立小端口而不扩展 AuthRepository，避免波及其全部测试实现。
#[async_trait]
pub trait H5TestLoginUserPort: Send + Sync {
    async fn find_active_user_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<AuthUser>>;
}
