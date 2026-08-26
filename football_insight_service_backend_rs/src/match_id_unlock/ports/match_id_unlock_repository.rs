use async_trait::async_trait;
use uuid::Uuid;

/// 结算写入由 payment 模块的 PaymentSettlementPort 在同一事务内完成，
/// 这里只承担读取侧：比赛存在性与已解锁判定。
#[async_trait]
pub trait MatchIdUnlockRepository: Send + Sync {
    async fn match_exists(&self, match_id: i64) -> anyhow::Result<bool>;

    async fn find_unlock(&self, user_id: Uuid, match_id: i64) -> anyhow::Result<bool>;
}
