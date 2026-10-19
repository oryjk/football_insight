# 战术板 阶段二：阵容编辑与保存 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 用户在小程序里选基础阵型、从全队名单（含前队员）选人上场、拖动圆点微调站位、保存"我的最佳阵容"（每人每队一套），并从洞察页和首页进入。

**Architecture:** 后端 `lineup` 模块新增 `f_i_lineups` 表、`GET/PUT /api/v1/lineups/me/{team_id}`（登录鉴权，校验在 application 层）。小程序新分包 `pages/lineup`：阵型数据与所有判定逻辑是纯函数（先写失败测试），数据层 / 编辑状态两个 composable，展示组件纯 props/emit；拖动用原生 `movable-area` / `movable-view`，松手时统一按纯函数判定"点击 / 互换 / 移动"。

**Tech Stack:** Rust + Axum + SQLx + tokio test；uni-app + Vue 3 + TypeScript + Bun。

**Spec:** `docs/superpowers/specs/2026-10-19-lineup-board-design.md`（第 3.1–3.3、4、5 节，第 7 节阶段二）。阶段一已完成：`f_i_squad_players`、`GET /api/v1/lineups/squads/{team_id}`。

## Global Constraints

- 只做微信小程序；H5 端 `pages/lineup/index` 只显示"请在微信小程序中使用"（`#ifdef H5`）。
- 阵型代码只有这 7 个：`4-3-3`、`4-4-2`、`4-2-3-1`、`3-5-2`、`3-4-3`、`5-3-2`、`4-1-4-1`。
- `slots` 正好 11 项，`slot` 为 0–10 且不重复；`x`、`y` 为 0–1 的有限小数（球场竖向，`y` 越大越靠近己方球门，门将在 `(0.5, 0.9)`）；`player_id` 可为空；非空的必须属于该队 `f_i_squad_players`（现役或前队员）且不重复。
- 每人每队一套：`(user_id, team_id)` 唯一，保存即覆盖；`share_code` 首次保存时生成（8 位 base62）、之后不变。
- 保存与读取"我的阵容"需要登录；名单读取不需要。登录失败 401 `请先登录`。
- 拖动判定数值：位移小于 10rpx 视为点击；落点与另一名球员中心距离小于圆点直径的 0.6 倍则互换；坐标钳制在 0.04–0.96。
- 小程序：颜色、间距、字号、圆角、阴影、动效引用 token；`src/pages/lineup` 下无渐变、无散落 hex（草坪与线条色先加 primitive）、无数字字重；单位用 rpx（只有 `movable-view` 的 x/y、`createSelectorQuery` 等只接受 px 的接口用 px，并在代码注释里写明）。
- 组件层不 import `api/**`、store、session；页面之间不互相 import；不往 `check-import-boundaries.mjs` 加 ALLOWLIST；运行时组件直接从 `.vue` 文件导入。
- 不手改 `manifest.json` 的 `versionName`；**不上传小程序**；生产后端部署与迁移执行在 Task 10 询问负责人后进行。
- 后端代码改动后，本地验证前先重启本地后端（`./deploy/local-test/up.sh`）；汇报区分 本地后端已重启 / 生产未部署 / 生产已部署。**不要对整个后端运行 `cargo fmt`**（会重排 60 多个无关文件）；只运行 `rustfmt --edition 2024 <本次新增或修改的 lineup 文件>`，或用 `cargo fmt -- --check` 查看后只处理 lineup 文件。
- 本地后端连生产库：测试写入一律走一次性 Docker PostgreSQL（`docker run --rm -d --name lineup-test-pg -p 127.0.0.1:55498:5432 -e POSTGRES_PASSWORD=x -e POSTGRES_DB=lineup_test postgres:16`），不用生产数据做写入测试；测试完 `docker stop lineup-test-pg`。
- 提交：`feat(backend): ...`、`feat(mini): ...`、`docs: ...`，subject 英文，末尾 `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`；提交并推送 `main`，提交前 `git pull --rebase`。
- 验证命令：后端 `cd football_insight_service_backend_rs && cargo test`；小程序 `cd football_insight_mini && bun test && bun run type-check && bun run check:boundaries && bun run verify:mp-components && bun run build:mp-weixin`。

## Review Focus

1. **并发首次保存同一用户同一队两次**：唯一约束 `(user_id, team_id)` 冲突不得返回 500，也不得生成两个 `share_code`；`share_code` 冲突（极小概率）要重试而不是失败。→ Task 3 postgres 测试 `concurrent_first_saves_keep_one_row_and_one_share_code`、`share_code_collision_retries`。
2. **保存他队球员 / 重复球员 / 前队员**：他队球员与重复球员 400；前队员（`active=false`）允许。→ Task 2 application 测试与 Task 3 postgres 测试。
3. **名单里球员后来离队（变前队员）后读取旧阵容**：读取不失败，球员照常返回；编辑页仍显示该球员并带"前"标。→ Task 4 `lineupViewModel` 测试 `former_player_in_saved_lineup_stays_visible`。
4. **拖动松手落在球场外或贴边**：坐标钳制在 0.04–0.96；点击与拖动的 10rpx 判定在 `movable-view` 触发 `tap` 前后都不重复弹出选人弹层。→ Task 5 `resolveDragRelease` 测试 + Task 8 手测清单。
5. **未登录先排再保存**：草稿按球队分键存本地；登录回来恢复草稿后自动保存一次且不重复保存；草稿里含已不存在的 `player_id` 时显示为空位。→ Task 6 `lineupDraft` 测试。
6. **切换阵型时已排球员的重新分配**：球员数多于新阵型同类别位置时不丢人、不重复上场。→ Task 5 `reassignPlayersToFormation` 测试。

---

## 文件结构

| 文件 | 动作 | 职责 |
|---|---|---|
| `football_insight_service_backend_rs/migrations/20261019130000_add_lineups.sql` | 新增 | 建表 |
| `football_insight_service_backend_rs/src/lineup/domain.rs` | 改 | 追加 `Formation`、`LineupSlot`、`Lineup`、`LineupSaveError`、`validate_slots` |
| `football_insight_service_backend_rs/src/lineup/ports.rs` | 改 | 追加 `LineupRepository` |
| `football_insight_service_backend_rs/src/lineup/application.rs` + `application/my_lineup.rs` | 改 + 新增 | `MyLineupService`：读取与保存 |
| `football_insight_service_backend_rs/src/lineup/adapters/persistence.rs` | 改 | `PostgresLineupRepository` |
| `football_insight_service_backend_rs/src/lineup/adapters/web.rs` + `adapters/my_web.rs` | 改 + 新增 | 我的阵容路由（含鉴权） |
| `football_insight_service_backend_rs/src/lineup/bootstrap.rs`、`src/app.rs` | 改 | 装配（需要 token_port） |
| `football_insight_service_backend_rs/tests/lineup_postgres.rs` | 新增 | 一次性库测试 |
| `football_insight_mini/src/types/lineup.ts`、`src/api/lineup.ts` | 新增 | 类型与接口 |
| `football_insight_mini/src/pages/lineup/formations.ts` | 新增 | 7 个阵型定义 |
| `football_insight_mini/src/pages/lineup/helpers.ts` | 新增 | 纯函数 |
| `football_insight_mini/src/pages/lineup/lineupViewModel.ts` | 新增 | 名单与阵容合成展示数据 |
| `football_insight_mini/src/pages/lineup/lineupDraft.ts` | 新增 | 本地草稿读写 |
| `football_insight_mini/src/pages/lineup/useLineupPage.ts`、`useLineupEditor.ts` | 新增 | 数据层与编辑状态 |
| `football_insight_mini/src/pages/lineup/components/*.vue` | 新增 | 展示组件 |
| `football_insight_mini/src/pages/lineup/index.vue` | 新增 | 页面 |
| `football_insight_mini/src/pages.json` | 改 | 分包 `pages/lineup` |
| `football_insight_mini/src/pages/insights/index.vue`、`pages/home/…` | 改 | 入口 |
| `football_insight_mini/src/styles/fi-tokens.css`、`scripts/lineup-style-audit.test.mjs` | 改 + 新增 | token 与样式审计 |
| `CHANGELOG.md`、`FEATURES.md` | 改 | 文档 |

---

### Task 1: 迁移 `f_i_lineups`

**Files:** Create: `football_insight_service_backend_rs/migrations/20261019130000_add_lineups.sql`

**Interfaces:** Produces 表 `f_i_lineups(id, user_id, team_id, formation, slots, share_code, created_at, updated_at)`，唯一 `(user_id, team_id)`，唯一 `share_code`。

- [ ] **Step 1: 写迁移**

```sql
-- 用户的"我的最佳阵容"：每人每队一套，保存即覆盖。球员只存 player_id，读取时再关联 f_i_squad_players。
CREATE TABLE IF NOT EXISTS f_i_lineups (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    team_id BIGINT NOT NULL,
    formation TEXT NOT NULL,
    slots JSONB NOT NULL,
    share_code TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_f_i_lineups_user_team UNIQUE (user_id, team_id),
    CONSTRAINT uq_f_i_lineups_share_code UNIQUE (share_code)
);
```

- [ ] **Step 2: 编译确认**：`cd football_insight_service_backend_rs && cargo build 2>&1 | tail -2`，Expected: Finished。

- [ ] **Step 3: 提交**

```bash
git pull --rebase
git add football_insight_service_backend_rs/migrations/20261019130000_add_lineups.sql
git commit -m "feat(backend): add lineups table migration

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 2: 后端 —— 领域校验、端口与 application

**Files:**
- Modify: `football_insight_service_backend_rs/src/lineup/domain.rs`、`ports.rs`、`application.rs`
- Create: `football_insight_service_backend_rs/src/lineup/application/my_lineup.rs`

**Interfaces:**
- Produces:

```rust
// domain.rs 追加
pub const FORMATIONS: [&str; 7] = ["4-3-3", "4-4-2", "4-2-3-1", "3-5-2", "3-4-3", "5-3-2", "4-1-4-1"];
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineupSlot { pub slot: u8, pub player_id: Option<i64>, pub x: f64, pub y: f64 }
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Lineup { pub formation: String, pub slots: Vec<LineupSlot>, pub share_code: String,
                    pub updated_at: chrono::DateTime<chrono::Utc> }
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupSaveError {
    #[error("阵型无效")] InvalidFormation,
    #[error("站位数据无效")] InvalidSlots,
    #[error("球员不属于这支球队")] PlayerNotInSquad,
    #[error("同一球员不能重复上场")] DuplicatePlayer,
    #[error("球队不存在")] TeamNotFound,
    #[error("阵容暂不可用")] Unavailable,
}
pub fn validate_lineup_input(formation: &str, slots: &[LineupSlot]) -> Result<(), LineupSaveError>
// ports.rs 追加
pub enum StoreError { ShareCodeTaken, Unavailable }
#[async_trait] pub trait LineupRepository: Send + Sync {
    async fn find(&self, user_id: Uuid, team_id: i64) -> Result<Option<Lineup>, StoreError>;
    async fn squad_members(&self, team_id: i64, player_ids: &[i64]) -> Result<Vec<i64>, StoreError>; // 返回属于该队的 id
    async fn upsert(&self, user_id: Uuid, team_id: i64, formation: &str, slots: &[LineupSlot], share_code: &str)
        -> Result<Lineup, StoreError>;
}
// application/my_lineup.rs
pub struct MyLineupService { .. }
impl MyLineupService {
    pub fn new(squads: Arc<dyn SquadRepository>, lineups: Arc<dyn LineupRepository>) -> Self;
    pub async fn get(&self, user_id: Uuid, team_id: i64) -> Result<Option<Lineup>, LineupSaveError>;
    pub async fn save(&self, user_id: Uuid, team_id: i64, formation: &str, slots: Vec<LineupSlot>) -> Result<Lineup, LineupSaveError>;
}
pub fn generate_share_code() -> String   // 8 位 base62
```

- [ ] **Step 1: 写失败测试（domain 校验）**，追加到 `domain.rs` 末尾：

```rust
#[cfg(test)]
mod lineup_validation_tests {
    use super::*;

    fn slots(n: u8) -> Vec<LineupSlot> {
        (0..n).map(|slot| LineupSlot { slot, player_id: None, x: 0.5, y: 0.5 }).collect()
    }

    #[test]
    fn accepts_eleven_unique_slots_for_every_formation() {
        for f in FORMATIONS {
            assert_eq!(validate_lineup_input(f, &slots(11)), Ok(()), "{f}");
        }
    }

    #[test]
    fn rejects_unknown_formation() {
        assert_eq!(validate_lineup_input("9-9-9", &slots(11)), Err(LineupSaveError::InvalidFormation));
    }

    #[test]
    fn rejects_wrong_slot_count_duplicate_or_out_of_range_slot_index() {
        assert_eq!(validate_lineup_input("4-3-3", &slots(10)), Err(LineupSaveError::InvalidSlots));
        assert_eq!(validate_lineup_input("4-3-3", &slots(12)), Err(LineupSaveError::InvalidSlots));
        let mut dup = slots(11);
        dup[3].slot = 2;
        assert_eq!(validate_lineup_input("4-3-3", &dup), Err(LineupSaveError::InvalidSlots));
        let mut high = slots(11);
        high[10].slot = 11;
        assert_eq!(validate_lineup_input("4-3-3", &high), Err(LineupSaveError::InvalidSlots));
    }

    #[test]
    fn rejects_coordinates_outside_unit_square_or_not_finite() {
        for (x, y) in [(-0.01, 0.5), (1.01, 0.5), (0.5, -0.01), (0.5, 1.01), (f64::NAN, 0.5), (0.5, f64::INFINITY)] {
            let mut s = slots(11);
            s[0].x = x;
            s[0].y = y;
            assert_eq!(validate_lineup_input("4-3-3", &s), Err(LineupSaveError::InvalidSlots), "{x},{y}");
        }
    }

    #[test]
    fn rejects_the_same_player_twice() {
        let mut s = slots(11);
        s[0].player_id = Some(7);
        s[5].player_id = Some(7);
        assert_eq!(validate_lineup_input("4-3-3", &s), Err(LineupSaveError::DuplicatePlayer));
    }
}
```

- [ ] **Step 2: 确认失败**：`cargo test lineup::domain 2>&1 | tail -5`，Expected: 编译失败（`FORMATIONS` 等未定义）。

- [ ] **Step 3: 实现 domain**（追加到 `domain.rs` 的类型之后、测试之前）

```rust
use std::collections::HashSet;

pub const FORMATIONS: [&str; 7] = [
    "4-3-3", "4-4-2", "4-2-3-1", "3-5-2", "3-4-3", "5-3-2", "4-1-4-1",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineupSlot {
    pub slot: u8,
    pub player_id: Option<i64>,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Lineup {
    pub formation: String,
    pub slots: Vec<LineupSlot>,
    pub share_code: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupSaveError {
    #[error("阵型无效")]
    InvalidFormation,
    #[error("站位数据无效")]
    InvalidSlots,
    #[error("球员不属于这支球队")]
    PlayerNotInSquad,
    #[error("同一球员不能重复上场")]
    DuplicatePlayer,
    #[error("球队不存在")]
    TeamNotFound,
    #[error("阵容暂不可用")]
    Unavailable,
}

/// 与球队名单无关的纯校验：阵型、11 个不重复的位置序号、坐标范围、球员不重复。
pub fn validate_lineup_input(formation: &str, slots: &[LineupSlot]) -> Result<(), LineupSaveError> {
    if !FORMATIONS.contains(&formation) {
        return Err(LineupSaveError::InvalidFormation);
    }
    if slots.len() != 11 {
        return Err(LineupSaveError::InvalidSlots);
    }
    let mut seen_slots = HashSet::new();
    for slot in slots {
        let in_unit = |v: f64| v.is_finite() && (0.0..=1.0).contains(&v);
        if slot.slot > 10 || !seen_slots.insert(slot.slot) || !in_unit(slot.x) || !in_unit(slot.y) {
            return Err(LineupSaveError::InvalidSlots);
        }
    }
    let mut seen_players = HashSet::new();
    for id in slots.iter().filter_map(|s| s.player_id) {
        if !seen_players.insert(id) {
            return Err(LineupSaveError::DuplicatePlayer);
        }
    }
    Ok(())
}
```

Run: `cargo test lineup::domain 2>&1 | tail -4`，Expected: 5 passed。

- [ ] **Step 4: ports 追加**

```rust
use super::domain::{Lineup, LineupSlot};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    ShareCodeTaken,
    Unavailable,
}

#[async_trait]
pub trait LineupRepository: Send + Sync {
    async fn find(&self, user_id: Uuid, team_id: i64) -> Result<Option<Lineup>, StoreError>;
    /// 返回 `player_ids` 里属于该队名单（现役或前队员）的那些 id。
    async fn squad_members(&self, team_id: i64, player_ids: &[i64]) -> Result<Vec<i64>, StoreError>;
    /// 首次保存插入并写入 `share_code`；已存在则覆盖 formation 与 slots，`share_code` 保持不变。
    async fn upsert(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: &[LineupSlot],
        share_code: &str,
    ) -> Result<Lineup, StoreError>;
}
```

（合并进现有 `use` 行，`async_trait` 已导入。）`application.rs` 顶部加 `pub mod my_lineup;`。

- [ ] **Step 5: 写失败测试（application）**，新建 `src/lineup/application/my_lineup.rs`：

```rust
use crate::lineup::{
    domain::{Lineup, LineupSaveError, LineupSlot, validate_lineup_input},
    ports::{LineupRepository, SquadRepository, StoreError},
};
use std::sync::Arc;
use uuid::Uuid;

pub struct MyLineupService {
    squads: Arc<dyn SquadRepository>,
    lineups: Arc<dyn LineupRepository>,
}

pub fn generate_share_code() -> String {
    todo!()
}

impl MyLineupService {
    pub fn new(squads: Arc<dyn SquadRepository>, lineups: Arc<dyn LineupRepository>) -> Self {
        Self { squads, lineups }
    }

    pub async fn get(&self, _user_id: Uuid, _team_id: i64) -> Result<Option<Lineup>, LineupSaveError> {
        let _ = (&self.squads, &self.lineups);
        todo!()
    }

    pub async fn save(
        &self,
        _user_id: Uuid,
        _team_id: i64,
        _formation: &str,
        _slots: Vec<LineupSlot>,
    ) -> Result<Lineup, LineupSaveError> {
        let _ = validate_lineup_input;
        let _ = StoreError::Unavailable;
        todo!()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::lineup::application::tests::FakeSquads;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    pub(crate) struct FakeLineups {
        pub members: Vec<i64>,
        pub stored: Mutex<Option<Lineup>>,
        pub collisions: Mutex<u32>,
        pub upserts: Mutex<Vec<String>>,
        pub fail: bool,
    }

    #[async_trait]
    impl LineupRepository for FakeLineups {
        async fn find(&self, _u: Uuid, _t: i64) -> Result<Option<Lineup>, StoreError> {
            if self.fail {
                return Err(StoreError::Unavailable);
            }
            Ok(self.stored.lock().unwrap().clone())
        }
        async fn squad_members(&self, _t: i64, ids: &[i64]) -> Result<Vec<i64>, StoreError> {
            Ok(ids.iter().copied().filter(|id| self.members.contains(id)).collect())
        }
        async fn upsert(&self, _u: Uuid, _t: i64, formation: &str, slots: &[LineupSlot], code: &str) -> Result<Lineup, StoreError> {
            let mut left = self.collisions.lock().unwrap();
            if *left > 0 {
                *left -= 1;
                return Err(StoreError::ShareCodeTaken);
            }
            self.upserts.lock().unwrap().push(code.to_string());
            let mut stored = self.stored.lock().unwrap();
            // 与真实 SQL 一致：已存在时 share_code 不变
            let share_code = stored.as_ref().map(|l| l.share_code.clone()).unwrap_or_else(|| code.to_string());
            let lineup = Lineup { formation: formation.into(), slots: slots.to_vec(), share_code, updated_at: chrono::Utc::now() };
            *stored = Some(lineup.clone());
            Ok(lineup)
        }
    }

    fn slots(players: &[Option<i64>]) -> Vec<LineupSlot> {
        (0..11u8).map(|i| LineupSlot { slot: i, player_id: players.get(i as usize).copied().flatten(), x: 0.5, y: 0.5 }).collect()
    }

    fn service(members: Vec<i64>) -> (MyLineupService, Arc<FakeLineups>) {
        let lineups = Arc::new(FakeLineups { members, ..Default::default() });
        let svc = MyLineupService::new(Arc::new(FakeSquads { players: vec![], missing_table: false }), lineups.clone());
        (svc, lineups)
    }

    const TEAM: i64 = 77680; // FakeSquads 只认识这支队
    fn user() -> Uuid { Uuid::new_v4() }

    #[test]
    fn share_codes_are_eight_base62_characters() {
        for _ in 0..50 {
            let code = generate_share_code();
            assert_eq!(code.len(), 8);
            assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
        }
    }

    #[tokio::test]
    async fn saves_a_valid_lineup_with_current_and_former_players() {
        let (svc, repo) = service(vec![1, 2, 3]); // 前队员与现役都在名单表里，端口不区分
        let lineup = svc.save(user(), TEAM, "4-3-3", slots(&[Some(1), Some(2), Some(3)])).await.unwrap();
        assert_eq!(lineup.formation, "4-3-3");
        assert_eq!(lineup.share_code.len(), 8);
        assert_eq!(repo.upserts.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn rejects_players_that_are_not_in_the_squad() {
        let (svc, _) = service(vec![1]);
        let err = svc.save(user(), TEAM, "4-3-3", slots(&[Some(1), Some(99)])).await.unwrap_err();
        assert_eq!(err, LineupSaveError::PlayerNotInSquad);
    }

    #[tokio::test]
    async fn rejects_duplicates_bad_formation_and_unknown_team_without_touching_the_store() {
        let (svc, repo) = service(vec![1, 2]);
        assert_eq!(svc.save(user(), TEAM, "4-3-3", slots(&[Some(1), Some(1)])).await.unwrap_err(), LineupSaveError::DuplicatePlayer);
        assert_eq!(svc.save(user(), TEAM, "1-1-1", slots(&[])).await.unwrap_err(), LineupSaveError::InvalidFormation);
        assert_eq!(svc.save(user(), 5, "4-3-3", slots(&[])).await.unwrap_err(), LineupSaveError::TeamNotFound);
        assert!(repo.upserts.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn saving_again_overwrites_and_keeps_the_share_code() {
        let (svc, _) = service(vec![1, 2]);
        let u = user();
        let first = svc.save(u, TEAM, "4-3-3", slots(&[Some(1)])).await.unwrap();
        let second = svc.save(u, TEAM, "4-4-2", slots(&[Some(2)])).await.unwrap();
        assert_eq!(second.share_code, first.share_code);
        assert_eq!(second.formation, "4-4-2");
    }

    #[tokio::test]
    async fn retries_on_share_code_collision_then_succeeds() {
        let (svc, repo) = service(vec![]);
        *repo.collisions.lock().unwrap() = 2;
        assert!(svc.save(user(), TEAM, "4-3-3", slots(&[])).await.is_ok());
    }

    #[tokio::test]
    async fn gives_up_after_repeated_collisions_as_unavailable() {
        let (svc, repo) = service(vec![]);
        *repo.collisions.lock().unwrap() = 50;
        assert_eq!(svc.save(user(), TEAM, "4-3-3", slots(&[])).await.unwrap_err(), LineupSaveError::Unavailable);
    }

    #[tokio::test]
    async fn get_returns_none_then_the_saved_lineup_and_maps_store_failure() {
        let (svc, _) = service(vec![]);
        let u = user();
        assert!(svc.get(u, TEAM).await.unwrap().is_none());
        svc.save(u, TEAM, "3-5-2", slots(&[])).await.unwrap();
        assert_eq!(svc.get(u, TEAM).await.unwrap().unwrap().formation, "3-5-2");
        let broken = MyLineupService::new(
            Arc::new(FakeSquads { players: vec![], missing_table: false }),
            Arc::new(FakeLineups { fail: true, ..Default::default() }),
        );
        assert_eq!(broken.get(u, TEAM).await.unwrap_err(), LineupSaveError::Unavailable);
    }
}
```

Run: `cargo test lineup::application::my_lineup 2>&1 | tail -6`，Expected: FAIL（`todo!()`）。

- [ ] **Step 6: 实现**

```rust
pub fn generate_share_code() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::rng();
    (0..8).map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char).collect()
}

fn store_error(_: StoreError) -> LineupSaveError {
    LineupSaveError::Unavailable
}

const SHARE_CODE_ATTEMPTS: usize = 5;

impl MyLineupService {
    pub async fn get(&self, user_id: Uuid, team_id: i64) -> Result<Option<Lineup>, LineupSaveError> {
        if team_id <= 0 {
            return Err(LineupSaveError::TeamNotFound);
        }
        self.lineups.find(user_id, team_id).await.map_err(store_error)
    }

    pub async fn save(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: Vec<LineupSlot>,
    ) -> Result<Lineup, LineupSaveError> {
        validate_lineup_input(formation, &slots)?;
        self.squads
            .find_team(team_id)
            .await
            .map_err(|_| LineupSaveError::Unavailable)?
            .ok_or(LineupSaveError::TeamNotFound)?;
        let ids: Vec<i64> = slots.iter().filter_map(|s| s.player_id).collect();
        if !ids.is_empty() {
            let members = self.lineups.squad_members(team_id, &ids).await.map_err(store_error)?;
            if members.len() != ids.len() {
                return Err(LineupSaveError::PlayerNotInSquad);
            }
        }
        for _ in 0..SHARE_CODE_ATTEMPTS {
            match self
                .lineups
                .upsert(user_id, team_id, formation, &slots, &generate_share_code())
                .await
            {
                Ok(lineup) => return Ok(lineup),
                Err(StoreError::ShareCodeTaken) => continue,
                Err(e) => return Err(store_error(e)),
            }
        }
        Err(LineupSaveError::Unavailable)
    }
}
```

（删掉占位的 `todo!()` 版本；`validate_lineup_input` 在 `save` 里真正使用后，占位的 `let _ =` 行一并删除。）

Run: `cargo test lineup 2>&1 | grep -E "test result: .* [1-9][0-9]* passed|FAILED|^error"`，Expected: 全部 passed（阶段一的 7 个 + domain 5 个 + application 8 个）。

- [ ] **Step 7: 只格式化本次文件并提交**

```bash
cd football_insight_service_backend_rs
rustfmt --edition 2024 src/lineup/domain.rs src/lineup/ports.rs src/lineup/application.rs src/lineup/application/my_lineup.rs
cd .. && git status --short   # 只应出现 lineup 文件
git pull --rebase
git add football_insight_service_backend_rs/src/lineup
git commit -m "feat(backend): validate and save my lineup in the lineup application layer

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 3: 后端 —— Postgres 实现与一次性库测试

**Files:**
- Modify: `football_insight_service_backend_rs/src/lineup/adapters/persistence.rs`
- Create: `football_insight_service_backend_rs/tests/lineup_postgres.rs`

**Interfaces:**
- Consumes: Task 2 的 `LineupRepository`、`StoreError`、`Lineup`、`LineupSlot`。
- Produces: `PostgresLineupRepository::new(pool)`。

- [ ] **Step 1: 实现持久层**（追加到 `persistence.rs`）

```rust
use crate::lineup::{
    domain::{Lineup, LineupSlot},
    ports::{LineupRepository, StoreError},
};
use sqlx::types::Json;
use uuid::Uuid;

pub struct PostgresLineupRepository {
    pool: PgPool,
}

impl PostgresLineupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn store_error(e: sqlx::Error) -> StoreError {
    if let Some(db) = e.as_database_error() {
        if db.code().as_deref() == Some("23505") && db.constraint() == Some("uq_f_i_lineups_share_code") {
            return StoreError::ShareCodeTaken;
        }
    }
    tracing::warn!("lineup store unavailable");
    StoreError::Unavailable
}

type LineupRow = (String, Json<Vec<LineupSlot>>, String, chrono::DateTime<chrono::Utc>);

fn to_lineup((formation, Json(slots), share_code, updated_at): LineupRow) -> Lineup {
    Lineup { formation, slots, share_code, updated_at }
}

#[async_trait]
impl LineupRepository for PostgresLineupRepository {
    async fn find(&self, user_id: Uuid, team_id: i64) -> Result<Option<Lineup>, StoreError> {
        sqlx::query_as::<_, LineupRow>(
            "SELECT formation, slots, share_code, updated_at FROM f_i_lineups WHERE user_id = $1 AND team_id = $2",
        )
        .bind(user_id)
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(to_lineup))
        .map_err(store_error)
    }

    async fn squad_members(&self, team_id: i64, player_ids: &[i64]) -> Result<Vec<i64>, StoreError> {
        sqlx::query_scalar::<_, i64>(
            "SELECT player_id FROM f_i_squad_players WHERE team_id = $1 AND player_id = ANY($2)",
        )
        .bind(team_id)
        .bind(player_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(store_error)
    }

    async fn upsert(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: &[LineupSlot],
        share_code: &str,
    ) -> Result<Lineup, StoreError> {
        sqlx::query_as::<_, LineupRow>(
            "INSERT INTO f_i_lineups (id, user_id, team_id, formation, slots, share_code)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (user_id, team_id) DO UPDATE
                SET formation = EXCLUDED.formation, slots = EXCLUDED.slots, updated_at = NOW()
             RETURNING formation, slots, share_code, updated_at",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(team_id)
        .bind(formation)
        .bind(Json(slots))
        .bind(share_code)
        .fetch_one(&self.pool)
        .await
        .map(to_lineup)
        .map_err(store_error)
    }
}
```

若 `Json<&[LineupSlot]>` 绑定类型报错，改为 `Json(slots.to_vec())`；`sqlx::types::Json` 需要 `json` feature，若 `Cargo.toml` 的 sqlx features 缺 `json` 则加上并说明。

- [ ] **Step 2: 写一次性库测试**（参照 `tests/support_cancel_postgres.rs`：`#[ignore]`、`LINEUP_TEST_DATABASE_URL`、主机必须 localhost、库名以 `_test` 结尾、每个测试独立 schema）：

```rust
//! Real lineup SQL against an explicit disposable local test database.
use football_insight_service_backend_rs::lineup::{
    adapters::persistence::PostgresLineupRepository,
    domain::LineupSlot,
    ports::{LineupRepository, StoreError},
};
use sqlx::{PgPool, postgres::{PgConnectOptions, PgPoolOptions}};
use std::sync::Arc;
use uuid::Uuid;

struct Db {
    admin: PgPool,
    pool: PgPool,
    schema: String,
}

async fn setup() -> Db {
    let url = std::env::var("LINEUP_TEST_DATABASE_URL").expect("explicit test database required");
    let parsed = url::Url::parse(&url).unwrap();
    assert!(matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")));
    assert!(parsed.path().ends_with("_test"));
    let admin = PgPool::connect(&url).await.unwrap();
    let schema = format!("lineup_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await.unwrap();
    let options = url.parse::<PgConnectOptions>().unwrap().options([("search_path", schema.as_str())]);
    let pool = PgPoolOptions::new().max_connections(8).connect_with(options).await.unwrap();
    sqlx::raw_sql("CREATE TABLE f_i_users (id UUID PRIMARY KEY)").execute(&pool).await.unwrap();
    sqlx::raw_sql(
        "CREATE TABLE f_i_squad_players (team_id BIGINT NOT NULL, player_id BIGINT NOT NULL, active BOOLEAN NOT NULL DEFAULT FALSE, PRIMARY KEY (team_id, player_id))",
    ).execute(&pool).await.unwrap();
    let migration = include_str!("../migrations/20261019130000_add_lineups.sql");
    sqlx::raw_sql(migration).execute(&pool).await.unwrap();
    Db { admin, pool, schema }
}

async fn teardown(db: Db) {
    db.pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {} CASCADE", db.schema)).execute(&db.admin).await.unwrap();
    db.admin.close().await;
}

async fn add_user(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users (id) VALUES ($1)").bind(id).execute(pool).await.unwrap();
    id
}

fn slots(first: Option<i64>) -> Vec<LineupSlot> {
    (0..11u8).map(|i| LineupSlot { slot: i, player_id: if i == 0 { first } else { None }, x: 0.5, y: 0.5 }).collect()
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn saving_twice_overwrites_one_row_and_keeps_share_code() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let user = add_user(&db.pool).await;
    let first = repo.upsert(user, 1, "4-3-3", &slots(Some(5)), "AAAAAAAA").await.unwrap();
    let second = repo.upsert(user, 1, "4-4-2", &slots(Some(6)), "BBBBBBBB").await.unwrap();
    assert_eq!(first.share_code, "AAAAAAAA");
    assert_eq!(second.share_code, "AAAAAAAA");
    assert_eq!(second.formation, "4-4-2");
    assert_eq!(second.slots[0].player_id, Some(6));
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_lineups").fetch_one(&db.pool).await.unwrap();
    assert_eq!(rows, 1);
    assert_eq!(repo.find(user, 1).await.unwrap().unwrap().formation, "4-4-2");
    assert!(repo.find(user, 2).await.unwrap().is_none());
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn one_user_can_keep_one_lineup_per_team_and_users_do_not_collide() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let (a, b) = (add_user(&db.pool).await, add_user(&db.pool).await);
    repo.upsert(a, 1, "4-3-3", &slots(None), "CODE0001").await.unwrap();
    repo.upsert(a, 2, "4-4-2", &slots(None), "CODE0002").await.unwrap();
    repo.upsert(b, 1, "3-5-2", &slots(None), "CODE0003").await.unwrap();
    assert_eq!(repo.find(a, 1).await.unwrap().unwrap().formation, "4-3-3");
    assert_eq!(repo.find(a, 2).await.unwrap().unwrap().formation, "4-4-2");
    assert_eq!(repo.find(b, 1).await.unwrap().unwrap().formation, "3-5-2");
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn share_code_collision_is_reported_not_swallowed() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let (a, b) = (add_user(&db.pool).await, add_user(&db.pool).await);
    repo.upsert(a, 1, "4-3-3", &slots(None), "SAMECODE").await.unwrap();
    let err = repo.upsert(b, 1, "4-3-3", &slots(None), "SAMECODE").await.unwrap_err();
    assert_eq!(err, StoreError::ShareCodeTaken);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn concurrent_first_saves_keep_one_row_and_one_share_code() {
    let db = setup().await;
    let repo = Arc::new(PostgresLineupRepository::new(db.pool.clone()));
    let user = add_user(&db.pool).await;
    let mut tasks = Vec::new();
    for i in 0..6 {
        let repo = repo.clone();
        tasks.push(tokio::spawn(async move {
            repo.upsert(user, 1, "4-3-3", &slots(Some(i)), &format!("CONC000{i}")).await
        }));
    }
    let mut codes = std::collections::HashSet::new();
    for t in tasks {
        let lineup = t.await.unwrap().expect("no 500 on concurrent first save");
        codes.insert(lineup.share_code);
    }
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_lineups").fetch_one(&db.pool).await.unwrap();
    assert_eq!(rows, 1);
    assert_eq!(codes.len(), 1, "every save returns the single stored share_code");
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn squad_members_include_former_players_and_exclude_other_teams() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    sqlx::raw_sql(
        "INSERT INTO f_i_squad_players (team_id, player_id, active) VALUES (1, 10, TRUE), (1, 11, FALSE), (2, 12, TRUE)",
    ).execute(&db.pool).await.unwrap();
    let mut found = repo.squad_members(1, &[10, 11, 12, 99]).await.unwrap();
    found.sort();
    assert_eq!(found, vec![10, 11]);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires LINEUP_TEST_DATABASE_URL pointing to a disposable local *_test database"]
async fn deleting_the_user_cascades_to_the_lineup() {
    let db = setup().await;
    let repo = PostgresLineupRepository::new(db.pool.clone());
    let user = add_user(&db.pool).await;
    repo.upsert(user, 1, "4-3-3", &slots(None), "CASCADE1").await.unwrap();
    sqlx::query("DELETE FROM f_i_users WHERE id = $1").bind(user).execute(&db.pool).await.unwrap();
    assert!(repo.find(user, 1).await.unwrap().is_none());
    teardown(db).await;
}
```

- [ ] **Step 3: 起一次性库并运行（必须实际跑通）**

```bash
docker run --rm -d --name lineup-test-pg -p 127.0.0.1:55498:5432 -e POSTGRES_PASSWORD=x -e POSTGRES_DB=lineup_test postgres:16
for i in $(seq 1 30); do docker exec lineup-test-pg pg_isready -U postgres -d lineup_test >/dev/null 2>&1 && break; sleep 1; done; sleep 2
cd football_insight_service_backend_rs
LINEUP_TEST_DATABASE_URL=postgresql://postgres:x@127.0.0.1:55498/lineup_test cargo test --test lineup_postgres -- --ignored 2>&1 | tail -10
```

Expected: 6 passed。`concurrent_first_saves_keep_one_row_and_one_share_code` 若失败说明并发插入的冲突处理有问题，需用 systematic-debugging 找根因而不是改测试。

- [ ] **Step 4: 格式化、提交**

```bash
rustfmt --edition 2024 src/lineup/adapters/persistence.rs tests/lineup_postgres.rs
cd .. && git status --short
git pull --rebase
git add football_insight_service_backend_rs/src/lineup football_insight_service_backend_rs/tests/lineup_postgres.rs football_insight_service_backend_rs/Cargo.toml football_insight_service_backend_rs/Cargo.lock
git commit -m "feat(backend): persist my lineup with per-team upsert and stable share codes

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 4: 后端 —— 路由、鉴权与装配

**Files:**
- Create: `football_insight_service_backend_rs/src/lineup/adapters/my_web.rs`
- Modify: `src/lineup/adapters/mod.rs`、`src/lineup/bootstrap.rs`、`src/app.rs`

**Interfaces:**
- Consumes: `MyLineupService`、`TokenPort`（`crate::auth::ports::token_port::TokenPort`，`verify_token(&str) -> anyhow::Result<AuthTokenClaims>`，`claims.sub: Uuid`）。
- Produces: 路由 `GET /api/v1/lineups/me/{team_id}`、`PUT /api/v1/lineups/me/{team_id}`；`pub fn build_lineup_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router`（签名变化，app.rs 同步改）。
  - GET 响应：`{ "lineup": null }` 或 `{ "lineup": { "formation", "slots": [{slot, player_id, x, y}], "share_code", "updated_at" } }`。
  - PUT 请求：`{ "formation": "4-3-3", "slots": [{ "slot": 0, "player_id": 123 | null, "x": 0.5, "y": 0.9 }, …] }`，成功返回同 GET 的 `{ "lineup": {...} }`。
  - 错误映射：未登录 401 `请先登录`；`InvalidFormation|InvalidSlots|PlayerNotInSquad|DuplicatePlayer` 400；`TeamNotFound` 404；`Unavailable` 503；JSON 解析失败 axum 默认 422/400。

- [ ] **Step 1: 写失败测试**，`my_web.rs` 先放路由骨架（`routes` 返回空 `Router`）与下列测试（用 fake token port：`Bearer good` 通过并返回固定 user，其他拒绝）：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::lineup::application::{
        my_lineup::tests::FakeLineups,
        tests::FakeSquads,
    };
    use axum::{body::{Body, to_bytes}, http::Request};
    use tower::ServiceExt;

    // 实现者按 src/seat_swap 或 src/auth 里已有的 fake TokenPort 写法补 FakeTokenPort：
    // verify_token("good") -> Ok(claims{sub: USER}), 其他 -> Err；其余方法 unimplemented!()。
    // 先用 grep 找现有可复用的测试 TokenPort：`grep -rn "impl TokenPort for" src tests`。

    fn app(members: Vec<i64>) -> Router { /* my_routes(Arc::new(MyLineupWebState { service, token_port })) */ todo!() }

    async fn send(app: Router, method: &str, uri: &str, token: Option<&str>, body: Option<&str>) -> (u16, String) { /* ... */ todo!() }

    fn body_ok() -> String {
        let slots: Vec<String> = (0..11).map(|i| format!(r#"{{"slot":{i},"player_id":null,"x":0.5,"y":0.5}}"#)).collect();
        format!(r#"{{"formation":"4-3-3","slots":[{}]}}"#, slots.join(","))
    }

    #[tokio::test] async fn requires_login_for_get_and_put() { /* 无 token → 401 */ }
    #[tokio::test] async fn get_returns_null_before_saving_then_the_saved_lineup() { /* PUT 200 → GET 带 lineup.share_code 8 位 */ }
    #[tokio::test] async fn put_maps_validation_errors_to_400_and_unknown_team_to_404() { /* 阵型无效/他队球员 → 400；team 5 → 404 */ }
    #[tokio::test] async fn put_rejects_malformed_json_without_500() { /* 非法 JSON → 4xx */ }
}
```

实现者要把上面的 `todo!()` 与注释补成完整测试（断言 JSON 字段与状态码），再写路由；先确认 `cargo test lineup::adapters::my_web` 失败（空 Router 全 404），再实现。

- [ ] **Step 2: 实现路由与鉴权**

```rust
use crate::{
    auth::ports::token_port::TokenPort,
    lineup::{
        application::my_lineup::MyLineupService,
        domain::{Lineup, LineupSaveError, LineupSlot},
    },
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct MyLineupWebState {
    pub service: Arc<MyLineupService>,
    pub token_port: Arc<dyn TokenPort>,
}

#[derive(Serialize)]
struct LineupResponse {
    lineup: Option<Lineup>,
}

#[derive(Deserialize)]
struct SaveRequest {
    formation: String,
    slots: Vec<LineupSlot>,
}

pub fn my_routes(state: Arc<MyLineupWebState>) -> Router {
    Router::new()
        .route("/api/v1/lineups/me/{team_id}", get(get_mine).put(save_mine))
        .with_state(state)
}

fn authenticate(headers: &HeaderMap, token_port: &dyn TokenPort) -> Result<Uuid, (StatusCode, String)> {
    let unauthorized = || (StatusCode::UNAUTHORIZED, "请先登录".to_string());
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(unauthorized)?;
    token_port.verify_token(token).map(|c| c.sub).map_err(|_| unauthorized())
}

fn map_error(e: LineupSaveError) -> (StatusCode, String) {
    let status = match e {
        LineupSaveError::InvalidFormation
        | LineupSaveError::InvalidSlots
        | LineupSaveError::PlayerNotInSquad
        | LineupSaveError::DuplicatePlayer => StatusCode::BAD_REQUEST,
        LineupSaveError::TeamNotFound => StatusCode::NOT_FOUND,
        LineupSaveError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    };
    (status, e.to_string())
}

async fn get_mine(
    State(s): State<Arc<MyLineupWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
) -> Result<Json<LineupResponse>, (StatusCode, String)> {
    let user = authenticate(&headers, s.token_port.as_ref())?;
    s.service.get(user, team_id).await.map(|lineup| Json(LineupResponse { lineup })).map_err(map_error)
}

async fn save_mine(
    State(s): State<Arc<MyLineupWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<LineupResponse>, (StatusCode, String)> {
    let user = authenticate(&headers, s.token_port.as_ref())?;
    s.service
        .save(user, team_id, &req.formation, req.slots)
        .await
        .map(|lineup| Json(LineupResponse { lineup: Some(lineup) }))
        .map_err(map_error)
}
```

- [ ] **Step 3: 装配**

`adapters/mod.rs` 加 `pub mod my_web;`。`bootstrap.rs`：

```rust
use super::{
    adapters::{
        my_web::{MyLineupWebState, my_routes},
        persistence::{PostgresLineupRepository, PostgresSquadRepository},
        web::routes,
    },
    application::{LineupService, my_lineup::MyLineupService},
};
use crate::auth::ports::token_port::TokenPort;
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_lineup_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router {
    let squads = Arc::new(PostgresSquadRepository::new(pool.clone()));
    let lineups = Arc::new(PostgresLineupRepository::new(pool));
    routes(Arc::new(LineupService::new(squads.clone()))).merge(my_routes(Arc::new(MyLineupWebState {
        service: Arc::new(MyLineupService::new(squads, lineups)),
        token_port,
    })))
}
```

`app.rs`：`build_lineup_routes(pool.clone(), auth.token_port.clone())`（`auth` 在该行之前已定义；若 `lineup_routes` 当前定义位置早于 `auth`，把这一行移到 `auth` 之后）。

- [ ] **Step 4: 运行与重启**

```bash
cd football_insight_service_backend_rs
cargo test lineup 2>&1 | grep -E "test result: .* [1-9][0-9]* passed|FAILED|^error"
cargo test 2>&1 | grep -E "^test result" | awk '{p+=$4; f+=$6} END {print "passed",p,"failed",f}'
cd .. && ./deploy/local-test/up.sh > /tmp/lineup-up.log 2>&1; tail -2 /tmp/lineup-up.log
curl -s -m 8 -o /dev/null -w "me-unauth=%{http_code}\n" http://127.0.0.1:18092/api/v1/lineups/me/77680
curl -s -m 8 -o /dev/null -w "squads=%{http_code}\n" http://127.0.0.1:18092/api/v1/lineups/squads/77680
```

Expected: 全部 passed、0 failed；`me-unauth=401`；`squads=200`。**迁移尚未在生产执行，本地后端连生产库，所以此时带登录的 GET 会因表不存在而 503（不要在生产库写入测试数据）**；带登录的完整读写验证等 Task 10 执行迁移后再做。

- [ ] **Step 5: 格式化、提交**

```bash
cd football_insight_service_backend_rs && rustfmt --edition 2024 src/lineup/adapters/my_web.rs src/lineup/adapters/mod.rs src/lineup/bootstrap.rs && cd .. && git status --short
git pull --rebase
git add football_insight_service_backend_rs/src
git commit -m "feat(backend): add authenticated get and save my lineup endpoints

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 5: 小程序 —— 类型、接口、阵型数据与纯函数

**Files:**
- Create: `football_insight_mini/src/types/lineup.ts`、`src/api/lineup.ts`、`src/pages/lineup/formations.ts`、`src/pages/lineup/helpers.ts`
- Test: `football_insight_mini/src/pages/lineup/helpers.test.ts`、`src/pages/lineup/formations.test.ts`

**Interfaces:**
- Produces:

```ts
// types/lineup.ts
export type LineupPosition = 'GK' | 'DF' | 'MF' | 'FW'
export interface SquadPlayer { player_id: number; name: string; shirt_number: number | null
  position: LineupPosition | null; avatar_url: string | null; last_season: number }
export interface SquadResponse { team: { team_id: number; team_name: string; avatar_url: string | null };
  active: SquadPlayer[]; former: SquadPlayer[] }
export interface LineupSlotDto { slot: number; player_id: number | null; x: number; y: number }
export interface SavedLineup { formation: string; slots: LineupSlotDto[]; share_code: string; updated_at: string }
// api/lineup.ts
getSquad(teamId: number): Promise<SquadResponse>
getMyLineup(teamId: number): Promise<{ lineup: SavedLineup | null }>      // auth
saveMyLineup(teamId: number, payload: { formation: string; slots: LineupSlotDto[] }): Promise<{ lineup: SavedLineup }>  // auth
// formations.ts
export interface FormationSlot { slot: number; position: LineupPosition; label: string; x: number; y: number }
export const FORMATION_CODES: readonly ['4-3-3','4-4-2','4-2-3-1','3-5-2','3-4-3','5-3-2','4-1-4-1']
export function getFormation(code: string): FormationSlot[]          // 未知代码回退 4-3-3
export const DEFAULT_FORMATION = '4-3-3'
// helpers.ts
export const TAP_THRESHOLD_RPX = 10
export const SWAP_DISTANCE_RATIO = 0.6
export const COORD_MIN = 0.04; export const COORD_MAX = 0.96
export function clampCoord(v: number): number
export type LineupSlots = Array<{ slot: number; player_id: number | null; x: number; y: number }>
export function emptyLineup(formation: string): LineupSlots          // 11 项，坐标取阵型默认，player_id 全为空
export function reassignPlayersToFormation(slots: LineupSlots, nextFormation: string, positionOf: (playerId: number) => LineupPosition | null): LineupSlots
export function resetPositions(slots: LineupSlots, formation: string): LineupSlots
export type DragOutcome =
  | { kind: 'tap' }
  | { kind: 'swap'; withSlot: number }
  | { kind: 'move'; x: number; y: number }
export function resolveDragRelease(input: { slot: number; fromX: number; fromY: number; toX: number; toY: number;
  areaWidthRpx: number; areaHeightRpx: number; dotDiameterRpx: number; slots: LineupSlots }): DragOutcome
export function applyDragOutcome(slots: LineupSlots, slot: number, outcome: DragOutcome): LineupSlots
export function assignPlayer(slots: LineupSlots, slot: number, playerId: number | null): LineupSlots   // 已在场的球员先从原位置移走（互换语义见测试）
export function countFilled(slots: LineupSlots): number
```

- [ ] **Step 1: 类型与接口**（无逻辑，随后用到时由类型检查约束）

```ts
// src/types/lineup.ts
export type LineupPosition = 'GK' | 'DF' | 'MF' | 'FW'

export interface SquadPlayer {
  player_id: number
  name: string
  shirt_number: number | null
  position: LineupPosition | null
  avatar_url: string | null
  last_season: number
}

export interface SquadResponse {
  team: { team_id: number; team_name: string; avatar_url: string | null }
  active: SquadPlayer[]
  former: SquadPlayer[]
}

export interface LineupSlotDto {
  slot: number
  player_id: number | null
  x: number
  y: number
}

export interface SavedLineup {
  formation: string
  slots: LineupSlotDto[]
  share_code: string
  updated_at: string
}
```

```ts
// src/api/lineup.ts
import type { LineupSlotDto, SavedLineup, SquadResponse } from '../types/lineup'
import { request } from '../utils/request'

export function getSquad(teamId: number): Promise<SquadResponse> {
  return request<SquadResponse>({ url: `/lineups/squads/${teamId}` })
}

export function getMyLineup(teamId: number): Promise<{ lineup: SavedLineup | null }> {
  return request<{ lineup: SavedLineup | null }>({ url: `/lineups/me/${teamId}`, auth: true })
}

export function saveMyLineup(
  teamId: number,
  payload: { formation: string; slots: LineupSlotDto[] },
): Promise<{ lineup: SavedLineup }> {
  return request<{ lineup: SavedLineup }>({ url: `/lineups/me/${teamId}`, method: 'PUT', auth: true, data: payload })
}
```

- [ ] **Step 2: 写失败测试（阵型数据）**

```ts
// src/pages/lineup/formations.test.ts
import { describe, expect, test } from 'bun:test'
import { DEFAULT_FORMATION, FORMATION_CODES, getFormation } from './formations'

const DOT_RATIO = 0.12 // 圆点直径约占球场宽度的 12%，任意两点中心距不得小于它

describe('formations', () => {
  for (const code of FORMATION_CODES) {
    test(`${code}: 11 slots, one goalkeeper, unique indices, coords in the unit square`, () => {
      const slots = getFormation(code)
      expect(slots.length).toBe(11)
      expect(slots.filter((s) => s.position === 'GK').length).toBe(1)
      expect(new Set(slots.map((s) => s.slot)).size).toBe(11)
      expect(slots.map((s) => s.slot).sort((a, b) => a - b)).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
      for (const s of slots) {
        expect(s.x >= 0 && s.x <= 1 && s.y >= 0 && s.y <= 1).toBe(true)
      }
      const gk = slots.find((s) => s.position === 'GK')!
      expect([gk.x, gk.y]).toEqual([0.5, 0.9])
    })

    test(`${code}: no two dots overlap`, () => {
      const slots = getFormation(code)
      for (let i = 0; i < slots.length; i += 1) {
        for (let j = i + 1; j < slots.length; j += 1) {
          const d = Math.hypot(slots[i].x - slots[j].x, (slots[i].y - slots[j].y) * 0.75)
          expect(d >= DOT_RATIO).toBe(true)
        }
      }
    })

    test(`${code}: position counts match the code (defenders-midfielders-forwards)`, () => {
      const slots = getFormation(code)
      const lines = code.split('-').map(Number)
      const df = slots.filter((s) => s.position === 'DF').length
      const fw = slots.filter((s) => s.position === 'FW').length
      const mf = slots.filter((s) => s.position === 'MF').length
      expect(df).toBe(lines[0])
      expect(fw).toBe(lines[lines.length - 1])
      expect(mf).toBe(lines.slice(1, -1).reduce((a, b) => a + b, 0))
    })
  }

  test('unknown code falls back to the default formation', () => {
    expect(getFormation('9-9-9')).toEqual(getFormation(DEFAULT_FORMATION))
  })
})
```

（"两点中心距不小于 12%"里对 y 乘 0.75 是球场宽高比 750×1000 的近似换算，保证视觉上不重叠；若某个阵型因此失败，调整 `formations.ts` 的坐标而不是放宽阈值。）

Run: `cd football_insight_mini && bun test src/pages/lineup/formations.test.ts 2>&1 | tail -3`，Expected: FAIL（模块不存在）。

- [ ] **Step 3: 实现 `formations.ts`**

按 spec 第 5 节坐标表实现。位置序号：0 = 门将，其后后卫、中场、前锋，各类别内从左到右。位置名：GK→"门将"；DF：4 人时 左后卫/左中卫/右中卫/右后卫，3 人时 左中卫/中卫/右中卫，5 人时 左翼卫/左中卫/中卫/右中卫/右翼卫；MF 与 FW 按阵型给"左中场/中场/右中场/后腰/前腰/边锋/前锋"等（实现者按 spec 表中的行分组命名，保证每个位置 `label` 非空）。

```ts
import type { LineupPosition } from '../../types/lineup'

export interface FormationSlot {
  slot: number
  position: LineupPosition
  label: string
  x: number
  y: number
}

export const FORMATION_CODES = ['4-3-3', '4-4-2', '4-2-3-1', '3-5-2', '3-4-3', '5-3-2', '4-1-4-1'] as const
export const DEFAULT_FORMATION = '4-3-3'

type Row = { position: LineupPosition; y: number; xs: number[]; labels: string[] }

// 行定义（y 为纵向位置，xs 为该行各球员的 x，从左到右）。门将固定在 (0.5, 0.9)。
const ROWS: Record<(typeof FORMATION_CODES)[number], Row[]> = {
  '4-3-3': [
    { position: 'GK', y: 0.9, xs: [0.5], labels: ['门将'] },
    { position: 'DF', y: 0.74, xs: [0.12, 0.37, 0.63, 0.88], labels: ['左后卫', '左中卫', '右中卫', '右后卫'] },
    { position: 'MF', y: 0.52, xs: [0.25, 0.5, 0.75], labels: ['左中场', '中场', '右中场'] },
    { position: 'FW', y: 0.24, xs: [0.18, 0.82], labels: ['左边锋', '右边锋'] },
    { position: 'FW', y: 0.18, xs: [0.5], labels: ['中锋'] },
  ],
  // 其余 6 个阵型按 spec 第 5.2 节表格逐行填写，同一格式；每行 xs 与 labels 长度必须相同。
  '4-4-2': [/* … */],
  '4-2-3-1': [/* … */],
  '3-5-2': [/* … */],
  '3-4-3': [/* … */],
  '5-3-2': [/* … */],
  '4-1-4-1': [/* … */],
}

export function getFormation(code: string): FormationSlot[] {
  const rows = ROWS[(FORMATION_CODES as readonly string[]).includes(code) ? (code as keyof typeof ROWS) : DEFAULT_FORMATION]
  let slot = 0
  const result: FormationSlot[] = []
  // 类别内从左到右编号：同类别的多行按 y 从大到小（靠近己方球门的先编）依次排
  for (const row of rows) {
    row.xs.forEach((x, i) => {
      result.push({ slot: slot++, position: row.position, label: row.labels[i], x, y: row.y })
    })
  }
  return result
}
```

实现者必须把 6 个 `/* … */` 按 spec 第 5.2 节补全（不得留占位），并保证 Step 2 的全部测试通过。

Run: `bun test src/pages/lineup/formations.test.ts 2>&1 | tail -3`，Expected: 全部 pass。

- [ ] **Step 4: 写失败测试（helpers）**

```ts
// src/pages/lineup/helpers.test.ts
import { describe, expect, test } from 'bun:test'
import { getFormation } from './formations'
import {
  COORD_MAX, COORD_MIN, SWAP_DISTANCE_RATIO, TAP_THRESHOLD_RPX,
  applyDragOutcome, assignPlayer, clampCoord, countFilled, emptyLineup,
  reassignPlayersToFormation, resetPositions, resolveDragRelease,
  type LineupSlots,
} from './helpers'

describe('clampCoord', () => {
  test('keeps dots inside 0.04–0.96', () => {
    expect(clampCoord(-1)).toBe(COORD_MIN)
    expect(clampCoord(2)).toBe(COORD_MAX)
    expect(clampCoord(0.5)).toBe(0.5)
    expect(clampCoord(Number.NaN)).toBe(0.5)
  })
})

describe('emptyLineup / resetPositions / countFilled', () => {
  test('empty lineup has 11 empty slots at default coordinates', () => {
    const slots = emptyLineup('4-4-2')
    expect(slots.length).toBe(11)
    expect(slots.every((s) => s.player_id === null)).toBe(true)
    expect(slots.map((s) => [s.x, s.y])).toEqual(getFormation('4-4-2').map((s) => [s.x, s.y]))
    expect(countFilled(slots)).toBe(0)
  })

  test('resetPositions restores coordinates but keeps players', () => {
    const moved: LineupSlots = emptyLineup('4-3-3').map((s, i) => ({ ...s, player_id: i < 3 ? 100 + i : null, x: 0.9, y: 0.1 }))
    const reset = resetPositions(moved, '4-3-3')
    expect(reset.map((s) => s.player_id)).toEqual(moved.map((s) => s.player_id))
    expect(reset.map((s) => [s.x, s.y])).toEqual(getFormation('4-3-3').map((s) => [s.x, s.y]))
    expect(countFilled(reset)).toBe(3)
  })
})

describe('assignPlayer', () => {
  test('puts a player into a slot', () => {
    const slots = assignPlayer(emptyLineup('4-3-3'), 5, 42)
    expect(slots.find((s) => s.slot === 5)!.player_id).toBe(42)
  })

  test('a player already on the pitch moves instead of duplicating; the target occupant swaps into the old slot', () => {
    let slots = assignPlayer(emptyLineup('4-3-3'), 1, 10)
    slots = assignPlayer(slots, 2, 20)
    slots = assignPlayer(slots, 2, 10) // 把 10 号选到 2 号位：互换
    expect(slots.find((s) => s.slot === 2)!.player_id).toBe(10)
    expect(slots.find((s) => s.slot === 1)!.player_id).toBe(20)
    expect(slots.filter((s) => s.player_id === 10).length).toBe(1)
  })

  test('null removes the occupant', () => {
    const slots = assignPlayer(assignPlayer(emptyLineup('4-3-3'), 3, 7), 3, null)
    expect(slots.find((s) => s.slot === 3)!.player_id).toBeNull()
  })
})

describe('reassignPlayersToFormation', () => {
  const positions: Record<number, 'GK' | 'DF' | 'MF' | 'FW'> = { 1: 'GK', 2: 'DF', 3: 'DF', 4: 'DF', 5: 'DF', 6: 'MF', 7: 'MF', 8: 'MF', 9: 'FW', 10: 'FW', 11: 'FW' }
  const positionOf = (id: number) => positions[id] ?? null

  function filled(formation: string, ids: number[]): LineupSlots {
    const form = getFormation(formation)
    return emptyLineup(formation).map((s, i) => ({ ...s, player_id: ids[i] ?? null, x: form[i].x, y: form[i].y }))
  }

  test('4-3-3 → 4-4-2 keeps every player once and resets coordinates', () => {
    const before = filled('4-3-3', [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    const after = reassignPlayersToFormation(before, '4-4-2', positionOf)
    const ids = after.map((s) => s.player_id).filter((id): id is number => id !== null).sort((a, b) => a - b)
    expect(ids).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    expect(after.map((s) => [s.x, s.y])).toEqual(getFormation('4-4-2').map((s) => [s.x, s.y]))
    // 门将仍在门将位
    expect(after.find((s) => s.slot === 0)!.player_id).toBe(1)
  })

  test('goalkeeper stays in the keeper slot and defenders prefer defender slots', () => {
    const before = filled('4-3-3', [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    const after = reassignPlayersToFormation(before, '5-3-2', positionOf)
    const slotsByPos = (pos: string) => getFormation('5-3-2').filter((f) => f.position === pos).map((f) => after.find((s) => s.slot === f.slot)!.player_id)
    expect(slotsByPos('GK')).toEqual([1])
    expect(slotsByPos('DF').sort()).toEqual([2, 3, 4, 5, null].sort())
  })

  test('players without a known position and overflow players are placed into remaining empty slots, never dropped or duplicated', () => {
    const many = filled('4-3-3', [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    const after = reassignPlayersToFormation(many, '3-4-3', (id) => (id === 6 ? null : positionOf(id)))
    const ids = after.map((s) => s.player_id).filter((id): id is number => id !== null)
    expect(new Set(ids).size).toBe(ids.length)
    expect(ids.length).toBe(11)
  })

  test('partial lineups keep the same players', () => {
    const before = filled('4-3-3', [1, 2, 6])
    const after = reassignPlayersToFormation(before, '4-2-3-1', positionOf)
    expect(after.map((s) => s.player_id).filter((id) => id !== null).sort()).toEqual([1, 2, 6])
  })
})

describe('resolveDragRelease', () => {
  const area = { areaWidthRpx: 670, areaHeightRpx: 900, dotDiameterRpx: 84 }
  function lineup(): LineupSlots {
    return emptyLineup('4-3-3').map((s, i) => ({ ...s, player_id: 100 + i }))
  }
  const slots = lineup()
  const from = slots[1] // 左后卫

  test(`movement under ${TAP_THRESHOLD_RPX}rpx is a tap`, () => {
    const fromX = from.x * area.areaWidthRpx
    const fromY = from.y * area.areaHeightRpx
    expect(resolveDragRelease({ slot: from.slot, fromX, fromY, toX: fromX + 4, toY: fromY + 3, ...area, slots }).kind).toBe('tap')
  })

  test('dropping near another player swaps with that slot', () => {
    const target = slots[4]
    const outcome = resolveDragRelease({
      slot: from.slot,
      fromX: from.x * area.areaWidthRpx, fromY: from.y * area.areaHeightRpx,
      toX: target.x * area.areaWidthRpx + 5, toY: target.y * area.areaHeightRpx - 5, ...area, slots,
    })
    expect(outcome).toEqual({ kind: 'swap', withSlot: target.slot })
  })

  test(`swap distance is ${SWAP_DISTANCE_RATIO} of the dot diameter`, () => {
    const target = slots[4]
    const cx = target.x * area.areaWidthRpx
    const cy = target.y * area.areaHeightRpx
    const edge = area.dotDiameterRpx * SWAP_DISTANCE_RATIO
    const near = resolveDragRelease({ slot: from.slot, fromX: 0, fromY: 0, toX: cx + edge - 1, toY: cy, ...area, slots })
    const far = resolveDragRelease({ slot: from.slot, fromX: 0, fromY: 0, toX: cx + edge + 1, toY: cy, ...area, slots })
    expect(near.kind).toBe('swap')
    expect(far.kind).toBe('move')
  })

  test('dropping on empty pitch moves and clamps to the allowed range', () => {
    const outside = resolveDragRelease({ slot: from.slot, fromX: 100, fromY: 100, toX: -300, toY: 5000, ...area, slots })
    expect(outside.kind).toBe('move')
    if (outside.kind === 'move') {
      expect(outside.x).toBe(COORD_MIN)
      expect(outside.y).toBe(COORD_MAX)
    }
  })

  test('dropping on its own position is not a swap with itself', () => {
    const fromX = from.x * area.areaWidthRpx
    const fromY = from.y * area.areaHeightRpx
    const outcome = resolveDragRelease({ slot: from.slot, fromX: fromX + 40, fromY, toX: fromX + 2, toY: fromY, ...area, slots })
    expect(outcome.kind).not.toBe('swap')
  })
})

describe('applyDragOutcome', () => {
  test('tap leaves slots untouched', () => {
    const slots = emptyLineup('4-3-3')
    expect(applyDragOutcome(slots, 1, { kind: 'tap' })).toEqual(slots)
  })

  test('swap exchanges the players but each slot keeps its own coordinates swapped too', () => {
    let slots = assignPlayer(assignPlayer(emptyLineup('4-3-3'), 1, 10), 4, 40)
    const a = slots.find((s) => s.slot === 1)!
    const b = slots.find((s) => s.slot === 4)!
    slots = applyDragOutcome(slots, 1, { kind: 'swap', withSlot: 4 })
    const a2 = slots.find((s) => s.slot === 1)!
    const b2 = slots.find((s) => s.slot === 4)!
    expect([a2.player_id, b2.player_id]).toEqual([40, 10])
    expect([a2.x, a2.y]).toEqual([a.x, a.y])
    expect([b2.x, b2.y]).toEqual([b.x, b.y])
  })

  test('move writes the new coordinates only for that slot', () => {
    const slots = applyDragOutcome(emptyLineup('4-3-3'), 2, { kind: 'move', x: 0.3, y: 0.6 })
    const moved = slots.find((s) => s.slot === 2)!
    expect([moved.x, moved.y]).toEqual([0.3, 0.6])
    expect(slots.filter((s) => s.slot !== 2).every((s, i) => s.x === getFormation('4-3-3').find((f) => f.slot === s.slot)!.x)).toBe(true)
  })
})
```

Run: `bun test src/pages/lineup/helpers.test.ts 2>&1 | tail -3`，Expected: FAIL（模块不存在）。

- [ ] **Step 5: 实现 `helpers.ts`**

要点（实现者据此写代码，使 Step 4 全部通过）：

```ts
import type { LineupPosition } from '../../types/lineup'
import { DEFAULT_FORMATION, getFormation } from './formations'

export const TAP_THRESHOLD_RPX = 10
export const SWAP_DISTANCE_RATIO = 0.6
export const COORD_MIN = 0.04
export const COORD_MAX = 0.96

export type LineupSlots = Array<{ slot: number; player_id: number | null; x: number; y: number }>

export function clampCoord(value: number): number {
  if (!Number.isFinite(value)) return 0.5
  return Math.min(COORD_MAX, Math.max(COORD_MIN, value))
}

export function emptyLineup(formation: string): LineupSlots {
  return getFormation(formation).map((s) => ({ slot: s.slot, player_id: null, x: s.x, y: s.y }))
}

export function resetPositions(slots: LineupSlots, formation: string): LineupSlots {
  const defaults = new Map(getFormation(formation).map((s) => [s.slot, s]))
  return slots.map((s) => ({ ...s, x: defaults.get(s.slot)?.x ?? s.x, y: defaults.get(s.slot)?.y ?? s.y }))
}

export function countFilled(slots: LineupSlots): number {
  return slots.filter((s) => s.player_id !== null).length
}

export function assignPlayer(slots: LineupSlots, slot: number, playerId: number | null): LineupSlots {
  const target = slots.find((s) => s.slot === slot)
  if (!target) return slots
  if (playerId === null) return slots.map((s) => (s.slot === slot ? { ...s, player_id: null } : s))
  const from = slots.find((s) => s.player_id === playerId)
  return slots.map((s) => {
    if (s.slot === slot) return { ...s, player_id: playerId }
    // 该球员已在场：把目标位置原来的人换到他原来的位置（目标为空则原位置清空）
    if (from && s.slot === from.slot) return { ...s, player_id: target.player_id }
    return s
  })
}
```

`reassignPlayersToFormation(slots, nextFormation, positionOf)`：
1. 取出所有在场球员（保持原 slot 顺序）。
2. 目标阵型 `getFormation(next)` 按类别（GK→DF→MF→FW）把球员依次放进同类别空位（球员类别由 `positionOf` 给出；`null` 或未知的暂存为"待定"）。
3. 同类别放不下的溢出球员、以及待定球员，按 slot 序放入剩余空位（先 GK 以外的空位，最后才用 GK 位，且 GK 位只在其余空位都满后才用）。
4. 返回的坐标全部取新阵型默认值；球员不重复、不丢失。

`resolveDragRelease`：`fromX/fromY/toX/toY` 为 rpx 坐标（相对球场区域左上角，圆点**中心**）；位移 `hypot(toX-fromX, toY-fromY) < TAP_THRESHOLD_RPX` → `tap`；否则在其他 slot（`slot !== input.slot`）中找中心距离 `< dotDiameterRpx * SWAP_DISTANCE_RATIO` 且最近的，命中返回 `swap`；否则返回 `move`，坐标 `clampCoord(toX / areaWidthRpx)`、`clampCoord(toY / areaHeightRpx)`。

`applyDragOutcome`：`tap` 原样返回；`swap` 交换两个 slot 的 `player_id`（**不交换坐标**：各 slot 保留自己的坐标）；`move` 只改该 slot 的 `x/y`（已钳制）。

注意 Step 4 里 `applyDragOutcome` 的 swap 用例断言"各 slot 保留自己的坐标"，即只交换 `player_id`。

Run: `bun test src/pages/lineup 2>&1 | tail -3 && bun run type-check 2>&1 | tail -2`，Expected: 全部 pass、类型检查通过。

- [ ] **Step 6: 提交**

```bash
git pull --rebase
git add football_insight_mini/src/types/lineup.ts football_insight_mini/src/api/lineup.ts football_insight_mini/src/pages/lineup
git commit -m "feat(mini): add lineup formations, drag rules and lineup helpers

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 6: 小程序 —— 展示模型、草稿、数据层与编辑状态

**Files:**
- Create: `football_insight_mini/src/pages/lineup/lineupViewModel.ts`、`lineupDraft.ts`、`useLineupPage.ts`、`useLineupEditor.ts`
- Test: `src/pages/lineup/lineupViewModel.test.ts`、`lineupDraft.test.ts`、`useLineup.test.ts`

**Interfaces:**
- Consumes: Task 5 全部；`useAppSession().hasAccessToken`、`rememberPostLoginRedirect`、`uni.getStorageSync/setStorageSync`。
- Produces:

```ts
// lineupViewModel.ts
export interface PitchPlayer { slot: number; label: string; position: LineupPosition; x: number; y: number
  player: null | { player_id: number; name: string; shirt_number: number | null; avatar_url: string | null; former: boolean; last_season: number } }
export function buildPitchPlayers(slots: LineupSlots, formation: string, squad: SquadResponse | null): PitchPlayer[]
export interface SheetGroup { key: 'active' | 'former'; title: string; players: SheetPlayer[] }
export interface SheetPlayer { player_id: number; name: string; shirt_number: number | null; position: LineupPosition | null
  avatar_url: string | null; former: boolean; last_season: number; onPitch: boolean; preferred: boolean }
export function buildSheetGroups(squad: SquadResponse, slots: LineupSlots, targetPosition: LineupPosition): SheetGroup[]
export function positionIndex(squad: SquadResponse | null): (playerId: number) => LineupPosition | null
// lineupDraft.ts
export interface LineupDraft { teamId: number; formation: string; slots: LineupSlots; savedAt: number }
export function draftKey(teamId: number): string
export function readDraft(teamId: number, storage?: Storage): LineupDraft | null
export function writeDraft(draft: LineupDraft, storage?: Storage): void
export function clearDraft(teamId: number, storage?: Storage): void
export function sanitizeDraftSlots(slots: LineupSlots, knownPlayerIds: Set<number>): LineupSlots   // 不认识的 player_id 置空
// useLineupEditor.ts
export function useLineupEditor(): { formation: Ref<string>; slots: Ref<LineupSlots>; dirty: Ref<boolean>; selectedSlot: Ref<number | null>;
  load(formation: string, slots: LineupSlots, markDirty?: boolean): void; setFormation(next: string, positionOf): void;
  pick(slot: number, playerId: number | null): void; applyDrag(slot: number, outcome: DragOutcome): void; reset(): void; markSaved(): void; filled: ComputedRef<number> }
// useLineupPage.ts
export function useLineupPage(deps?): { teamId; squad; loading; errorMessage; saving; saveState: Ref<'idle'|'saved'|'dirty'|'error'>; editor; load(teamId): Promise<void>; save(): Promise<'saved'|'need-login'|'failed'> }
```

- [ ] **Step 1: 写失败测试（展示模型）**

```ts
// src/pages/lineup/lineupViewModel.test.ts
import { describe, expect, test } from 'bun:test'
import type { SquadPlayer, SquadResponse } from '../../types/lineup'
import { assignPlayer, emptyLineup } from './helpers'
import { buildPitchPlayers, buildSheetGroups, positionIndex } from './lineupViewModel'

function player(id: number, position: SquadPlayer['position'], extra: Partial<SquadPlayer> = {}): SquadPlayer {
  return { player_id: id, name: `P${id}`, shirt_number: id, position, avatar_url: null, last_season: 2026, ...extra }
}

const squad: SquadResponse = {
  team: { team_id: 1, team_name: '成都蓉城', avatar_url: null },
  active: [player(1, 'GK'), player(2, 'DF'), player(3, 'MF'), player(4, 'FW')],
  former: [player(20, 'DF', { last_season: 2025 }), player(21, 'FW', { last_season: 2024 })],
}

describe('buildPitchPlayers', () => {
  test('joins slots with squad data and marks former players', () => {
    let slots = assignPlayer(emptyLineup('4-3-3'), 0, 1)
    slots = assignPlayer(slots, 1, 20)
    const pitch = buildPitchPlayers(slots, '4-3-3', squad)
    expect(pitch.length).toBe(11)
    expect(pitch[0].player).toMatchObject({ player_id: 1, former: false })
    expect(pitch[1].player).toMatchObject({ player_id: 20, former: true, last_season: 2025 })
    expect(pitch[2].player).toBeNull()
    expect(pitch[0].label).toBe('门将')
  })

  test('former_player_in_saved_lineup_stays_visible even when the squad later lists him as former', () => {
    const slots = assignPlayer(emptyLineup('4-3-3'), 3, 21)
    expect(buildPitchPlayers(slots, '4-3-3', squad)[3].player).toMatchObject({ player_id: 21, former: true })
  })

  test('a player id the squad no longer contains renders as an empty slot', () => {
    const slots = assignPlayer(emptyLineup('4-3-3'), 3, 9999)
    expect(buildPitchPlayers(slots, '4-3-3', squad)[3].player).toBeNull()
  })

  test('works before the squad has loaded', () => {
    expect(buildPitchPlayers(emptyLineup('4-3-3'), '4-3-3', null).every((p) => p.player === null)).toBe(true)
  })
})

describe('buildSheetGroups', () => {
  test('active group first, former group second; preferred position first; on-pitch players flagged', () => {
    const slots = assignPlayer(emptyLineup('4-3-3'), 1, 2)
    const groups = buildSheetGroups(squad, slots, 'DF')
    expect(groups.map((g) => g.key)).toEqual(['active', 'former'])
    const active = groups[0].players
    expect(active[0].player_id).toBe(2) // DF 对口排最前
    expect(active.find((p) => p.player_id === 2)!.onPitch).toBe(true)
    expect(active.find((p) => p.player_id === 3)!.onPitch).toBe(false)
    expect(active.find((p) => p.player_id === 1)!.preferred).toBe(false)
    expect(groups[1].players[0].player_id).toBe(20) // 前队员：对口的 DF 在前
    expect(groups[1].players.every((p) => p.former)).toBe(true)
  })

  test('omits the former group when there are no former players', () => {
    expect(buildSheetGroups({ ...squad, former: [] }, emptyLineup('4-3-3'), 'MF').map((g) => g.key)).toEqual(['active'])
  })
})

describe('positionIndex', () => {
  test('looks up positions of both current and former players', () => {
    const of = positionIndex(squad)
    expect(of(2)).toBe('DF')
    expect(of(21)).toBe('FW')
    expect(of(999)).toBeNull()
    expect(positionIndex(null)(1)).toBeNull()
  })
})
```

Run: `bun test src/pages/lineup/lineupViewModel.test.ts 2>&1 | tail -3`，Expected: FAIL。实现 `lineupViewModel.ts`（`former` 取自球员是否在 `squad.former`；对口排序：`position === targetPosition` 优先，其次按名单原顺序；前队员组按名单已有的顺序，对口的排前）。Expected after: pass。

- [ ] **Step 2: 写失败测试（草稿）**

```ts
// src/pages/lineup/lineupDraft.test.ts
import { describe, expect, test } from 'bun:test'
import { assignPlayer, emptyLineup } from './helpers'
import { clearDraft, draftKey, readDraft, sanitizeDraftSlots, writeDraft } from './lineupDraft'

function memoryStorage() {
  const map = new Map<string, unknown>()
  return {
    getStorageSync: (k: string) => map.get(k) ?? '',
    setStorageSync: (k: string, v: unknown) => { map.set(k, v) },
    removeStorageSync: (k: string) => { map.delete(k) },
  }
}

describe('lineup draft', () => {
  test('is stored per team', () => {
    expect(draftKey(1)).not.toBe(draftKey(2))
  })

  test('round trips and clears', () => {
    const storage = memoryStorage()
    const slots = assignPlayer(emptyLineup('4-4-2'), 0, 5)
    writeDraft({ teamId: 1, formation: '4-4-2', slots, savedAt: 1 }, storage)
    expect(readDraft(1, storage)).toEqual({ teamId: 1, formation: '4-4-2', slots, savedAt: 1 })
    expect(readDraft(2, storage)).toBeNull()
    clearDraft(1, storage)
    expect(readDraft(1, storage)).toBeNull()
  })

  test('rejects corrupted drafts instead of crashing', () => {
    const storage = memoryStorage()
    storage.setStorageSync(draftKey(1), { teamId: 1, formation: '4-4-2', slots: 'nope' })
    expect(readDraft(1, storage)).toBeNull()
    storage.setStorageSync(draftKey(1), { teamId: 1, formation: '9-9-9', slots: emptyLineup('4-4-2'), savedAt: 1 })
    expect(readDraft(1, storage)).toBeNull()
    storage.setStorageSync(draftKey(1), { teamId: 1, formation: '4-4-2', slots: emptyLineup('4-4-2').slice(0, 5), savedAt: 1 })
    expect(readDraft(1, storage)).toBeNull()
  })

  test('sanitize blanks players the squad no longer contains', () => {
    const slots = assignPlayer(assignPlayer(emptyLineup('4-3-3'), 0, 1), 1, 999)
    const clean = sanitizeDraftSlots(slots, new Set([1]))
    expect(clean[0].player_id).toBe(1)
    expect(clean[1].player_id).toBeNull()
  })
})
```

（`storage` 参数类型用 `Pick<typeof uni, 'getStorageSync' | 'setStorageSync' | 'removeStorageSync'>`，默认值为全局 `uni`。）实现 `lineupDraft.ts` 使其通过；`readDraft` 校验：对象、`teamId` 数字、`formation` 属于 `FORMATION_CODES`、`slots` 为 11 项数组且每项字段类型合法。

- [ ] **Step 3: 写失败测试（编辑状态 + 数据层，注入假接口）**

```ts
// src/pages/lineup/useLineup.test.ts
import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import type { SavedLineup, SquadResponse } from '../../types/lineup'
import { useLineupEditor } from './useLineupEditor'
import { useLineupPage } from './useLineupPage'
import { assignPlayer, emptyLineup } from './helpers'
import { readDraft } from './lineupDraft'

const squad: SquadResponse = {
  team: { team_id: 1, team_name: '成都蓉城', avatar_url: null },
  active: [{ player_id: 1, name: 'A', shirt_number: 1, position: 'GK', avatar_url: null, last_season: 2026 }],
  former: [],
}
const saved: SavedLineup = { formation: '4-4-2', slots: emptyLineup('4-4-2'), share_code: 'ABCDEFGH', updated_at: '2026-10-19T00:00:00Z' }

function memoryStorage() {
  const map = new Map<string, unknown>()
  return { getStorageSync: (k: string) => map.get(k) ?? '', setStorageSync: (k: string, v: unknown) => { map.set(k, v) }, removeStorageSync: (k: string) => { map.delete(k) } }
}

describe('useLineupEditor', () => {
  test('starts clean; changes mark dirty; markSaved clears it', () => {
    const editor = useLineupEditor()
    editor.load('4-3-3', emptyLineup('4-3-3'))
    expect(editor.dirty.value).toBe(false)
    editor.pick(0, 1)
    expect(editor.dirty.value).toBe(true)
    expect(editor.filled.value).toBe(1)
    editor.markSaved()
    expect(editor.dirty.value).toBe(false)
  })

  test('switching formation keeps players and resets coordinates; reset only resets coordinates', () => {
    const editor = useLineupEditor()
    editor.load('4-3-3', assignPlayer(emptyLineup('4-3-3'), 0, 1))
    editor.setFormation('4-4-2', () => 'GK')
    expect(editor.formation.value).toBe('4-4-2')
    expect(editor.slots.value.filter((s) => s.player_id === 1).length).toBe(1)
    editor.applyDrag(5, { kind: 'move', x: 0.2, y: 0.2 })
    editor.reset()
    expect(editor.slots.value.find((s) => s.slot === 5)!.x).not.toBe(0.2)
  })
})

describe('useLineupPage', () => {
  function build(overrides: Partial<Parameters<typeof useLineupPage>[0]> = {}) {
    const storage = memoryStorage()
    const calls: string[] = []
    const page = useLineupPage({
      getSquad: async () => squad,
      getMyLineup: async () => ({ lineup: null }),
      saveMyLineup: async (_t, payload) => { calls.push('save'); return { lineup: { ...saved, formation: payload.formation, slots: payload.slots } } },
      isLoggedIn: () => true,
      storage,
      now: () => 1,
      ...overrides,
    })
    return { page, storage, calls }
  }

  test('loads the saved lineup when there is one, otherwise an empty 4-3-3', async () => {
    const { page } = build({ getMyLineup: async () => ({ lineup: saved }) })
    await page.load(1)
    expect(page.editor.formation.value).toBe('4-4-2')
    const empty = build()
    await empty.page.load(1)
    expect(empty.page.editor.formation.value).toBe('4-3-3')
    expect(empty.page.editor.filled.value).toBe(0)
  })

  test('anonymous visitors can still load and edit (no my-lineup call)', async () => {
    let myCalls = 0
    const { page } = build({ isLoggedIn: () => false, getMyLineup: async () => { myCalls += 1; return { lineup: null } } })
    await page.load(1)
    expect(myCalls).toBe(0)
    expect(page.errorMessage.value).toBe('')
  })

  test('saving while logged out stores a draft and reports need-login', async () => {
    const { page, storage, calls } = build({ isLoggedIn: () => false })
    await page.load(1)
    page.editor.pick(0, 1)
    expect(await page.save()).toBe('need-login')
    expect(calls).toEqual([])
    expect(readDraft(1, storage)?.slots[0].player_id).toBe(1)
  })

  test('a stored draft wins over the saved lineup, is sanitized, and is saved once after login then cleared', async () => {
    const { page, storage, calls } = build({ getMyLineup: async () => ({ lineup: saved }) })
    // 预先写入含未知球员的草稿
    const bad = assignPlayer(assignPlayer(emptyLineup('3-5-2'), 0, 1), 1, 999)
    storage.setStorageSync('fi-lineup-draft-1', { teamId: 1, formation: '3-5-2', slots: bad, savedAt: 1 })
    await page.load(1)
    expect(page.editor.formation.value).toBe('3-5-2')
    expect(page.editor.slots.value[1].player_id).toBeNull() // 未知球员被清掉
    expect(page.editor.dirty.value).toBe(true)
    expect(await page.save()).toBe('saved')
    expect(await page.save()).toBe('saved') // 第二次没有改动：不重复请求
    expect(calls).toEqual(['save'])
    expect(readDraft(1, storage)).toBeNull()
  })

  test('save failure keeps the draft and reports failed', async () => {
    const { page, storage } = build({ saveMyLineup: async () => { throw new Error('offline') } })
    await page.load(1)
    page.editor.pick(0, 1)
    expect(await page.save()).toBe('failed')
    expect(page.saveState.value).toBe('error')
    expect(readDraft(1, storage)).not.toBeNull()
  })

  test('loading failure surfaces an error message', async () => {
    const { page } = build({ getSquad: async () => { throw new Error('boom') } })
    await page.load(1)
    expect(page.errorMessage.value).toBe('boom')
    expect(page.loading.value).toBe(false)
  })
})
```

（`draftKey(1)` 的实际返回值以实现为准；测试里 `'fi-lineup-draft-1'` 必须与 `draftKey(1)` 一致——实现者让 `draftKey` 返回 ``fi-lineup-draft-${teamId}``。）

Run: `bun test src/pages/lineup 2>&1 | tail -3`，Expected: FAIL。

- [ ] **Step 4: 实现 `useLineupEditor.ts`、`useLineupPage.ts`**

`useLineupEditor`：持有 `formation`、`slots`、`dirty`、`selectedSlot`；`load(formation, slots, markDirty=false)`；`setFormation(next, positionOf)` 调 `reassignPlayersToFormation`；`pick(slot, playerId)` 调 `assignPlayer`；`applyDrag` 调 `applyDragOutcome`；`reset` 调 `resetPositions`；任何修改置 `dirty=true`；`markSaved` 置 `dirty=false`；`filled` 为 `countFilled`。

`useLineupPage(deps)`：依赖注入 `getSquad/getMyLineup/saveMyLineup/isLoggedIn/storage/now`，默认值为真实 `api/lineup`、`useAppSession().hasAccessToken`、`uni`、`Date.now`。
- `load(teamId)`：并行取名单与（已登录时）我的阵容；有草稿则优先用草稿（经 `sanitizeDraftSlots`，`markDirty=true`）；否则有已保存阵容用已保存的；否则 `emptyLineup(DEFAULT_FORMATION)`。失败设 `errorMessage`（用 `extractApiErrorMessage`）且 `loading=false`。
- `save()`：未登录 → `writeDraft` 并返回 `'need-login'`；无改动（`!dirty`）直接返回 `'saved'`；调用 `saveMyLineup`，成功 → `markSaved`、`clearDraft`、`saveState='saved'`、返回 `'saved'`；失败 → 保留/写入草稿、`saveState='error'`、返回 `'failed'`。
- `saveState`：`dirty` 时为 `'dirty'`，保存成功 `'saved'`，失败 `'error'`，其余 `'idle'`。

Run: `bun test src/pages/lineup 2>&1 | tail -3 && bun run type-check 2>&1 | tail -2 && bun run check:boundaries | tail -1`，Expected: 全部 pass。

- [ ] **Step 5: 提交**

```bash
git pull --rebase
git add football_insight_mini/src/pages/lineup
git commit -m "feat(mini): add lineup view model, drafts and editor state

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 7: 小程序 —— token、样式审计与展示组件

**Files:**
- Modify: `football_insight_mini/src/styles/fi-tokens.css`
- Create: `football_insight_mini/scripts/lineup-style-audit.test.mjs`
- Create: `football_insight_mini/src/pages/lineup/components/LineupPitch.vue`、`LineupPlayerDot.vue`、`LineupPlayerSheet.vue`、`LineupFormationBar.vue`、`LineupTeamPicker.vue`

**Interfaces:**
- Produces（组件契约，全部纯 props/emit，不 import api/store）：

```ts
// LineupPitch.vue
props: { players: PitchPlayer[]; readonly?: boolean; selectedSlot?: number | null }
emits: { tap: [slot: number]; drag: [slot: number, outcome: DragOutcome] }
// LineupPlayerDot.vue（内部使用，也可单独给只读页/分享图预览用）
props: { player: PitchPlayer; selected?: boolean; compact?: boolean }
// LineupPlayerSheet.vue
props: { visible: boolean; slotLabel: string; occupied: boolean; groups: SheetGroup[] }
emits: { 'update:visible': [boolean]; pick: [playerId: number]; remove: [] }
// LineupFormationBar.vue
props: { modelValue: string }  emits: { 'update:modelValue': [string] }
// LineupTeamPicker.vue
props: { teams: Array<{ team_id: number; team_name: string; avatar_url: string | null }>; modelValue: number }
emits: { 'update:modelValue': [number] }
```

- [ ] **Step 1: token（先加 primitive 再建 semantic）**

在 `fi-tokens.css` 的 primitive 段加：

```css
  /* ===== Primitive：战术板球场（草坪与线条） ===== */
  --fi-primitive-pitch-grass: #2f7d4f;
  --fi-primitive-pitch-grass-alt: #2a7448;
  --fi-primitive-pitch-line: rgba(255, 255, 255, 0.7);
```

semantic 段：

```css
  --fi-color-lineup-grass: var(--fi-primitive-pitch-grass);
  --fi-color-lineup-grass-alt: var(--fi-primitive-pitch-grass-alt);
  --fi-color-lineup-line: var(--fi-primitive-pitch-line);
```

component 段：

```css
  /* 战术板：球场宽高比 3:4；圆点直径 */
  --fi-component-lineup-pitch-ratio: 133.33%;
  --fi-component-lineup-dot-size: 84rpx;
```

- [ ] **Step 2: 样式审计测试**：复制 `scripts/seat-swap-style-audit.test.mjs`，把 `ROOT` 指向 `../src/pages/lineup`，文件顶部辅助函数同名，`AUDITED_FILES` 设为 `'ALL'`（先 RED：目录下还没有 `.vue` 时只跑辅助函数测试，组件加入后逐个纳入）。草坪、线条的色值只能引用上面的 token（不再有 decorative 例外）。

- [ ] **Step 3: `LineupPlayerDot.vue`**

结构：圆形头像（`image`，无头像时显示号码）+ 右下角号码角标 + 下方姓名（单行省略）；`former` 时左上角"前"字角标；`player` 为 `null` 时显示虚线圆、"+"与位置名（`label`）；`selected` 时加描边；`compact` 用于分享图预览（本阶段不用，但保留 prop 以便阶段三复用）。尺寸 `--fi-component-lineup-dot-size`；颜色用 `--fi-color-*`；"前"角标用 `--fi-color-primary` 底白字、`--fi-font-20`、`--fi-weight-bold`。

- [ ] **Step 4: `LineupPitch.vue`（拖动核心）**

```vue
<template>
  <view class="pitch">
    <view class="pitch__field" id="lineup-pitch-field">
      <!-- 草坪、边线、中线、中圈、禁区：用普通 view + border，颜色全部引用 token -->
      <view class="pitch__stripe" v-for="n in 8" :key="n" :class="{ 'pitch__stripe--alt': n % 2 === 0 }" />
      <view class="pitch__line pitch__line--outline" />
      <view class="pitch__line pitch__line--half" />
      <view class="pitch__circle" />
      <view class="pitch__box pitch__box--top" />
      <view class="pitch__box pitch__box--bottom" />

      <movable-area class="pitch__area" :scale-area="false">
        <movable-view
          v-for="p in players"
          :key="p.slot"
          class="pitch__mover"
          direction="all"
          :x="position[p.slot]?.x ?? 0"
          :y="position[p.slot]?.y ?? 0"
          :inertia="false"
          :out-of-bounds="false"
          :damping="40"
          :disabled="readonly"
          @change="(e) => onChange(p.slot, e)"
          @touchstart="() => onStart(p.slot)"
          @touchend="() => onEnd(p.slot)"
          @tap="() => onTap(p.slot)"
        >
          <LineupPlayerDot :player="p" :selected="selectedSlot === p.slot" />
        </movable-view>
      </movable-area>
    </view>
  </view>
</template>
```

脚本要点（实现者据此写，行为必须满足）：
1. **测量**：`onMounted` / 数据变化后用 `uni.createSelectorQuery().in(instance).select('#lineup-pitch-field').boundingClientRect()` 取球场区域宽高（**px**，因为该接口与 `movable-view` 的 `x`/`y` 只接受 px；在注释里写明），并取 `uni.getWindowInfo().windowWidth` 把 px 换算成 rpx：`rpx = px * 750 / windowWidth`。
2. **位置**：`position[slot] = { x: p.x * areaWidthPx - dotPx / 2, y: p.y * areaHeightPx - dotPx / 2 }`（`movable-view` 的 x/y 是左上角，圆点中心坐标要减半径）。圆点 px 尺寸由 `--fi-component-lineup-dot-size`（84rpx）换算（`84 * windowWidth / 750`）。
3. **拖动中**：`@change`（`source === 'touch'`）只更新一个**非响应式**的 `live[slot] = { x, y }`，**不要**在拖动中写回 `position`（否则会和原生拖动互相打架导致抖动）。
4. **松手**（`touchend`）：用 `live` 值算出圆点中心的 rpx 坐标，调用纯函数 `resolveDragRelease`（`fromX/fromY` 取按下时的中心位置，`toX/toY` 取 `live` 的中心位置，`dotDiameterRpx = 84`，`slots` 由 `players` 还原），再 `emit('drag', slot, outcome)`，并清空 `live`。若 `outcome.kind === 'tap'` 不触发 `drag`，由 `tap` 事件负责。
5. **防重复弹层**：`movable-view` 在拖动后仍可能触发 `tap`。用一个 `dragged` 标记：`touchend` 判定结果非 `tap` 时置 `dragged=true` 并在下一个 tick 清除；`onTap` 若 `dragged` 为真则忽略，否则 `emit('tap', slot)`。
6. **父组件更新后复位**：`drag` 事件处理完（互换 / 移动）→ 父组件更新 `players` → `position` 重新计算 → `movable-view` 的 `x`/`y` 变化触发其自带过渡动画（互换时即是 220ms 平滑移动）。注意：当松手落点与重新计算出的位置相同（纯 move）时 `x`/`y` 值与原生位置一致，不会二次跳动。
7. `readonly` 为真时 `disabled`，且不 emit。
8. 整个组件只依赖 `uni`（测量），不 import api/store；`#ifdef H5` 下 `movable-view` 由 uni-app 模拟，不做额外处理（H5 不在本期验证范围）。

样式：`.pitch__field` 宽 100%，`padding-top: var(--fi-component-lineup-pitch-ratio)` 做 3:4 比例（用 `position: relative` + 绝对定位的内容层）；草坪用 8 条纯色横条交替 `--fi-color-lineup-grass` / `-alt`（**不得渐变**）；线条用 `--fi-color-lineup-line`、`--fi-border-width`。

- [ ] **Step 5: `LineupPlayerSheet.vue`、`LineupFormationBar.vue`、`LineupTeamPicker.vue`**

- `LineupPlayerSheet`：基于 `FiBottomSheet`（`fitContent`、`height="80vh"`）；标题为 `slotLabel`（如"左后卫"）；`occupied` 为真时顶部有"移出"按钮（`emit('remove')`）；按 `groups` 渲染——"现役"分组直接列出，"前队员"分组默认折叠（点标题行展开，带 `chevron-down` 图标旋转动画，复用 `src/static/seat-swap/chevron-down.svg`）；每行：头像、号码、姓名、位置标签、前队员显示"{last_season} 赛季在队"；`onPitch` 的行置灰并标"已上场"，仍可点（点选即互换，行为由页面处理）；`preferred` 的行不加额外样式（排序已体现）。
- `LineupFormationBar`：横向 `scroll-view`，7 个胶囊（样式与洞察页球队胶囊一致：选中墨色底白字，未选中 `--fi-color-page` 底），选中项居中滚动（沿用 `InsightsTeamSelector.vue` 里 `centerSelectedChip` 的做法，但不 import 它）。
- `LineupTeamPicker`：球队胶囊行 + 点击弹出选择（本期简化为横向滚动胶囊，16 队）。

- [ ] **Step 6: 验证**

```bash
cd football_insight_mini
bun test scripts/lineup-style-audit.test.mjs 2>&1 | tail -3
bun test 2>&1 | grep -E "^ *[0-9]+ (pass|fail)"
bun run type-check 2>&1 | tail -2
bun run check:boundaries | tail -1
bun run verify:mp-components 2>&1 | tail -1
```

Expected: 全部通过；审计无 gradient / hex / 数字字重。（此时页面尚未注册进 `pages.json`，`verify:mp-components` 只检查已注册页面，组件在 Task 8 页面接入后再随构建验证。）

- [ ] **Step 7: 提交**

```bash
git pull --rebase
git add football_insight_mini/src/styles/fi-tokens.css football_insight_mini/scripts/lineup-style-audit.test.mjs football_insight_mini/src/pages/lineup/components
git commit -m "feat(mini): add lineup pitch with native drag, player sheet and formation bar

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 8: 小程序 —— 编辑页、分包与入口

**Files:**
- Create: `football_insight_mini/src/pages/lineup/index.vue`
- Modify: `football_insight_mini/src/pages.json`、`src/pages/insights/index.vue`、首页主队卡所在组件（见 Step 4）

**Interfaces:**
- Consumes: Task 6 的 `useLineupPage`；Task 7 的全部组件；`FiSecondaryNav`、`FiBrandNav` 之外的页面模式与 `pages/team-board/index.vue` 的外壳（`page-meta`、背景图、`FiSecondaryNav`）。
- Produces: 页面路径 `/pages/lineup/index?teamId=<id>`（`teamId` 缺省取主队，再缺省取第一支队）。

- [ ] **Step 1: 分包与预加载（`pages.json`）**

在 `subPackages` 追加：

```json
		{
			"root": "pages/lineup",
			"pages": [
				{
					"path": "index",
					"style": {
						"navigationBarTitleText": "战术板",
						"navigationStyle": "custom"
					}
				}
			]
		}
```

`preloadRule` 里为 `pages/insights/index` 与 `pages/home/index` 追加预加载 `pages/lineup`（`packages` 数组加入，不覆盖现有 `pages/ticket-watch`）。

- [ ] **Step 2: 页面 `index.vue`**

结构与交互严格按 spec 第 4.2–4.4 节：二级导航（`FiSecondaryNav`，`@back` 返回，分享按钮本阶段 `shareable=false`）→ 球队胶囊 → 阵型条 → `LineupPitch` → 底部操作（重置站位 / 保存 / 分享占位）。要点：
- `onLoad(query)` 解析 `teamId`；球队列表用现有 `listSupportTeams()`（`api/support.ts`，返回 16 队）；默认队 = `teamId` 参数 → 用户主队（`getSupportProfile().favorite_team` 已登录时）→ 列表第一队。
- 切换球队：若当前有未保存改动，弹确认；确认后 `load(newTeamId)`。
- 点圆点：`selectedSlot = slot`，打开 `LineupPlayerSheet`（`buildSheetGroups(squad, slots, position)`）；`pick(playerId)` → `editor.pick`；`remove` → `editor.pick(slot, null)`；选中后关闭弹层。
- `LineupPitch` 的 `@drag` → `editor.applyDrag`。
- 阵型条 `v-model` 变化 → `editor.setFormation(next, positionIndex(squad))`。
- 保存按钮：`save()` 返回 `need-login` → `rememberPostLoginRedirect({ type: 'navigateTo', url: '/pages/lineup/index?teamId=<id>' })` 后 `uni.switchTab({ url: '/pages/user/index' })`（与 `insights` 的 `goToLogin` 同款）；`saved` → `uni.showToast({ title: '已保存' })`；`failed` → 提示"保存失败，已存为草稿，稍后重试"。
- 保存状态文案：`dirty` → "未保存"，`saved` → "已保存"，`error` → "保存失败"。
- 离开确认：`onBackPress`（mp 端为页面生命周期 `onUnload` 无法拦截）——用 `FiSecondaryNav` 的 `@back` 自己处理：有改动时 `uni.showModal` 确认，确认后 `uni.navigateBack()`；系统返回手势不拦截（草稿机制兜底：改动不会丢，因为每次修改都会写入草稿——**`useLineupPage` 在 `dirty` 变化时 debounce 300ms 写草稿**，登录与否都写）。
- 加载中显示 `FiLoading`；失败显示错误卡与"重试"；H5 端：`#ifdef H5` 整个内容替换为一句"请在微信小程序中使用"。
- 分享按钮本阶段放一个禁用的占位（`shareable=false` 时导航栏不显示分享；底部"分享"按钮置灰并提示"即将上线"）。
- 页面所有颜色、间距、字号引用 token；沿用换座页/洞察页的外壳样式与 `page-meta` 导航高度处理（`useNavigationLayout`）。

- [ ] **Step 3: 草稿写入补充测试（`useLineup.test.ts` 追加）**

```ts
  test('edits are written to the draft after the debounce even when logged out', async () => {
    const { page, storage } = build({ isLoggedIn: () => false })
    await page.load(1)
    page.editor.pick(0, 1)
    await new Promise((resolve) => setTimeout(resolve, 400))
    expect(readDraft(1, storage)?.slots[0].player_id).toBe(1)
  })
```

先确认失败再实现（`useLineupPage` 内对 `editor.slots` / `editor.formation` 的 `watch` + `setTimeout` 300ms 写草稿；`dispose()` 清理定时器；`markSaved` 后清草稿）。

- [ ] **Step 4: 入口改造**
  - **洞察页**：`pages/insights/index.vue` 的 `goToTeamBoard()` 改为 `uni.navigateTo({ url: `/pages/lineup/index?teamId=${selectedInsight.value.team_id}` })`；`InsightsSummaryPanel` 里"战术板 ›"文案不变。旧的 `pages/team-board` 页面与 `useTeamBoardPage.ts` 保持不动。
  - **首页主队卡**：找到渲染主队卡的组件（`grep -n "favorite" src/pages/home/index.vue` 定位到 `HomeSupportPanel` 或 `HomeStoryPanel` 中的主队区域），在主队卡片的次要操作区加一个"排我的最佳阵容"入口（图标按钮 + 文案，样式用现有次按钮 token），`@tap` 触发新 emit `open-lineup`，首页 `index.vue` 里 `uni.navigateTo({ url: '/pages/lineup/index?teamId=' + favoriteTeamId })`。无主队时入口不显示。保持该组件现有 props/emit 契约兼容（只新增一个可选 emit）。
  - 入口新增后，更新这两处现有测试（若有对 `goToTeamBoard` 目标页的断言）并补一条纯函数/路径拼装的测试（路径拼装逻辑放 `pages/lineup/helpers.ts` 的 `buildLineupPageUrl(teamId?: number): string` 并加测试：有 id 带 `?teamId=`，无 id 不带参数）。

- [ ] **Step 5: 全量验证与构建**

```bash
cd football_insight_mini
bun test 2>&1 | grep -E "^ *[0-9]+ (pass|fail)"
bun run type-check 2>&1 | tail -2
bun run check:boundaries | tail -1
bun run build:mp-weixin > /tmp/lineup-build.log 2>&1; echo build=$?; grep -E "DONE|registered|Unregistered|kept manifest" /tmp/lineup-build.log
bun run verify:mp-components | tail -1
ls dist/build/mp-weixin/pages/lineup/ | head
du -sh dist/build/mp-weixin | tail -1
```

Expected: 全部通过；构建产物里存在 `pages/lineup/index.*`（在分包目录），`manifest` 版本号未变，组件注册检查通过；记录完整包与主包大小（汇报写明主包增量）。

- [ ] **Step 6: 提交**

```bash
git pull --rebase
git add football_insight_mini/src
git commit -m "feat(mini): add the lineup editor page, subpackage and entries

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 9: 文档

**Files:** `CHANGELOG.md`、`FEATURES.md`

- [ ] **Step 1**: `CHANGELOG.md` 顶部加"战术板阶段二：阵容编辑与保存（待上传，生产后端待部署）"，写明：新增表 `f_i_lineups` 与 `GET/PUT /api/v1/lineups/me/{team_id}`；小程序新分包 `pages/lineup`（7 个阵型、点位置选人含前队员、拖动微调与互换、保存、草稿、洞察页与首页入口）；原因（"我心中的最佳阵容"，保留前队员以引发"如果没卖他"的讨论，粘性与分享的基础）；验证数字（后端与前端测试数、一次性库测试数、构建主包增量）；发布状态逐项（本地后端已重启 / 生产迁移是否已执行 / 生产后端是否已部署 / 小程序是否已上传）。
- [ ] **Step 2**: `FEATURES.md`"进行中"里的战术板条目改为"阶段二编辑与保存已完成，阶段三分享待开发"。
- [ ] **Step 3: 提交**

```bash
git pull --rebase
git add CHANGELOG.md FEATURES.md
git commit -m "docs: record lineup board phase two editor

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
git push
```

---

### Task 10: 生产迁移与后端部署（需负责人确认后执行）

此任务有生产副作用：**执行前必须停下来询问负责人**，并写明将要做的事。

- [ ] **Step 1: 询问负责人**：是否在 peiqian 执行迁移 `20261019130000_add_lineups.sql`（只新建一张表）、并按现有 Docker 约定部署生产后端（local233 构建镜像 → `docker save | gzip | ssh peiqian 'gunzip | docker load'` → 用 `deploy_peiqian_docker.sh` 里的 `docker run` 参数重启容器，保留上一个镜像 tag 便于回滚；**不要**直接运行该脚本、**不要**在 peiqian 上 `docker build`）。迁移必须先于新后端发布。

- [ ] **Step 2（同意后）: 迁移**

```bash
ssh peiqian "cd /root/projects/football_insight && zsh -ic 'proxyOn >/dev/null 2>&1 || true; git -c http.version=HTTP/1.1 pull --ff-only' && cd football_insight_service_backend_rs && set -a && . ../football-insight-service-backend-rs.env && set +a && cargo run --release --bin run_migrations 2>&1 | tail -3"
```

Expected: `migrations applied successfully`；`_sqlx_migrations` 中 `20261019130000` 成功。

- [ ] **Step 3（同意后）: 构建并部署后端镜像**：按 `build-images-on-local233` 约定在 local233 构建（镜像 tag 用 `git rev-parse --short HEAD`），传到 peiqian，沿用 `deploy_peiqian_docker.sh` 的运行参数（`--network host`、`--env-file /root/projects/football_insight/football-insight-service-backend-rs.env`、`-e PORT=8092`、日志卷）重启 `football-insight-service-backend-rs` 容器，保留上一个 tag；确认容器 healthy。

- [ ] **Step 4: 线上验证（只读 + 一个测试账号写入前先问）**

```bash
curl -s -m 10 -o /dev/null -w "squads=%{http_code}\n" https://match.oryjk.cn/api/v1/lineups/squads/77680
curl -s -m 10 -o /dev/null -w "me-unauth=%{http_code}\n" https://match.oryjk.cn/api/v1/lineups/me/77680
```

Expected: `squads=200`、`me-unauth=401`。带登录的保存与读取验证需要登录令牌：**不要用真实用户的令牌**；改为让负责人在体验版里保存一次，或由负责人提供测试账号令牌后再验证；验证完成后核对 `f_i_lineups` 行数与 `share_code` 格式（只读查询）。

- [ ] **Step 5: 更新 `CHANGELOG.md` / `FEATURES.md` 的发布状态**（迁移已执行、生产后端已部署的镜像 tag、回滚 tag），提交推送。
