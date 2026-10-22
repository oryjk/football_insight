use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};

use crate::ticket_watch::domain::best_watch_windows::RestockProfile;

/// 离线批处理写入的统计快照；football 只读。
#[async_trait]
pub trait RestockProfileStore: Send + Sync {
    /// 按候选 id（football 比赛 id、监控侧 id）依次查找，要求统计起点与 `since` 一致、
    /// schema 与口径匹配；找不到返回 `None`，由调用方决定是否回退。
    async fn load(
        &self,
        lookup_ids: &[i64],
        since: DateTime<FixedOffset>,
    ) -> anyhow::Result<Option<RestockProfile>>;
}
