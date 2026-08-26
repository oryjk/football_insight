use async_trait::async_trait;
use uuid::Uuid;

/// 解锁记录读取。写入侧由 payment 的 PaymentSettlementPort 在结算事务里完成。
#[async_trait]
pub trait MatchIdUnlockRepository: Send + Sync {
    async fn find_unlock(&self, user_id: Uuid, match_id: i64) -> anyhow::Result<bool>;
}
