# 排阵容：球迷共识阵容设计

日期：2026-10-20
状态：设计已在对话中确认，待负责人审阅本文档

## 1. 目标与边界

**目标**：让每一套阵容都有参照。把一支球队所有公开阵容汇总成"大家心中的最佳 11 人"，并告诉球迷"你和大家重合 8 / 11"，让排阵容从"自己排着玩"变成"和其他球迷比一比"，重合度本身也是讨论和分享的理由。休赛期即可用，不依赖任何新数据源。

**做**：
- 后端按球队计算共识阵容（阵型 + 11 人 + 每人被选比例），样本不足时只返回进度。
- 洞察页"排阵容"展示共识阵容卡片；自己已发布时显示重合度。
- 别人的阵容详情页显示"TA 和大家重合 N / 11"，并标出与共识不同的球员（"独家"）。

**不做**：共识阵容的海报与单独分享页；按时间段的共识（如"本月共识"）；按比赛的首发预测；替补、多套阵容。

## 2. 全局约束

- 界面风格与现有洞察页、战术板一致；组件化（组件只收属性、发事件，调接口与跳转放页面或组合函数，`bun run check:boundaries` 保证）；token 化（界面样式引用 `fi-tokens.css`，全局与战术板样式审计自动覆盖）。
- 后端六边形：沿用 `lineup` 模块（`domain` 纯规则 / `ports` / `application` / `adapters`）；共识算法是 domain 纯函数。
- 只做微信小程序，不处理 H5。
- 不新增数据库表、不需要迁移；不在生产库做写入测试。
- 主包距 2MB 约 410KB：洞察页卡片属于主包，组件要轻；详情页改动在战术板分包。

## 3. 算法（domain 纯函数）

输入：某球队所有**公开阵容**（与现有公开列表、点赞榜同一口径：满 11 人且 11 人互不相同、作者账号正常），每套阵容为 `{ formation, player_ids[11] }`；以及该球队名单里每个球员的位置（`GK/DF/MF/FW`，取名单表中该球员在**本队**那一行的 `position`，含已离队旧将；查不到位置记为未知）。每人每队只有一套阵容，所以天然一人一票。

1. **门槛**：阵容数 `< CONSENSUS_MIN_LINEUPS`（常量，取 10）时不出共识，只返回总数与门槛。
2. **阵型**：取使用次数最多的阵型；次数相同时按 `FORMATIONS` 常量顺序取靠前者。
3. **位置需求**：由阵型名解析：门将 1；第一个数字为后卫数，最后一个数字为前锋数，中间数字之和为中场数（`4-2-3-1` → 后卫 4、中场 5、前锋 1）。七种阵型均满足合计 10 名场上球员。
4. **选人**：统计每名球员被多少套阵容选中；在每类位置中按被选次数从多到少取所需人数，次数相同按 `player_id` 升序。位置按名单位置归类，与他在某套阵容里被摆在哪个位置无关。
5. **补齐**：某类位置被选过的球员不够时，用其余尚未入选、被选次数最多的球员补齐（同样按次数降序、`player_id` 升序），补进来的球员占用该类位置；保证结果恰为 11 人。未知位置的球员只参与补齐。
6. **被选比例**：`被选次数 ÷ 阵容总数`，返回为 0–100 的整数百分比（四舍五入）。
7. **阵型占比**：使用共识阵型的阵容数 ÷ 阵容总数，同样为整数百分比。

## 4. 重合度与"独家"（小程序纯函数）

- **重合度**：两套阵容中相同 `player_id` 的个数，显示为"N / 11"。只比较球员，不比较位置。
- **独家**：某套阵容里不在共识 11 人中的球员。
- 都在小程序端用共识接口返回的球员 ID 计算，不需要额外接口。
- 自己的重合度只用**已发布**的阵容计算，草稿不算；共识未解锁时不显示重合度与"独家"。

## 5. 接口

`GET /api/v1/lineups/teams/{team_id}/consensus`，不需要登录，响应加 `no-store`（与现有公开战术接口一致）。

```json
{
  "team": { "team_id": 77680, "team_name": "成都蓉城", "avatar_url": "…" },
  "total": 36,
  "threshold": 10,
  "unlocked": true,
  "formation": "4-3-3",
  "formation_share": 58,
  "players": [
    { "player_id": 1, "name": "某某", "avatar_url": null, "shirt_number": 1, "position": "GK", "group": "GK", "pick_rate": 97 }
  ]
}
```

- 未解锁时 `formation` 为 `null`、`formation_share` 为 `0`、`players` 为空数组。
- `position` 为名单位置（可为 `null`）；`group` 为该球员在共识里占用的位置类（补齐的球员 `group` 可能与 `position` 不同），小程序按 `group` 把 11 人摆进共识阵型的对应位置（同类内按返回顺序），坐标沿用小程序 `formations.ts`。
- 球队不存在 404，`team_id` 非正数 400，存储不可用 503。

## 6. 后端结构

在 `football_insight_service_backend_rs/src/lineup/`：
- `domain.rs`：`CONSENSUS_MIN_LINEUPS`、`PositionGroup` 需求解析 `formation_needs(formation) -> Option<[usize; 4]>`、`compute_consensus(lineups, positions) -> Consensus`、相关类型。
- `ports.rs`：`PublicLineupRepository` 增加 `async fn team_lineup_players(&self, team_id: i64) -> Result<Vec<(String, Vec<i64>)>, LineupPublicError>`（每套公开阵容的阵型与 11 个球员 ID，复用现有公开可见条件 `VISIBLE`）。球员位置与资料复用 `SquadRepository::list_squad`（含现役与旧将）。
- `application/public_lineup.rs`：`PublicLineupService::consensus(team_id)`，读球队 → 读阵容 → 读名单 → 调纯函数 → 组装返回。
- `adapters/public_persistence.rs`、`public_dto.rs`、`public_web.rs`：SQL、响应结构、路由。
- 每次请求现算，不缓存（阵容规模在数百以内）。

## 7. 小程序

- `src/types/lineup.ts`、`src/api/lineup.ts`：共识类型与 `getLineupConsensus(teamId)`。
- `src/utils/lineupConsensus.ts`（纯函数，主包）：`lineupOverlap(playerIds, consensus)`、`exclusivePlayerIds(playerIds, consensus)`、`consensusProgressText(total, threshold)`、按 `group` 摆位的坐标计算（复用 `formations.ts` 的位置坐标；若 `formations.ts` 位于战术板分包，则把摆位所需的最小数据提到 `utils`，不跨包引用页面文件）。
- 洞察页"排阵容"：
  - 新组件 `InsightsConsensusCard.vue`：未解锁显示"已有 N 套阵容，再有 M 套就能看到大家心中的最佳 11 人"与进度条；已解锁显示小球场（11 人头像/名字/被选比例）与"基于 N 套阵容 · 4-3-3（58% 的人用）"。
  - 表态卡在自己已发布且共识已解锁时显示"你和大家重合 N / 11"。
  - `useInsightsTactics` 负责随球队加载共识，与公开列表同样做换队、换账号、返回页面的请求隔离。
- 别人的阵容详情页 `pages/lineup/view.vue`：加载共识；已解锁时标题下显示"TA 和大家重合 N / 11"（本人看自己的阵容显示"你和大家重合"），并给不在共识里的球员加"独家"小标记（只读球场上的标记样式走 token）。

## 8. 测试

- **domain**：阵型需求解析（七种阵型合计 10）；门槛前后；阵型平票按 `FORMATIONS` 顺序；选人平票按 `player_id`；按名单位置归类而非阵容摆位；某类不够时补齐且总数恰为 11；未知位置只参与补齐；被选比例与阵型占比的四舍五入。
- **SQL**（一次性数据库，`#[ignore]`）：只统计公开且满 11 人、作者正常的阵容；草稿与不足 11 人的不计入。
- **Web**：匿名可访问、`no-store`、404/400。
- **小程序**：重合度、独家、进度文案；组合函数的换队隔离；构建、类型、边界、组件注册。

## 9. 风险

- 当前线上公开阵容只有 2 套，上线后一段时间内都会显示"还差几套"。这是有意的：进度本身是邀请大家去排。门槛常量上线后按数据调整。
- 某些阵型（如 3-5-2）需要 5 名中场，若球迷普遍把后卫摆在中场位置，按名单位置归类后中场人数可能不够，会触发补齐；补齐球员的 `group` 与名单位置不同，界面按 `group` 摆放，这是可接受的近似。
