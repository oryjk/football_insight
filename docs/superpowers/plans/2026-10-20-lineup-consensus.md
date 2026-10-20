# 排阵容：球迷共识阵容 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 后端按球队算出"大家心中的最佳 11 人"（满 10 套公开阵容才解锁），洞察页"排阵容"展示共识卡片与"你和大家重合 N / 11"，别人的阵容详情页显示重合度并标出"独家"球员。

**Architecture:** 共识算法是 `lineup/domain.rs` 的纯函数 `compute_consensus`；`PublicLineupRepository` 新增读取球队全部公开阵容球员 ID 的查询；`PublicLineupService::consensus` 编排；新增公开路由 `GET /api/v1/lineups/teams/{team_id}/consensus`。小程序把阵型坐标 `formations.ts` 移到 `utils/lineupFormations.ts`（原位置保留转发导出），新增共识纯函数与加载器，洞察页新增 `InsightsConsensusCard`，详情页给只读球场传"独家"球员。

**Tech Stack:** Rust 2024 / Axum 0.8 / SQLx 0.8；uni-app (Vue 3 + TS) / Bun test。

**Spec:** `docs/superpowers/specs/2026-10-20-lineup-consensus-design.md`

## Global Constraints

- 界面风格与现有洞察页、战术板一致；组件化（组件只收属性、发事件，调接口与跳转放页面或组合函数，`bun run check:boundaries`）；token 化（`fi-tokens.css`；全局 `style-token-audit` 与 `lineup-style-audit` 自动覆盖，不写渐变、散写 hex、数字字重）。
- 后端六边形：只改 `lineup` 模块的 `domain` / `ports` / `application` / `adapters`；算法是 domain 纯函数。
- 只做微信小程序，不处理 `football_insight_h5`，不做 H5 构建。
- 不新增表、无迁移；不在生产库做写入测试；SQL 测试用一次性本地库（`LINEUP_TEST_DATABASE_URL`，localhost、库名以 `_test` 结尾）。
- 主包距 2MB 约 410KB：`InsightsConsensusCard`、`utils/lineupConsensus.ts`、`utils/lineupFormations.ts`、`composables/useLineupConsensus.ts` 在主包，保持精简；详情页改动在战术板分包。
- **交互与美观（负责人要求）**：共识卡片不是静态展示——未解锁时进度条有填充动画并配"去排一套"按钮；已解锁时我选了的球员红色描边、没选的变淡，点球员有"78% 的球迷选了他（你也选了）"提示；卡片高度要克制（洞察页球场比编辑页扁）；动画只动 `width`/`opacity`/`transform`，时长与缓动用 `--fi-motion-*` token；尊重 `prefers-reduced-motion`。完成后在真机核对观感，不只看构建通过。
- `watch` 多个依赖时用多数据源数组写法 `watch([a, () => b], …)`，不要用"返回新数组的单个 getter"（会在任一依赖触发时重复执行，之前出过按钮闪动问题）。
- 后端只用 `rustfmt --edition 2024 <本次改动的文件>`，**不要格式化 `src/lib.rs`**，不要 `cargo fmt`。
- 不部署、不上传小程序、不改 `manifest.json` 版本号（构建后被改动就 `git checkout --` 还原 `manifest.json` 与 `src/config/generatedMiniProgramVersion.ts`）；不读不改 `football_insight_admin_android`；不打印、不提交 `.env` 与密钥。
- 提交信息：英文 subject，`feat(lineup): …` / `feat(mini): …` / `docs: …`，结尾 Co-Authored-By 行。

## Review Focus

1. 样本刚好 9 套与 10 套——9 套未解锁只返回进度，10 套解锁。（Task 1 测试 `consensus_unlocks_at_the_threshold`）
2. 球迷普遍把后卫摆到中场位置，按名单位置归类后某类人数不够——补齐后仍恰好 11 人，不重复、不缺位。（Task 1 测试 `shortages_are_filled_with_the_most_picked_remaining_players`）
3. 共识里的球员已从名单中消失（同步后被移除）——仍返回 11 人，名字为"未知球员"，不报错。（Task 2 测试 `missing_squad_rows_still_yield_eleven_players`）
4. 停用账号或不足 11 人的阵容——不计入样本。（Task 2 集成测试 `consensus_counts_only_public_full_lineups`）
5. 切换球队时旧球队的共识响应迟到——不能覆盖新球队；共识加载失败不影响页面其他内容。（Task 3 测试 `stale_team_responses_are_dropped`）

---

## File Structure

| 文件 | 职责 |
|---|---|
| `football_insight_service_backend_rs/src/lineup/domain.rs` | `CONSENSUS_MIN_LINEUPS`、`formation_needs`、`compute_consensus`、`percent`、类型 |
| `…/src/lineup/ports.rs` | `PublicLineupRepository::team_lineup_players` |
| `…/src/lineup/application/public_lineup.rs` | `PublicLineupService::consensus` + 假实现 |
| `…/src/lineup/adapters/public_persistence.rs` | SQL |
| `…/src/lineup/adapters/public_dto.rs`、`public_web.rs` | 响应与路由 |
| `…/tests/lineup_public_postgres.rs` | 集成测试 |
| `football_insight_mini/src/utils/lineupFormations.ts` | 由 `pages/lineup/formations.ts` 移入 |
| `football_insight_mini/src/pages/lineup/formations.ts` | 转发导出 |
| `src/types/lineup.ts`、`src/api/lineup.ts` | 共识类型与接口 |
| `src/utils/lineupConsensus.ts`(+test) | 重合度、独家、进度文案、摆位 |
| `src/composables/useLineupConsensus.ts`(+test) | 按球队加载共识 |
| `src/pages/insights/components/InsightsConsensusCard.vue` | 共识卡片 |
| `src/pages/insights/components/InsightsMyTacticsCard.vue`、`InsightsTacticsPanel.vue`、`src/pages/insights/index.vue` | 接入与重合度 |
| `src/pages/lineup/components/LineupPlayerDot.vue`、`LineupPitch.vue`、`src/pages/lineup/view.vue` | 详情页重合度与"独家" |

---

### Task 1: 后端 domain——共识算法

**Files:**
- Modify: `football_insight_service_backend_rs/src/lineup/domain.rs`

**Interfaces:**
- Produces: `pub const CONSENSUS_MIN_LINEUPS: usize = 10;`、`pub fn formation_needs(formation: &str) -> Option<[usize; 4]>`（顺序 `[GK, DF, MF, FW]`）、`pub struct ConsensusPick { pub player_id: i64, pub group: Position, pub picks: usize }`、`pub struct Consensus { pub total: usize, pub formation: Option<String>, pub formation_picks: usize, pub players: Vec<ConsensusPick> }`、`pub fn compute_consensus(lineups: &[(String, Vec<i64>)], positions: &HashMap<i64, Option<Position>>) -> Consensus`、`pub fn percent(part: usize, total: usize) -> u8`。

- [ ] **Step 1: 写失败测试**

在 `domain.rs` 现有 `#[cfg(test)] mod tests` 内追加（若该模块没有 `use std::collections::HashMap;` 则在测试函数内引入）：

```rust
    fn consensus_positions() -> std::collections::HashMap<i64, Option<Position>> {
        use Position::*;
        let mut map = std::collections::HashMap::new();
        for id in [1, 2] { map.insert(id, Some(GK)); }
        for id in 10..=15 { map.insert(id, Some(DF)); }
        for id in 20..=25 { map.insert(id, Some(MF)); }
        for id in 30..=33 { map.insert(id, Some(FW)); }
        map.insert(40, None);
        map
    }
    const BASE: [i64; 11] = [1, 10, 11, 12, 13, 20, 21, 22, 30, 31, 32];
    fn lineups(count: usize, formation: &str) -> Vec<(String, Vec<i64>)> {
        (0..count).map(|_| (formation.to_string(), BASE.to_vec())).collect()
    }

    #[test]
    fn formation_needs_split_every_formation_into_ten_outfield_players() {
        for formation in FORMATIONS {
            let needs = formation_needs(formation).unwrap();
            assert_eq!(needs[0], 1, "{formation}");
            assert_eq!(needs[1] + needs[2] + needs[3], 10, "{formation}");
        }
        assert_eq!(formation_needs("4-2-3-1"), Some([1, 4, 5, 1]));
        assert_eq!(formation_needs("3-5-2"), Some([1, 3, 5, 2]));
        assert_eq!(formation_needs("4-1-4-1"), Some([1, 4, 5, 1]));
        assert_eq!(formation_needs("9-9"), None);
    }

    #[test]
    fn consensus_unlocks_at_the_threshold() {
        let positions = consensus_positions();
        let locked = compute_consensus(&lineups(9, "4-3-3"), &positions);
        assert_eq!((locked.total, locked.formation.clone(), locked.players.len()), (9, None, 0));
        let unlocked = compute_consensus(&lineups(10, "4-3-3"), &positions);
        assert_eq!(unlocked.formation.as_deref(), Some("4-3-3"));
        assert_eq!(unlocked.formation_picks, 10);
        assert_eq!(unlocked.players.iter().map(|p| p.player_id).collect::<Vec<_>>(), BASE.to_vec());
        assert!(unlocked.players.iter().all(|p| p.picks == 10));
    }

    #[test]
    fn formation_and_player_ties_are_broken_deterministically() {
        let positions = consensus_positions();
        let mut sample = lineups(5, "4-4-2");
        sample.extend(lineups(5, "4-3-3"));
        // 后卫 13 与 14 各被选 5 次：取 player_id 小的 13
        for (index, lineup) in sample.iter_mut().enumerate() {
            if index % 2 == 0 { lineup.1[4] = 14; }
        }
        let consensus = compute_consensus(&sample, &positions);
        assert_eq!(consensus.formation.as_deref(), Some("4-3-3")); // 平票按 FORMATIONS 顺序
        let df: Vec<i64> = consensus.players.iter().filter(|p| p.group == Position::DF).map(|p| p.player_id).collect();
        assert_eq!(df, vec![10, 11, 12, 13]);
    }

    #[test]
    fn players_are_grouped_by_squad_position_not_by_placement() {
        let positions = consensus_positions();
        // 每套阵容都选了中场 23，而没有选前锋 32：前锋只剩 30、31 两人，需要补 1 人
        let mut sample = lineups(10, "4-3-3");
        for lineup in &mut sample { lineup.1[10] = 23; }
        let consensus = compute_consensus(&sample, &positions);
        assert_eq!(consensus.players.len(), 11);
        let mf: Vec<i64> = consensus.players.iter().filter(|p| p.group == Position::MF).map(|p| p.player_id).collect();
        assert_eq!(mf, vec![20, 21, 22]);
        // 中场 23 虽然被选 10 次，但中场已满，只能作为补齐进入前锋位
        let fw: Vec<(i64, Position)> = consensus.players.iter().filter(|p| p.group == Position::FW).map(|p| (p.player_id, p.group)).collect();
        assert_eq!(fw, vec![(23, Position::FW), (30, Position::FW), (31, Position::FW)]);
    }

    #[test]
    fn shortages_are_filled_with_the_most_picked_remaining_players() {
        let positions = consensus_positions();
        // 3-5-2 需要 5 名中场，但大家只选了 3 名中场，其余是 2 名后卫 14、15 与未知位置 40
        let sample: Vec<(String, Vec<i64>)> = (0..10)
            .map(|index| ("3-5-2".to_string(), vec![1, 10, 11, 12, 14, 15, 20, 21, 22, if index < 6 { 40 } else { 13 }, 30]))
            .collect();
        let consensus = compute_consensus(&sample, &positions);
        let ids: std::collections::HashSet<i64> = consensus.players.iter().map(|p| p.player_id).collect();
        assert_eq!(consensus.players.len(), 11);
        assert_eq!(ids.len(), 11, "no duplicates");
        let groups = |g: Position| consensus.players.iter().filter(|p| p.group == g).count();
        assert_eq!([groups(Position::GK), groups(Position::DF), groups(Position::MF), groups(Position::FW)], [1, 3, 5, 2]);
        // 前锋只有 30 一人被选，第二个前锋位由剩余最热门的补上
        assert!(ids.contains(&30));
    }

    #[test]
    fn percentages_round_to_whole_numbers() {
        assert_eq!(percent(2, 3), 67);
        assert_eq!(percent(1, 3), 33);
        assert_eq!(percent(10, 10), 100);
        assert_eq!(percent(0, 0), 0);
    }
```

- [ ] **Step 2: 运行确认失败**

Run: `cd football_insight_service_backend_rs && cargo test --lib lineup::domain 2>&1 | tail -5`
Expected: 编译失败，`formation_needs`、`compute_consensus`、`percent` 未定义。

- [ ] **Step 3: 实现**

在 `domain.rs` 中 `FORMATIONS` 常量之后加入（文件顶部已有 `use std::collections::HashSet;`，需再引入 `HashMap`）：

```rust
use std::collections::HashMap;

/// 共识阵容的最少公开阵容数；不足时只返回进度。
pub const CONSENSUS_MIN_LINEUPS: usize = 10;

/// 阵型的位置需求 [门将, 后卫, 中场, 前锋]：第一个数字为后卫，最后一个为前锋，中间之和为中场。
pub fn formation_needs(formation: &str) -> Option<[usize; 4]> {
    if !FORMATIONS.contains(&formation) {
        return None;
    }
    let parts: Vec<usize> = formation
        .split('-')
        .map(|part| part.parse().ok())
        .collect::<Option<Vec<usize>>>()?;
    if parts.len() < 3 {
        return None;
    }
    let mf = parts[1..parts.len() - 1].iter().sum();
    Some([1, parts[0], mf, *parts.last()?])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusPick {
    pub player_id: i64,
    /// 在共识中占用的位置类；补齐的球员可能与名单位置不同。
    pub group: Position,
    pub picks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consensus {
    pub total: usize,
    pub formation: Option<String>,
    pub formation_picks: usize,
    pub players: Vec<ConsensusPick>,
}

pub fn percent(part: usize, total: usize) -> u8 {
    if total == 0 {
        return 0;
    }
    ((part as f64) * 100.0 / (total as f64)).round() as u8
}

/// 共识阵容：取最常用阵型，按名单位置在每类位置里取被选次数最多的球员（平票按 player_id），
/// 某类不够时用剩余最热门的球员补齐，结果恰为 11 人。不足门槛时 formation 为 None、players 为空。
pub fn compute_consensus(
    lineups: &[(String, Vec<i64>)],
    positions: &HashMap<i64, Option<Position>>,
) -> Consensus {
    let total = lineups.len();
    let locked = Consensus { total, formation: None, formation_picks: 0, players: Vec::new() };
    if total < CONSENSUS_MIN_LINEUPS {
        return locked;
    }
    let mut best: Option<(&str, usize)> = None;
    for formation in FORMATIONS {
        let count = lineups.iter().filter(|(f, _)| f.as_str() == formation).count();
        if count > 0 && best.is_none_or(|(_, top)| count > top) {
            best = Some((formation, count));
        }
    }
    let Some((formation, formation_picks)) = best else {
        return locked;
    };
    let Some(needs) = formation_needs(formation) else {
        return locked;
    };
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for (_, players) in lineups {
        for id in players {
            *counts.entry(*id).or_default() += 1;
        }
    }
    let mut ranked: Vec<(i64, usize)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    const GROUPS: [Position; 4] = [Position::GK, Position::DF, Position::MF, Position::FW];
    let mut chosen: Vec<ConsensusPick> = Vec::with_capacity(11);
    let mut used: HashSet<i64> = HashSet::new();
    let mut shortage = [0usize; 4];
    for (index, group) in GROUPS.iter().enumerate() {
        let mut taken = 0;
        for (id, picks) in &ranked {
            if taken == needs[index] {
                break;
            }
            if positions.get(id).copied().flatten() == Some(*group) && used.insert(*id) {
                chosen.push(ConsensusPick { player_id: *id, group: *group, picks: *picks });
                taken += 1;
            }
        }
        shortage[index] = needs[index] - taken;
    }
    for (index, group) in GROUPS.iter().enumerate() {
        for _ in 0..shortage[index] {
            if let Some((id, picks)) = ranked.iter().find(|(id, _)| !used.contains(id)) {
                used.insert(*id);
                chosen.push(ConsensusPick { player_id: *id, group: *group, picks: *picks });
            }
        }
    }
    chosen.sort_by(|a, b| {
        Position::rank(Some(a.group))
            .cmp(&Position::rank(Some(b.group)))
            .then(b.picks.cmp(&a.picks))
            .then(a.player_id.cmp(&b.player_id))
    });
    Consensus { total, formation: Some(formation.to_string()), formation_picks, players: chosen }
}
```

注：`Option::is_none_or` 需要 Rust 1.82+；若编译器较旧，改为 `best.map_or(true, |(_, top)| count > top)`，并在进度文件记一行 Ruling。

测试 `players_are_grouped_by_squad_position_not_by_placement` 的期望顺序依赖输出排序（同组内按被选次数降序、再按 `player_id` 升序）：前锋组里 23（10 次）、30（10 次）、31（10 次）按 `player_id` 升序为 23、30、31。

- [ ] **Step 4: 运行确认通过**

Run: `cargo test --lib lineup::domain 2>&1 | grep -E "test result|FAILED"`
Expected: `ok`，新增 6 个测试全部通过。

- [ ] **Step 5: 格式化并提交**

```bash
rustfmt --edition 2024 src/lineup/domain.rs
git add src/lineup/domain.rs
git commit -m "feat(lineup): compute the fan consensus lineup"
```

---

### Task 2: 后端——端口、SQL、服务与路由

**Files:**
- Modify: `football_insight_service_backend_rs/src/lineup/ports.rs`、`application/public_lineup.rs`、`adapters/public_persistence.rs`、`adapters/public_dto.rs`、`adapters/public_web.rs`
- Modify: `football_insight_service_backend_rs/tests/lineup_public_postgres.rs`

**Interfaces:**
- Consumes: Task 1 全部。
- Produces: `PublicLineupRepository::team_lineup_players(&self, team_id: i64) -> Result<Vec<(String, Vec<i64>)>, LineupPublicError>`；`pub struct ConsensusPlayer { pub player: SquadPlayer, pub group: Position, pub pick_rate: u8 }`、`pub struct LineupConsensus { pub team: SquadTeam, pub total: usize, pub threshold: usize, pub formation: Option<String>, pub formation_share: u8, pub players: Vec<ConsensusPlayer> }`（放在 `domain.rs`）；`PublicLineupService::consensus(&self, team_id: i64) -> Result<LineupConsensus, LineupPublicError>`。
- HTTP：`GET /api/v1/lineups/teams/{team_id}/consensus`（匿名，`no-store`），字段见 spec 第 5 节：`team{team_id,team_name,avatar_url}`、`total`、`threshold`、`unlocked`、`formation`、`formation_share`、`players[{player_id,name,avatar_url,shirt_number,position,group,pick_rate}]`。

- [ ] **Step 1: 写服务与路由的失败测试**

`application/public_lineup.rs` 测试模块：
1. `FakePublicLineups` 增加字段 `pub team_lineups: Mutex<Vec<(String, Vec<i64>)>>`，并实现新方法：

```rust
        async fn team_lineup_players(&self, _: i64) -> Result<Vec<(String, Vec<i64>)>, LineupPublicError> {
            Ok(self.team_lineups.lock().unwrap().clone())
        }
```

2. 新增测试（`player(...)` 辅助函数来自 `application::tests`，签名 `player(id, number, position, season, active)`）：

```rust
    fn consensus_service(lineups: Vec<(String, Vec<i64>)>, squad: Vec<SquadPlayer>) -> PublicLineupService {
        let repo = FakePublicLineups::default();
        *repo.team_lineups.lock().unwrap() = lineups;
        PublicLineupService::new(
            Arc::new(FakeSquads { players: squad, missing_table: false }),
            Arc::new(repo),
            Arc::new(FakeUserAccess { active: true, unavailable: false }),
            Arc::new(FakeCodes),
        )
    }
    fn full_squad() -> Vec<SquadPlayer> {
        use crate::lineup::application::tests::player;
        let mut squad = vec![player(1, Some(1), Some(Position::GK), 2026, true)];
        squad.extend((10..=13).map(|id| player(id, Some(id as i32), Some(Position::DF), 2026, true)));
        squad.extend((20..=22).map(|id| player(id, Some(id as i32), Some(Position::MF), 2026, true)));
        squad.extend((30..=32).map(|id| player(id, Some(id as i32), Some(Position::FW), 2026, true)));
        squad
    }
    const XI: [i64; 11] = [1, 10, 11, 12, 13, 20, 21, 22, 30, 31, 32];

    #[tokio::test]
    async fn consensus_reports_progress_below_the_threshold_and_players_above_it() {
        let locked = consensus_service(vec![("4-3-3".into(), XI.to_vec()); 3], full_squad()).consensus(77680).await.unwrap();
        assert_eq!((locked.total, locked.threshold, locked.formation.clone(), locked.players.len()), (3, 10, None, 0));
        let mut sample = vec![("4-3-3".to_string(), XI.to_vec()); 9];
        sample.push(("4-4-2".into(), XI.to_vec()));
        let unlocked = consensus_service(sample, full_squad()).consensus(77680).await.unwrap();
        assert_eq!(unlocked.formation.as_deref(), Some("4-3-3"));
        assert_eq!(unlocked.formation_share, 90);
        assert_eq!(unlocked.players.len(), 11);
        assert_eq!(unlocked.players[0].player.player_id, 1);
        assert_eq!(unlocked.players[0].pick_rate, 100);
    }

    #[tokio::test]
    async fn missing_squad_rows_still_yield_eleven_players() {
        let mut squad = full_squad();
        squad.retain(|p| p.player_id != 32);
        let consensus = consensus_service(vec![("4-3-3".into(), XI.to_vec()); 10], squad).consensus(77680).await.unwrap();
        assert_eq!(consensus.players.len(), 11);
        let unknown = consensus.players.iter().find(|p| p.player.player_id == 32).unwrap();
        assert_eq!(unknown.player.name, "未知球员");
    }

    #[tokio::test]
    async fn consensus_validates_the_team() {
        let svc = consensus_service(vec![], full_squad());
        assert_eq!(svc.consensus(0).await.unwrap_err(), LineupPublicError::Invalid);
        assert_eq!(svc.consensus(1).await.unwrap_err(), LineupPublicError::NotFound);
    }
```

（测试模块 `use` 中加入 `crate::lineup::domain::{Position, SquadPlayer}`。）

`adapters/public_web.rs` 测试模块新增：

```rust
    #[tokio::test]
    async fn consensus_is_anonymous_and_validates_the_team() {
        let (status, body) = call(true, "GET", "/api/v1/lineups/teams/77680/consensus", None).await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["team"]["team_id"], 77680);
        assert_eq!(json["unlocked"], false);
        assert_eq!(json["threshold"], 10);
        assert_eq!(json["players"], serde_json::json!([]));
        assert_eq!(call(true, "GET", "/api/v1/lineups/teams/1/consensus", None).await.0, StatusCode::NOT_FOUND);
        assert_eq!(call(true, "GET", "/api/v1/lineups/teams/0/consensus", None).await.0, StatusCode::BAD_REQUEST);
    }
```

- [ ] **Step 2: 运行确认失败**

Run: `cargo test --lib lineup:: 2>&1 | tail -5`
Expected: 编译失败，`team_lineup_players`、`consensus` 未定义。

- [ ] **Step 3: 实现**

`domain.rs` 增加：

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ConsensusPlayer {
    pub player: SquadPlayer,
    pub group: Position,
    pub pick_rate: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineupConsensus {
    pub team: SquadTeam,
    pub total: usize,
    pub threshold: usize,
    pub formation: Option<String>,
    pub formation_share: u8,
    pub players: Vec<ConsensusPlayer>,
}
```

`ports.rs` 的 `PublicLineupRepository` 增加：

```rust
    /// 球队全部公开阵容（与公开列表同一可见条件）的阵型与 11 个球员 ID。
    async fn team_lineup_players(
        &self,
        team_id: i64,
    ) -> Result<Vec<(String, Vec<i64>)>, LineupPublicError>;
```

`application/public_lineup.rs` 增加（`use` 中加入 `CONSENSUS_MIN_LINEUPS, ConsensusPlayer, LineupConsensus, SquadPlayer, compute_consensus, percent`）：

```rust
    pub async fn consensus(&self, team_id: i64) -> Result<LineupConsensus, LineupPublicError> {
        if team_id <= 0 {
            return Err(LineupPublicError::Invalid);
        }
        let team = self
            .squads
            .find_team(team_id)
            .await
            .map_err(|_| LineupPublicError::Unavailable)?
            .ok_or(LineupPublicError::NotFound)?;
        let lineups = self.lineups.team_lineup_players(team_id).await?;
        let squad = self
            .squads
            .list_squad(team_id)
            .await
            .map_err(|_| LineupPublicError::Unavailable)?;
        let positions = squad.iter().map(|p| (p.player_id, p.position)).collect();
        let result = compute_consensus(&lineups, &positions);
        let players = result
            .players
            .iter()
            .map(|pick| ConsensusPlayer {
                // 球员已从名单移除时仍保留位置，用"未知球员"占位，保证 11 人
                player: squad.iter().find(|p| p.player_id == pick.player_id).cloned().unwrap_or(SquadPlayer {
                    player_id: pick.player_id,
                    name: "未知球员".into(),
                    shirt_number: None,
                    position: None,
                    avatar_url: None,
                    last_season: 0,
                    active: false,
                }),
                group: pick.group,
                pick_rate: percent(pick.picks, result.total),
            })
            .collect();
        Ok(LineupConsensus {
            team,
            total: result.total,
            threshold: CONSENSUS_MIN_LINEUPS,
            formation_share: percent(result.formation_picks, result.total),
            formation: result.formation,
            players,
        })
    }
```

`adapters/public_persistence.rs` 的 `impl PublicLineupRepository for PostgresPublicLineupRepository` 增加：

```rust
    async fn team_lineup_players(
        &self,
        team_id: i64,
    ) -> Result<Vec<(String, Vec<i64>)>, LineupPublicError> {
        sqlx::query_as::<_, (String, Vec<i64>)>(&format!(
            "SELECT l.formation,
                ARRAY(SELECT (s->>'player_id')::BIGINT FROM jsonb_array_elements(l.slots) s) AS players
             FROM f_i_lineups l JOIN f_i_users u ON u.id = l.user_id
             WHERE l.team_id = $1 AND {VISIBLE}"
        ))
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)
    }
```

`adapters/public_dto.rs` 增加：

```rust
fn position_code(position: Position) -> &'static str {
    match position {
        Position::GK => "GK",
        Position::DF => "DF",
        Position::MF => "MF",
        Position::FW => "FW",
    }
}

#[derive(Serialize)]
pub(super) struct ConsensusTeamDto { team_id: i64, team_name: String, avatar_url: Option<String> }
#[derive(Serialize)]
pub(super) struct ConsensusPlayerDto {
    player_id: i64,
    name: String,
    avatar_url: Option<String>,
    shirt_number: Option<i32>,
    position: Option<&'static str>,
    group: &'static str,
    pick_rate: u8,
}
#[derive(Serialize)]
pub(super) struct ConsensusDto {
    team: ConsensusTeamDto,
    total: usize,
    threshold: usize,
    unlocked: bool,
    formation: Option<String>,
    formation_share: u8,
    players: Vec<ConsensusPlayerDto>,
}
impl From<LineupConsensus> for ConsensusDto {
    fn from(c: LineupConsensus) -> Self {
        Self {
            team: ConsensusTeamDto { team_id: c.team.team_id, team_name: c.team.team_name, avatar_url: c.team.avatar_url },
            total: c.total,
            threshold: c.threshold,
            unlocked: c.formation.is_some(),
            formation: c.formation,
            formation_share: c.formation_share,
            players: c.players.into_iter().map(|p| ConsensusPlayerDto {
                player_id: p.player.player_id,
                name: p.player.name,
                avatar_url: p.player.avatar_url,
                shirt_number: p.player.shirt_number,
                position: p.player.position.map(position_code),
                group: position_code(p.group),
                pick_rate: p.pick_rate,
            }).collect(),
        }
    }
}
```

（`use` 中加入 `LineupConsensus`、`Position`；文件已有 `Serialize` 引入则复用。）

`adapters/public_web.rs`：`use super::public_dto::{…, ConsensusDto, …}`；在 `public_routes` 中加 `.route("/api/v1/lineups/teams/{team_id}/consensus", get(consensus))`；处理函数：

```rust
async fn consensus(
    State(state): State<Arc<PublicLineupWebState>>,
    Path(team_id): Path<i64>,
) -> Result<Json<ConsensusDto>, WebError> {
    state.service.consensus(team_id).await.map(|c| Json(c.into())).map_err(map_error)
}
```

- [ ] **Step 4: 写集成测试**

`tests/lineup_public_postgres.rs` 末尾追加（`setup`、`user`、`save` 已存在；`save(pool, actor, team, code, full)` 里 `full = false` 会保存不足 11 人的阵容）：

```rust
#[tokio::test]
#[ignore = "requires explicit disposable local LINEUP_TEST_DATABASE_URL"]
async fn consensus_counts_only_public_full_lineups() {
    let db = setup().await;
    let (a, b, c, d) = (user(&db.pool).await, user(&db.pool).await, user(&db.pool).await, user(&db.pool).await);
    save(&db.pool, a, 1, "CONS0001", true).await;
    save(&db.pool, b, 1, "CONS0002", true).await;
    save(&db.pool, c, 1, "CONS0003", false).await;
    save(&db.pool, d, 1, "CONS0004", true).await;
    sqlx::query("UPDATE f_i_users SET status = 'disabled' WHERE id = $1").bind(d).execute(&db.pool).await.unwrap();
    let rows = PostgresPublicLineupRepository::new(db.pool.clone()).team_lineup_players(1).await.unwrap();
    assert_eq!(rows.len(), 2);
    for (formation, players) in rows {
        assert_eq!(formation, "4-3-3");
        let mut sorted = players.clone();
        sorted.sort();
        assert_eq!(sorted, (1..=11).collect::<Vec<i64>>());
    }
    teardown(db).await;
}
```

- [ ] **Step 5: 运行全部后端测试与集成测试**

```bash
cargo test 2>&1 | grep -E "^test result|FAILED|panicked" | tail -8
docker run -d --rm --name consensus-pg -e POSTGRES_PASSWORD=pg -e POSTGRES_DB=lineup_test -p 55496:5432 postgres:16
for i in $(seq 1 30); do docker exec consensus-pg pg_isready -U postgres >/dev/null 2>&1 && break; sleep 1; done
LINEUP_TEST_DATABASE_URL=postgres://postgres:pg@127.0.0.1:55496/lineup_test cargo test --test lineup_public_postgres -- --ignored --test-threads=4 2>&1 | grep -E "^test |test result"
docker stop consensus-pg
```

Expected: 全部 `ok`；集成测试含新增 `consensus_counts_only_public_full_lineups` 全部通过。

- [ ] **Step 6: 格式化并提交**

```bash
rustfmt --edition 2024 src/lineup/domain.rs src/lineup/ports.rs src/lineup/application/public_lineup.rs src/lineup/adapters/public_persistence.rs src/lineup/adapters/public_dto.rs src/lineup/adapters/public_web.rs tests/lineup_public_postgres.rs
git add src/lineup tests/lineup_public_postgres.rs
git commit -m "feat(lineup): expose the fan consensus lineup"
```

---

### Task 3: 小程序——阵型移至公共目录、类型、接口、纯函数与加载器

**Files:**
- Move: `football_insight_mini/src/pages/lineup/formations.ts` → `src/utils/lineupFormations.ts`
- Create: `src/pages/lineup/formations.ts`（转发导出）
- Modify: `src/types/lineup.ts`、`src/api/lineup.ts`
- Create: `src/utils/lineupConsensus.ts`、`src/utils/lineupConsensus.test.ts`
- Create: `src/composables/useLineupConsensus.ts`、`src/composables/useLineupConsensus.test.ts`

**Interfaces:**
- Produces:
  - 类型 `LineupConsensusPlayer { player_id; name; avatar_url: string | null; shirt_number: number | null; position: LineupPosition | null; group: LineupPosition; pick_rate: number }`、`LineupConsensus { team: SquadResponse['team']; total; threshold; unlocked: boolean; formation: string | null; formation_share: number; players: LineupConsensusPlayer[] }`
  - `getLineupConsensus(teamId: number): Promise<LineupConsensus>`
  - `lineupOverlap(playerIds: Array<number | null>, consensus: LineupConsensus | null): number | null`、`exclusivePlayerIds(playerIds, consensus): number[]`、`overlapText(count: number | null, who: '你' | 'TA'): string`、`consensusProgressText(total: number, threshold: number): string`、`placeConsensus(consensus: LineupConsensus): Array<{ slot: number; x: number; y: number; player: LineupConsensusPlayer }>`
  - `createLineupConsensus(read?)` → `{ consensus: Ref<LineupConsensus | null>, load(teamId: number): Promise<void>, reset(): void }`

- [ ] **Step 1: 移动阵型文件并保留转发**

```bash
cd football_insight_mini
git mv src/pages/lineup/formations.ts src/utils/lineupFormations.ts
sed -i "s#from '../../types/lineup'#from '../types/lineup'#" src/utils/lineupFormations.ts
printf "// 阵型坐标已移到 utils，供洞察页（主包）与战术板分包共用；保留此转发以免改动分包内的引用。\nexport * from '../../utils/lineupFormations'\n" > src/pages/lineup/formations.ts
bun test src/pages/lineup 2>&1 | grep -E "^ *[0-9]+ (pass|fail)"
```

Expected: 战术板现有测试全部通过（`formations.test.ts` 经转发导入）。

- [ ] **Step 2: 写纯函数与加载器的失败测试**

`src/utils/lineupConsensus.test.ts`：

```ts
import { describe, expect, test } from 'bun:test'
import type { LineupConsensus } from '../types/lineup'
import { consensusProgressText, exclusivePlayerIds, lineupOverlap, overlapText, placeConsensus } from './lineupConsensus'

const ids = [1, 10, 11, 12, 13, 20, 21, 22, 30, 31, 32]
const consensus: LineupConsensus = {
  team: { team_id: 77680, team_name: '成都蓉城', avatar_url: null },
  total: 36, threshold: 10, unlocked: true, formation: '4-3-3', formation_share: 58,
  players: ids.map((id, index) => ({
    player_id: id, name: `P${id}`, avatar_url: null, shirt_number: id, position: null,
    group: index === 0 ? 'GK' : index <= 4 ? 'DF' : index <= 7 ? 'MF' : 'FW', pick_rate: 90 - index,
  })),
}
const locked: LineupConsensus = { ...consensus, total: 2, unlocked: false, formation: null, formation_share: 0, players: [] }

describe('lineup consensus helpers', () => {
  test('overlap counts shared players and ignores empty slots', () => {
    expect(lineupOverlap(ids, consensus)).toBe(11)
    expect(lineupOverlap([1, 10, 11, 99, 98, null, 20, 21, 22, 30, 31], consensus)).toBe(9)
    expect(lineupOverlap(ids, locked)).toBe(null)
    expect(lineupOverlap(ids, null)).toBe(null)
  })

  test('exclusive players are the ones outside the consensus', () => {
    expect(exclusivePlayerIds([1, 99, null, 10, 98], consensus)).toEqual([99, 98])
    expect(exclusivePlayerIds([99], locked)).toEqual([])
  })

  test('texts', () => {
    expect(overlapText(8, '你')).toBe('你和大家重合 8 / 11')
    expect(overlapText(6, 'TA')).toBe('TA和大家重合 6 / 11')
    expect(overlapText(null, '你')).toBe('')
    expect(consensusProgressText(2, 10)).toBe('已有 2 套阵容，再有 8 套就能看到大家心中的最佳 11 人')
    expect(consensusProgressText(12, 10)).toBe('已有 12 套阵容，再有 0 套就能看到大家心中的最佳 11 人')
  })

  test('players are placed into the consensus formation by group', () => {
    const placed = placeConsensus(consensus)
    expect(placed.length).toBe(11)
    expect(placed.find(entry => entry.player.player_id === 1)).toMatchObject({ x: 0.5, y: 0.9 })
    expect(placed.filter(entry => entry.player.group === 'FW').every(entry => entry.y < 0.3)).toBe(true)
    expect(placeConsensus(locked)).toEqual([])
  })
})
```

`src/composables/useLineupConsensus.test.ts`：

```ts
import { describe, expect, test } from 'bun:test'
import type { LineupConsensus } from '../types/lineup'
import { createLineupConsensus } from './useLineupConsensus'

const sample = (teamId: number, total = 3): LineupConsensus => ({
  team: { team_id: teamId, team_name: `T${teamId}`, avatar_url: null },
  total, threshold: 10, unlocked: false, formation: null, formation_share: 0, players: [],
})

describe('lineup consensus loader', () => {
  test('stale_team_responses_are_dropped', async () => {
    let releaseFirst: (value: LineupConsensus) => void = () => {}
    const loader = createLineupConsensus(id => id === 1 ? new Promise(resolve => { releaseFirst = resolve }) : Promise.resolve(sample(2)))
    const first = loader.load(1)
    await loader.load(2)
    releaseFirst(sample(1)); await first
    expect(loader.consensus.value?.team.team_id).toBe(2)
  })

  test('a failed refresh keeps the same team result; switching teams clears it', async () => {
    let fail = false
    const loader = createLineupConsensus(async id => { if (fail) throw new Error('offline'); return sample(id, 5) })
    await loader.load(1)
    fail = true
    await loader.load(1)
    expect(loader.consensus.value?.total).toBe(5)
    await loader.load(2)
    expect(loader.consensus.value).toBe(null)
    loader.reset()
    expect(loader.consensus.value).toBe(null)
  })
})
```

- [ ] **Step 3: 运行确认失败**

Run: `bun test src/utils/lineupConsensus.test.ts src/composables/useLineupConsensus.test.ts 2>&1 | tail -3`
Expected: 失败，模块不存在。

- [ ] **Step 4: 实现**

`src/types/lineup.ts` 末尾：

```ts
export interface LineupConsensusPlayer {
  player_id: number
  name: string
  avatar_url: string | null
  shirt_number: number | null
  position: LineupPosition | null
  /** 在共识里占用的位置类；补齐的球员可能与名单位置不同 */
  group: LineupPosition
  pick_rate: number
}
export interface LineupConsensus {
  team: SquadResponse['team']
  total: number
  threshold: number
  unlocked: boolean
  formation: string | null
  formation_share: number
  players: LineupConsensusPlayer[]
}
```

`src/api/lineup.ts`（`import type` 中加入 `LineupConsensus`）：

```ts
export function getLineupConsensus(teamId: number): Promise<LineupConsensus> {
  return request({ url: `/lineups/teams/${teamId}/consensus` })
}
```

`src/utils/lineupConsensus.ts`：

```ts
import type { LineupConsensus, LineupConsensusPlayer, LineupPosition } from '../types/lineup'
import { getFormation } from './lineupFormations'

/** 与共识 11 人相同的球员数；共识未解锁时为 null（不显示重合度）。 */
export function lineupOverlap(playerIds: Array<number | null>, consensus: LineupConsensus | null): number | null {
  if (!consensus?.unlocked) return null
  const chosen = new Set(consensus.players.map(player => player.player_id))
  return new Set(playerIds.filter((id): id is number => id !== null && chosen.has(id))).size
}

/** 不在共识里的球员（"独家"）；未解锁时为空。 */
export function exclusivePlayerIds(playerIds: Array<number | null>, consensus: LineupConsensus | null): number[] {
  if (!consensus?.unlocked) return []
  const chosen = new Set(consensus.players.map(player => player.player_id))
  return playerIds.filter((id): id is number => id !== null && !chosen.has(id))
}

export function overlapText(count: number | null, who: '你' | 'TA'): string {
  return count === null ? '' : `${who}和大家重合 ${count} / 11`
}

export function consensusProgressText(total: number, threshold: number): string {
  return `已有 ${total} 套阵容，再有 ${Math.max(0, threshold - total)} 套就能看到大家心中的最佳 11 人`
}

/** 按 group 把共识 11 人依次放进共识阵型的同类位置（坐标沿用阵型定义）。 */
export function placeConsensus(consensus: LineupConsensus): Array<{ slot: number; x: number; y: number; player: LineupConsensusPlayer }> {
  if (!consensus.unlocked || !consensus.formation) return []
  const queues: Record<LineupPosition, LineupConsensusPlayer[]> = { GK: [], DF: [], MF: [], FW: [] }
  for (const player of consensus.players) queues[player.group].push(player)
  return getFormation(consensus.formation).flatMap(slot => {
    const player = queues[slot.position].shift()
    return player ? [{ slot: slot.slot, x: slot.x, y: slot.y, player }] : []
  })
}
```

`src/composables/useLineupConsensus.ts`：

```ts
import { ref } from 'vue'
import { getLineupConsensus } from '../api/lineup'
import type { LineupConsensus } from '../types/lineup'

/** 按球队加载共识阵容：换队后迟到的旧响应丢弃；同队刷新失败保留上次结果（共识是增强信息，不报错）。 */
export function createLineupConsensus(read: (teamId: number) => Promise<LineupConsensus> = getLineupConsensus) {
  const consensus = ref<LineupConsensus | null>(null)
  let generation = 0, current = 0
  async function load(teamId: number): Promise<void> {
    const owner = ++generation
    if (teamId !== current) consensus.value = null
    current = teamId
    if (!teamId) return
    try {
      const result = await read(teamId)
      if (owner === generation) consensus.value = result
    } catch {
      /* 保留同队上次结果 */
    }
  }
  function reset(): void { generation++; current = 0; consensus.value = null }
  return { consensus, load, reset }
}
```

- [ ] **Step 5: 运行确认通过并检查**

Run: `bun test src/utils/lineupConsensus.test.ts src/composables/useLineupConsensus.test.ts src/pages/lineup 2>&1 | grep -E "^\(fail|^ *[0-9]+ (pass|fail)" && bun run type-check 2>&1 | tail -1 && bun run check:boundaries | tail -1`
Expected: 0 fail；类型与边界通过。

- [ ] **Step 6: 提交**

```bash
git add src/utils/lineupFormations.ts src/pages/lineup/formations.ts src/types/lineup.ts src/api/lineup.ts src/utils/lineupConsensus.ts src/utils/lineupConsensus.test.ts src/composables/useLineupConsensus.ts src/composables/useLineupConsensus.test.ts
git commit -m "feat(mini): add lineup consensus api, helpers and loader"
```

---

### Task 4: 洞察页"排阵容"——共识卡片与我的重合度

**Files:**
- Create: `football_insight_mini/src/pages/insights/components/InsightsConsensusCard.vue`
- Modify: `src/pages/insights/components/InsightsMyTacticsCard.vue`、`InsightsTacticsPanel.vue`、`src/pages/insights/index.vue`

**Interfaces:**
- Consumes: Task 3。
- Produces: `InsightsConsensusCard` 属性 `{ consensus: LineupConsensus; myPlayerIds: number[] }`，事件 `edit`（未解锁时点"去排一套"）；`InsightsMyTacticsCard` 新增属性 `overlap: string`；`InsightsTacticsPanel` 新增属性 `consensus: LineupConsensus | null`、`myOverlap: string`。

- [ ] **Step 1: 先重新读取当前的 `InsightsMyTacticsCard.vue`、`InsightsTacticsPanel.vue`、`index.vue`**（这几个文件近期改过：表态卡、`teamName`、胶囊名、滚动定位）。

- [ ] **Step 2: 共识卡片**

`src/pages/insights/components/InsightsConsensusCard.vue`：

```vue
<template>
  <!-- 大家心中的最佳 11 人：未解锁时进度 + "去排一套"（把进度变成行动）；
       解锁后在小球场展示共识，我也选了的球员红色描边、没选的变淡，点球员看被选比例。 -->
  <view class="consensus">
    <view class="consensus__head">
      <text class="consensus__title">大家心中的最佳 11 人</text>
      <text v-if="consensus.unlocked" class="consensus__meta">基于 {{ consensus.total }} 套阵容 · {{ consensus.formation }}（{{ consensus.formation_share }}% 的人用）</text>
    </view>
    <view v-if="!consensus.unlocked" class="consensus__locked">
      <text class="consensus__progress-text">{{ consensusProgressText(consensus.total, consensus.threshold) }}</text>
      <view class="consensus__bar"><view class="consensus__bar-fill" :style="{ width: `${progress}%` }" /></view>
      <button class="consensus__action" hover-class="consensus__action--pressed" @tap="emit('edit')">去排一套</button>
    </view>
    <view v-else class="consensus__pitch">
      <view class="consensus__half" />
      <view class="consensus__circle" />
      <view
        v-for="entry in placed"
        :key="entry.slot"
        class="consensus__player"
        :class="{ 'consensus__player--mine': mine.has(entry.player.player_id), 'consensus__player--other': hasMine && !mine.has(entry.player.player_id) }"
        :style="{ left: `${entry.x * 100}%`, top: `${entry.y * 100}%` }"
        hover-class="consensus__player--pressed"
        @tap="explain(entry.player)"
      >
        <image v-if="entry.player.avatar_url" class="consensus__avatar" :src="entry.player.avatar_url" mode="aspectFill" />
        <view v-else class="consensus__avatar consensus__avatar--fallback"><text>{{ entry.player.name.slice(0, 1) }}</text></view>
        <text class="consensus__name">{{ entry.player.name }}</text>
        <text class="consensus__rate">{{ entry.player.pick_rate }}%</text>
      </view>
    </view>
    <text v-if="consensus.unlocked && hasMine" class="consensus__legend">红框是你也选了的球员</text>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import type { LineupConsensus, LineupConsensusPlayer } from '../../../types/lineup'
import { consensusPlayerHint, consensusProgressText, placeConsensus } from '../../../utils/lineupConsensus'
const props = defineProps<{ consensus: LineupConsensus; myPlayerIds: number[] }>()
const emit = defineEmits<{ (e: 'edit'): void; (e: 'explain', message: string): void }>()
const placed = computed(() => placeConsensus(props.consensus))
const mine = computed(() => new Set(props.myPlayerIds))
const hasMine = computed(() => props.myPlayerIds.length > 0)
const progress = computed(() => props.consensus.threshold > 0 ? Math.min(100, Math.round(props.consensus.total * 100 / props.consensus.threshold)) : 0)
// 组件只发事件，由页面决定怎么提示（toast），保持组件不直接调用 uni。
function explain(player: LineupConsensusPlayer): void { emit('explain', consensusPlayerHint(player, mine.value.has(player.player_id))) }
</script>
<style scoped>
.consensus { display: flex; flex-direction: column; gap: var(--fi-space-12); padding: var(--fi-space-20); border-radius: var(--fi-radius-md); background: var(--fi-color-page-soft); }
.consensus__head { display: flex; flex-direction: column; gap: var(--fi-space-4); }
.consensus__title { color: var(--fi-color-text-strong); font-size: var(--fi-font-28); font-weight: var(--fi-weight-extrabold); }
.consensus__meta, .consensus__progress-text, .consensus__legend { color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
.consensus__locked { display: flex; flex-direction: column; gap: var(--fi-space-12); }
.consensus__bar { height: var(--fi-space-10); border-radius: var(--fi-radius-round); background: var(--fi-color-surface); overflow: hidden; }
.consensus__bar-fill { height: 100%; border-radius: var(--fi-radius-round); background: var(--fi-color-primary); transition: width var(--fi-motion-duration-base, 240ms) var(--fi-motion-ease-out); }
.consensus__action { align-self: flex-start; margin: 0; padding: var(--fi-space-14) var(--fi-space-32); border-radius: var(--fi-radius-round); background: var(--fi-color-primary); color: var(--fi-color-surface); font-size: var(--fi-font-24); font-weight: var(--fi-weight-bold); line-height: var(--fi-leading-normal); }
.consensus__action::after { border: 0; }
.consensus__action--pressed { opacity: var(--fi-opacity-pressed); }
.consensus__pitch { position: relative; width: 100%; height: 0; padding-top: var(--fi-component-consensus-pitch-ratio); border-radius: var(--fi-radius-md); background: var(--fi-color-lineup-grass); overflow: hidden; }
.consensus__half { position: absolute; top: 50%; left: 3%; right: 3%; border-top: var(--fi-border-width) solid var(--fi-color-lineup-markings); }
.consensus__circle { position: absolute; top: 50%; left: 50%; width: 22%; height: 16%; border: var(--fi-border-width) solid var(--fi-color-lineup-markings); border-radius: var(--fi-radius-round); transform: translate(-50%, -50%); }
.consensus__player { position: absolute; display: flex; flex-direction: column; align-items: center; gap: var(--fi-space-2); width: var(--fi-space-96); transform: translate(-50%, -50%); transition: opacity var(--fi-motion-duration-fast) var(--fi-motion-ease-out); }
.consensus__player--other { opacity: var(--fi-opacity-pressed); }
.consensus__player--pressed { transform: translate(-50%, -50%) scale(0.94); }
.consensus__avatar { width: var(--fi-space-48); height: var(--fi-space-48); border: var(--fi-border-width) solid var(--fi-color-surface); border-radius: var(--fi-radius-round); background: var(--fi-color-surface); }
.consensus__player--mine .consensus__avatar { border-color: var(--fi-color-primary); box-shadow: 0 0 0 var(--fi-border-width) var(--fi-color-primary); }
.consensus__avatar--fallback { display: flex; align-items: center; justify-content: center; color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); }
.consensus__name { max-width: 100%; overflow: hidden; color: var(--fi-color-surface); font-size: var(--fi-font-20); font-weight: var(--fi-weight-bold); text-overflow: ellipsis; white-space: nowrap; }
.consensus__rate { color: var(--fi-color-surface); font-size: var(--fi-font-18); }
@media (prefers-reduced-motion: reduce) {
  .consensus__bar-fill, .consensus__player { transition: none; }
}
</style>
```

`src/styles/fi-tokens.css` 的 component 段（`--fi-component-lineup-pitch-ratio` 附近）新增：

```css
  /* 洞察页共识卡片的球场：比编辑页（133.33%）扁，避免一张卡占满一屏；坐标按比例压缩仍可读。 */
  --fi-component-consensus-pitch-ratio: 105%;
```

`src/utils/lineupConsensus.ts` 增加（并在 Task 3 的测试里补一条：`consensusPlayerHint({ ...player, name: '某某', pick_rate: 78 }, true)` 为 `'78% 的球迷选了某某 · 你也选了'`，`false` 时为 `'78% 的球迷选了某某'`）：

```ts
export function consensusPlayerHint(player: { name: string; pick_rate: number }, mine: boolean): string {
  return `${player.pick_rate}% 的球迷选了${player.name}${mine ? ' · 你也选了' : ''}`
}
```


（若 `--fi-space-96`、`--fi-space-10`、`--fi-color-lineup-markings`、`--fi-motion-duration-base` 不存在，查 `fi-tokens.css` 用最接近的现有 token，不要写死数字；`box-shadow` 里的 0 允许。）

- [ ] **Step 3: 表态卡显示我的重合度**

`InsightsMyTacticsCard.vue`：`defineProps` 增加 `overlap: string`；在 `.my-tactics__lead` 那行之后加：

```vue
    <text v-if="overlap" class="my-tactics__overlap">{{ overlap }}</text>
```

样式：

```css
.my-tactics__overlap { display: block; margin-top: var(--fi-space-8); color: var(--fi-color-primary); font-size: var(--fi-font-24); font-weight: var(--fi-weight-bold); }
```

- [ ] **Step 4: 面板与洞察页接入**

`InsightsTacticsPanel.vue`：
- `defineProps` 增加 `consensus: LineupConsensus | null; myOverlap: string`（`import type { LineupConsensus }`）。
- `<InsightsMyTacticsCard …>` 增加 `:overlap="myOverlap"`。
- 在 `InsightsMyTacticsCard` 与 `<view class="community">` 之间加 `<InsightsConsensusCard v-if="consensus" :consensus="consensus" :my-player-ids="myPlayerIds" @edit="emit('edit')" @explain="emit('explain', $event)" />`（`import InsightsConsensusCard from './InsightsConsensusCard.vue'`；`defineProps` 再加 `myPlayerIds: number[]`，`defineEmits` 加 `(e: 'explain', message: string): void`）。

`src/pages/insights/index.vue`：

```ts
import { createLineupConsensus } from '../../composables/useLineupConsensus'
import { lineupOverlap, overlapText } from '../../utils/lineupConsensus'

const lineupConsensus = createLineupConsensus()
const consensus = lineupConsensus.consensus
// 多数据源 watch：球队变化或"排阵容"胶囊被激活（含 onShow 时 pageVisible 变化）时刷新共识
watch([selectedTeamId, () => pageVisible.value && mainTab.value === 'tactics'], ([id, active]) => {
  if (active && id !== null) void lineupConsensus.load(id)
}, { immediate: true })
const myOverlap = computed(() => minePublished.value
  ? overlapText(lineupOverlap(minePublished.value.slots.map(slot => slot.player_id), consensus.value), '你')
  : '')
```

另加 `const myPlayerIds = computed(() => (minePublished.value?.slots ?? []).map(slot => slot.player_id).filter((id): id is number => id !== null))`。模板中 `<InsightsTacticsPanel …>` 增加 `:consensus="consensus" :my-overlap="myOverlap" :my-player-ids="myPlayerIds" @explain="showHint"`，并在 script 加 `function showHint(message: string): void { uni.showToast({ title: message, icon: 'none' }) }`。（`minePublished` 已由 `useInsightsTactics` 解构得到；该 `watch` 放在 `pageVisible`、`mainTab`、`selectedTeamId` 都已声明之后。）

- [ ] **Step 5: 检查与构建**

Run:

```bash
bun test > /tmp/consensus-mini.log 2>&1; grep -E "^\(fail|^ *[0-9]+ (pass|fail)" /tmp/consensus-mini.log
bun run type-check 2>&1 | tail -1 && bun run check:boundaries | tail -1
bun run build:mp-weixin > /tmp/consensus-build.log 2>&1; echo build=$?
bun run verify:mp-components | tail -1
```

Expected: 0 fail；类型、边界通过；`build=0`；组件已注册。构建后还原被改动的版本号文件。

- [ ] **Step 6: 提交**

```bash
git add src/pages/insights src/utils/lineupConsensus.ts src/utils/lineupConsensus.test.ts src/styles/fi-tokens.css
git commit -m "feat(mini): show the fan consensus lineup and my overlap in insights"
```

---

### Task 5: 别人的阵容详情页——重合度与"独家"

**Files:**
- Modify: `football_insight_mini/src/pages/lineup/components/LineupPlayerDot.vue`、`LineupPitch.vue`、`src/pages/lineup/view.vue`

**Interfaces:**
- Consumes: Task 3。
- Produces: `LineupPlayerDot` 新增可选属性 `exclusive?: boolean`；`LineupPitch` 新增可选属性 `exclusiveIds?: number[]`（默认空数组；编辑页不传，外观不变）。

- [ ] **Step 1: 先重新读取当前的 `view.vue`、`LineupPitch.vue`、`LineupPlayerDot.vue`。**

- [ ] **Step 2: 球员圆点与球场**

`LineupPlayerDot.vue`：`defineProps` 增加 `exclusive?: boolean`；在 `dot__former` 那行之后加：

```vue
        <text v-if="exclusive" class="dot__exclusive">独家</text>
```

样式（位置与"前"对称，放右上；颜色走 token）：

```css
.dot__exclusive {
  position: absolute;
  top: calc(-1 * var(--fi-space-6));
  right: calc(-1 * var(--fi-space-14));
  padding: var(--fi-space-2) var(--fi-space-8);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-strong);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}
```

`LineupPitch.vue`：`withDefaults(defineProps<{ … exclusiveIds?: number[] }>(), { …, exclusiveIds: () => [] })`；`<LineupPlayerDot :player="entry" :selected="selectedSlot === entry.slot" :exclusive="Boolean(entry.player && exclusiveIds.includes(entry.player.player_id))" />`。

- [ ] **Step 3: 详情页接入**

`view.vue`：

```ts
import { createLineupConsensus } from '../../composables/useLineupConsensus'
import { exclusivePlayerIds, lineupOverlap, overlapText } from '../../utils/lineupConsensus'

const lineupConsensus = createLineupConsensus()
const consensus = lineupConsensus.consensus
watch(() => detail.value?.team.team_id ?? 0, teamId => { void lineupConsensus.load(teamId) }, { immediate: true })
const detailPlayerIds = computed(() => detail.value?.slots.map(slot => slot.player_id) ?? [])
const overlapLine = computed(() => overlapText(lineupOverlap(detailPlayerIds.value, consensus.value), isOwner.value ? '你' : 'TA'))
const exclusiveIds = computed(() => exclusivePlayerIds(detailPlayerIds.value, consensus.value))
```

（`watch` 只依赖一个数字，不是数组，无重复触发问题；放在 `detail`、`isOwner` 声明之后。）

模板：在"发布 … · 更新 …"那行 `community-caption` 之后加 `<text v-if="overlapLine" class="community-caption community-caption--overlap">{{ overlapLine }}</text>`；`<LineupPitch :players="pitchPlayers" readonly />` 改为 `<LineupPitch :players="pitchPlayers" :exclusive-ids="exclusiveIds" readonly />`。`community.css` 追加：

```css
.community-caption--overlap { color: var(--fi-color-primary); font-weight: var(--fi-weight-bold); }
```

- [ ] **Step 4: 检查与构建**

Run: 同 Task 4 Step 5，另加 `bun test scripts/lineup-style-audit.test.mjs 2>&1 | grep -E "^ *[0-9]+ (pass|fail)"`。
Expected: 全部通过；`build=0`。

- [ ] **Step 5: 提交**

```bash
git add src/pages/lineup/components/LineupPlayerDot.vue src/pages/lineup/components/LineupPitch.vue src/pages/lineup/view.vue src/pages/lineup/community.css
git commit -m "feat(mini): show consensus overlap and exclusive picks on lineup details"
```

---

### Task 6: 文档

- [ ] **Step 1**：`CHANGELOG.md` 顶部新增"2026-10-20 · 排阵容：球迷共识阵容（代码完成，待部署与上传）"：变更、原因（spec 路径）、验证（实际测试数字）、发布状态（无迁移；生产后端未部署；小程序未上传；线上公开阵容少于 10 套时显示进度）。`FEATURES.md` 战术板相关小节补一条。`DEPLOYMENT.md` 战术板小节补：无迁移，部署后端 → 上传小程序；核对 `GET /api/v1/lineups/teams/77680/consensus` 匿名 200。
- [ ] **Step 2**：

```bash
git add CHANGELOG.md FEATURES.md DEPLOYMENT.md
git commit -m "docs: record the fan consensus lineup"
git pull --rebase && git push
```

---

## Self-Review 记录

- **Spec 覆盖**：第 3 节算法 1–7 → Task 1（含门槛、阵型平票、按名单位置、补齐、百分比）；第 4 节重合度与独家 → Task 3 纯函数、Task 4/5 展示（只用已发布阵容、未解锁不显示）；第 5 节接口字段 → Task 2；第 6 节后端结构 → Task 1–2；第 7 节小程序（`formations` 移到 utils、卡片、表态卡重合度、详情页"TA/你"与独家）→ Task 3–5；第 8 节测试 → 各任务。
- **已裁定**：`team_lineup_players` 返回的球员顺序不重要（算法只用集合计数），SQL 不排序；共识加载失败静默（增强信息），同队保留上次结果。
- **类型一致性**：后端 `group`/`position` 为 `"GK"|"DF"|"MF"|"FW"`，与前端 `LineupPosition` 一致；`unlocked` 由 `formation.is_some()` 得出，与 `players` 是否为空一致。
