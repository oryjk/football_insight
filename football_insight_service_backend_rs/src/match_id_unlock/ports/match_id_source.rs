use async_trait::async_trait;

/// 比赛号是否存在于余票看板的数据源。
/// 看板的 match_id 来自外部 ticket-monitor 服务，不是 f_i_matches 表的业务键，
/// 存在性校验必须回到同一数据源，否则会出现"看板有比赛但解锁接口 404"。
#[async_trait]
pub trait MatchIdSource: Send + Sync {
    async fn known_match_id(&self, match_id: i64) -> anyhow::Result<bool>;
}
