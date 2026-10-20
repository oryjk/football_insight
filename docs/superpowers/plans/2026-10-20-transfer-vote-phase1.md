# 转会投票第一期 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 球迷可以在本期转会窗口里，对自己主队提名"希望离队 / 希望留下 / 希望买入（含旧将回归）"的球员，并对每张提名卡赞成或踩；洞察页有"转会"胶囊，完整榜单与单卡页在新分包里，支持好友卡片分享。

**Architecture:** 后端新增六边形模块 `transfer`（domain 纯规则 → ports → application `TransferService` → adapters：SQLx 持久化 + Axum 路由），三张新表由一条迁移创建并写入第一期"2026 冬窗"。小程序主包只放洞察页面板、共用的 `FiTransferCard` 与改名后的 `FiReactionBar`、接口/类型/链接工具和赞踩组合函数；完整榜单页、单卡页与提名弹层放在新分包 `pages/transfer`。

**Tech Stack:** Rust 2024 / Axum 0.8 / SQLx 0.8 (PostgreSQL) / async-trait / thiserror 2 / rand 0.9；uni-app (Vue 3 + TS) / Bun test。

**Spec:** `docs/superpowers/specs/2026-10-20-transfer-vote-design.md`（本计划只实现第 10 节"第一期"；海报与小程序码属于第二期）。

## Global Constraints

- 界面风格与现有首页、洞察页、战术板一致；只用现有 Fi 组件拼装（`FiPanel`、`FiSectionHeading`、`FiSegmented`、`FiBottomSheet`、`FiLoading`、`FiSecondaryNav`）。
- 组件化：组件只接收属性、发出事件；调接口和跳转放在页面或组合函数里（`bun run check:boundaries` 机器保证；不要往 ALLOWLIST 加例外）。
- token 化：颜色、字号、间距、圆角全部引用 `fi-tokens.css`；`scripts/style-token-audit.test.mjs`（全局）自动覆盖新文件；新增 `scripts/transfer-style-audit.test.mjs`：无渐变、无散写 hex、无数字字重。
- 后端六边形：`src/transfer/{domain.rs,ports.rs,application.rs,adapters/,bootstrap.rs}`；业务规则只在 domain 与 application，不写进路由或 SQL。
- 主包距 2MB 上限约 450KB：完整页面只放分包 `pages/transfer`。
- 迁移文件名 `20261020120000_add_transfer_votes.sql`（大于现有最新 `20261019160000`）。迁移不随部署执行；生产迁移、部署、上传均需负责人确认，本计划不执行。
- 不在生产库做写入测试；SQL 测试只在一次性本地数据库（`TRANSFER_TEST_DATABASE_URL`，主机必须是 localhost 且库名以 `_test` 结尾）。
- 提交信息：英文 subject，`feat(transfer): ...` / `feat(mini): ...` / `docs: ...`，结尾 `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`。
- 后端只格式化本次改动的文件：`rustfmt --edition 2024 <files>`（不要 `cargo fmt` 全仓）。
- 审核期间不隐藏转会功能（负责人确认）。
- **只做微信小程序**：不处理 `football_insight_h5`，也不验证 uni-app 的 H5 构建；验证只用 `type-check`、`bun test`、`check:boundaries`、`build:mp-weixin`、`verify:mp-components`。

## 已裁定的实现细节（相对 spec 的收窄，执行时不再讨论）

- **错误响应体**：沿用 `lineup` 模块写法——HTTP 状态码 + 纯文本中文消息（小程序 `request` 已把字符串响应体变成 `ApiRequestError.message`）。spec 第 6 节表格里的 `code` 列落实为 HTTP 状态码区分，文案即 `TransferError` 的 `Display`。
- **排序在 SQL 中完成**（分页需要）：`ROW_NUMBER() OVER (ORDER BY up - down DESC, up DESC, created_at, share_code)`，只对"有票"的卡排名；spec 中的 `rank_cards` 纯函数不单独实现。排序规则由 Task 3 集成测试锁定。
- **只统计账号正常（`status = 'active'`）用户的票**，与战术板点赞口径一致；被停用账号的票不计入赞踩数，也不让卡片"有票"。
- **分享码**：8 位 base62，字符集与战术板相同；`transfer` 模块自带生成与校验函数，不依赖 `lineup` 模块。

## Review Focus

1. 球员已从所有名单中消失（同步后被移除）——卡片仍能显示，名字回退为"未知球员"、头像为空，不报错。（Task 3 测试 `card_survives_player_removed_from_all_squads`）
2. 用户提名后改了主队——旧队的票仍计数，但不能再给旧队投票，返回 403。（Task 2 测试 `changing_favorite_team_keeps_votes_but_blocks_new_ones`）
3. 浏览时本期还开放、点击时已截止——返回 409，小程序保留原状态并提示"本期投票已结束"。（Task 2 测试 `closed_window_rejects_votes_and_nominations`；Task 6 测试 `server_rejection_keeps_tally_and_returns_message`）
4. 游客从分享卡片打开单卡页后点赞踩——先去登录，登录后回到这张卡。（Task 6 测试 `guest_vote_remembers_the_card_url`）
5. 停用账号的票不计入。（Task 3 测试 `disabled_voters_are_not_counted_and_empty_cards_hide`）

---

## File Structure

后端 `football_insight_service_backend_rs/`：

| 文件 | 职责 |
|---|---|
| `migrations/20261020120000_add_transfer_votes.sql` | 三张表 + 第一期数据 |
| `src/transfer/mod.rs` | 子模块声明 |
| `src/transfer/domain.rs` | 意向/投票值/窗口/身份/卡片类型、纯规则、错误 |
| `src/transfer/ports.rs` | `TransferRepository`、`TransferCandidateDirectory`、`TransferUserAccess` |
| `src/transfer/application.rs` | `TransferService` 编排 + 假实现单元测试 |
| `src/transfer/adapters/mod.rs` | 子模块声明 |
| `src/transfer/adapters/persistence.rs` | SQLx 实现三个端口 |
| `src/transfer/adapters/dto.rs` | 请求/响应结构 |
| `src/transfer/adapters/web.rs` | Axum 路由、鉴权、错误映射 + 路由测试 |
| `src/transfer/bootstrap.rs` | 装配 |
| `src/lib.rs`、`src/app.rs` | 注册模块与路由 |
| `tests/transfer_postgres.rs` | 一次性数据库集成测试 |

小程序 `football_insight_mini/`：

| 文件 | 职责 |
|---|---|
| `src/components/FiReactionBar.vue` | 由 `FiLineupReactionBar.vue` 改名，新增 `dislikeCount` |
| `src/components/FiTransferCard.vue` | 一行提名卡（主包，洞察页与分包共用） |
| `src/types/transfer.ts` | 类型 |
| `src/api/transfer.ts` | 接口 |
| `src/utils/transferLinks.ts` | 页面链接、意向文案、分享标题（纯函数） |
| `src/composables/useTransferVotes.ts` | 赞踩共享逻辑 |
| `src/pages/insights/useInsightsTransfer.ts` | 洞察页转会数据 |
| `src/pages/insights/components/InsightsTransferPanel.vue` | 洞察页转会面板 |
| `src/pages/insights/index.vue` | 第三个胶囊、分享标题、onShow 刷新 |
| `src/pages/transfer/helpers.ts` | 候选过滤等纯函数 |
| `src/pages/transfer/useTransferBoard.ts` | 完整榜单页状态 |
| `src/pages/transfer/useTransferNominate.ts` | 提名流程 |
| `src/pages/transfer/components/TransferNominateSheet.vue` | 提名弹层 |
| `src/pages/transfer/board.vue`、`view.vue`、`transfer.css` | 分包页面与共用样式 |
| `src/pages.json` | 分包与预加载 |
| `src/pages/home/components/HomeSupportPanel.vue`、`src/pages/home/index.vue` | 首页入口 |
| `scripts/transfer-style-audit.test.mjs` | 分包样式审计 |

---

### Task 1: 迁移与 domain 纯规则

**Files:**
- Create: `football_insight_service_backend_rs/migrations/20261020120000_add_transfer_votes.sql`
- Create: `football_insight_service_backend_rs/src/transfer/mod.rs`
- Create: `football_insight_service_backend_rs/src/transfer/domain.rs`
- Create: `football_insight_service_backend_rs/src/transfer/ports.rs`（Task 2 填充，本任务先建空模块以便编译）
- Create: `football_insight_service_backend_rs/src/transfer/application.rs`（同上，空）
- Create: `football_insight_service_backend_rs/src/transfer/adapters/mod.rs`（空）
- Modify: `football_insight_service_backend_rs/src/lib.rs`（加 `pub mod transfer;`）

**Interfaces:**
- Produces（`crate::transfer::domain`）:
  - `enum TransferIntent { Out, Stay, In }`，`TransferIntent::parse(&str) -> Option<Self>`，`as_str(self) -> &'static str`，`const ALL: [TransferIntent; 3]`
  - `enum VoteValue { Up, Down }`，`VoteValue::from_i16(i16) -> Option<Self>`，`as_i16(self) -> i16`
  - `struct TransferWindow { id: i64, code: String, name: String, opens_at: DateTime<Utc>, closes_at: DateTime<Utc> }`，`is_open(&self, now) -> bool`
  - `enum Membership { Current, Former, Elsewhere, Unknown }`，`membership_from(team_id: i64, rows: &[(i64, bool)]) -> Membership`
  - `candidate_allowed(TransferIntent, Membership) -> bool`、`is_returning(TransferIntent, former_of_team: bool) -> bool`、`is_departed(TransferIntent, active_in_team: bool) -> bool`
  - `const NOMINATION_LIMIT: i64 = 3`、`nomination_limit_reached(created: i64) -> bool`
  - `valid_share_code(&str) -> bool`、`generate_share_code() -> String`
  - `struct TransferTeam { team_id: i64, team_name: String, avatar_url: Option<String> }`
  - `struct CardPlayer { player_id: i64, name: String, avatar_url: Option<String>, position: Option<String>, shirt_number: Option<i32>, current_team: Option<TransferTeam>, departed: bool }`
  - `struct NominationCard { share_code: String, intent: TransferIntent, returning: bool, player: CardPlayer, up: i64, down: i64, my_vote: i16, rank: Option<i64>, created_at: DateTime<Utc> }`
  - `struct CandidatePlayer { player_id: i64, name: String, shirt_number: Option<i32>, position: Option<String>, avatar_url: Option<String>, current_team: Option<TransferTeam> }`
  - `struct VoteTally { up: i64, down: i64, my_vote: i16 }`
  - `enum TransferError { Invalid, Unauthorized, NotFavoriteTeam, NotFound, WindowClosed, InvalidCandidate, NominationLimit, ShareCodeTaken, Unavailable }`（`thiserror`，`Display` 为中文文案）

- [ ] **Step 1: 写迁移**

`migrations/20261020120000_add_transfer_votes.sql`：

```sql
-- 转会投票：每期一个窗口；"期 + 球队 + 球员 + 意向"唯一的一张提名卡；每人每卡一票（+1 赞成 / -1 踩）。
CREATE TABLE f_i_transfer_windows (
    id BIGSERIAL PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    opens_at TIMESTAMPTZ NOT NULL,
    closes_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uq_f_i_transfer_windows_code UNIQUE (code),
    CONSTRAINT ck_f_i_transfer_windows_range CHECK (closes_at > opens_at)
);

CREATE TABLE f_i_transfer_nominations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    window_id BIGINT NOT NULL REFERENCES f_i_transfer_windows(id) ON DELETE CASCADE,
    team_id BIGINT NOT NULL REFERENCES f_i_teams(team_id),
    player_id BIGINT NOT NULL,
    intent TEXT NOT NULL,
    share_code TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_f_i_transfer_nominations_intent CHECK (intent IN ('out', 'stay', 'in')),
    CONSTRAINT uq_f_i_transfer_nominations_share_code UNIQUE (share_code),
    CONSTRAINT uq_f_i_transfer_nominations_card UNIQUE (window_id, team_id, player_id, intent)
);
CREATE INDEX idx_f_i_transfer_nominations_board ON f_i_transfer_nominations (window_id, team_id, intent);
CREATE INDEX idx_f_i_transfer_nominations_creator ON f_i_transfer_nominations (window_id, created_by, intent);

CREATE TABLE f_i_transfer_votes (
    nomination_id UUID NOT NULL REFERENCES f_i_transfer_nominations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    value SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (nomination_id, user_id),
    CONSTRAINT ck_f_i_transfer_votes_value CHECK (value IN (1, -1))
);
CREATE INDEX idx_f_i_transfer_votes_user ON f_i_transfer_votes (user_id);

INSERT INTO f_i_transfer_windows (code, name, opens_at, closes_at)
VALUES ('2026-winter', '2026 冬窗', '2026-10-20T00:00:00+08:00', '2027-03-01T00:00:00+08:00');
```

- [ ] **Step 2: 建模块骨架并注册**

`src/transfer/mod.rs`：

```rust
pub mod adapters;
pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod ports;
```

`src/transfer/adapters/mod.rs`、`src/transfer/ports.rs`、`src/transfer/application.rs`、`src/transfer/bootstrap.rs` 先写成空文件（只含一行注释 `// filled by later tasks`）。在 `src/lib.rs` 的 `pub mod ticket_watch;` 之后加 `pub mod transfer;`。

- [ ] **Step 3: 写 domain 的失败测试**

`src/transfer/domain.rs` 末尾（实现写在 Step 5，先放测试）：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 1, hour, 0, 0).unwrap()
    }

    #[test]
    fn intent_and_vote_round_trip_and_reject_unknown_values() {
        for intent in TransferIntent::ALL {
            assert_eq!(TransferIntent::parse(intent.as_str()), Some(intent));
        }
        assert_eq!(TransferIntent::parse("OUT"), None);
        assert_eq!(VoteValue::from_i16(1), Some(VoteValue::Up));
        assert_eq!(VoteValue::from_i16(-1), Some(VoteValue::Down));
        assert_eq!(VoteValue::from_i16(0), None);
        assert_eq!(VoteValue::Down.as_i16(), -1);
    }

    #[test]
    fn window_is_open_on_a_half_open_interval() {
        let window = TransferWindow {
            id: 1,
            code: "w".into(),
            name: "w".into(),
            opens_at: at(8),
            closes_at: at(10),
        };
        assert!(!window.is_open(at(7)));
        assert!(window.is_open(at(8)));
        assert!(window.is_open(at(9)));
        assert!(!window.is_open(at(10)));
    }

    #[test]
    fn membership_prefers_current_then_former_then_elsewhere() {
        assert_eq!(membership_from(1, &[(1, true)]), Membership::Current);
        assert_eq!(membership_from(1, &[(1, false), (2, true)]), Membership::Former);
        assert_eq!(membership_from(1, &[(2, true)]), Membership::Elsewhere);
        assert_eq!(membership_from(1, &[(2, false)]), Membership::Unknown);
        assert_eq!(membership_from(1, &[]), Membership::Unknown);
    }

    #[test]
    fn candidates_follow_the_three_intents() {
        use Membership::*;
        for (intent, allowed) in [
            (TransferIntent::Out, [true, false, false, false]),
            (TransferIntent::Stay, [true, false, false, false]),
            (TransferIntent::In, [false, true, true, false]),
        ] {
            let got = [Current, Former, Elsewhere, Unknown].map(|m| candidate_allowed(intent, m));
            assert_eq!(got, allowed, "{intent:?}");
        }
    }

    #[test]
    fn returning_and_departed_flags() {
        assert!(is_returning(TransferIntent::In, true));
        assert!(!is_returning(TransferIntent::In, false));
        assert!(!is_returning(TransferIntent::Out, true));
        assert!(is_departed(TransferIntent::Out, false));
        assert!(is_departed(TransferIntent::Stay, false));
        assert!(!is_departed(TransferIntent::Stay, true));
        assert!(!is_departed(TransferIntent::In, false));
    }

    #[test]
    fn nomination_limit_is_three_new_cards() {
        assert!(!nomination_limit_reached(2));
        assert!(nomination_limit_reached(3));
        assert!(nomination_limit_reached(4));
    }

    #[test]
    fn share_codes_are_eight_base62_characters() {
        for _ in 0..50 {
            let code = generate_share_code();
            assert!(valid_share_code(&code), "{code}");
        }
        assert!(valid_share_code("aAzZ0199"));
        for bad in ["", "short", "toolong123", "with-dsh", "中文中文中文中文"] {
            assert!(!valid_share_code(bad), "{bad}");
        }
    }

    #[test]
    fn errors_have_user_facing_chinese_messages() {
        assert_eq!(TransferError::NotFavoriteTeam.to_string(), "只能为你的主队投票");
        assert_eq!(TransferError::WindowClosed.to_string(), "本期投票已结束");
        assert_eq!(TransferError::NominationLimit.to_string(), "本期这一类最多提名 3 位");
    }
}
```

- [ ] **Step 4: 运行，确认失败**

Run: `cd football_insight_service_backend_rs && cargo test --lib transfer::domain 2>&1 | tail -5`
Expected: 编译失败，提示 `TransferIntent` 等未定义。

- [ ] **Step 5: 实现 domain**

`src/transfer/domain.rs` 测试模块之前：

```rust
use chrono::{DateTime, Utc};

/// 提名意向：希望离队 / 希望留下 / 希望买入（本队旧将显示为"希望回归"）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferIntent {
    Out,
    Stay,
    In,
}

impl TransferIntent {
    pub const ALL: [TransferIntent; 3] = [Self::Out, Self::Stay, Self::In];

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "out" => Some(Self::Out),
            "stay" => Some(Self::Stay),
            "in" => Some(Self::In),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Out => "out",
            Self::Stay => "stay",
            Self::In => "in",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoteValue {
    Up,
    Down,
}

impl VoteValue {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Up),
            -1 => Some(Self::Down),
            _ => None,
        }
    }

    pub fn as_i16(self) -> i16 {
        match self {
            Self::Up => 1,
            Self::Down => -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferWindow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub opens_at: DateTime<Utc>,
    pub closes_at: DateTime<Utc>,
}

impl TransferWindow {
    /// 开放区间左闭右开：`opens_at <= now < closes_at`。
    pub fn is_open(&self, now: DateTime<Utc>) -> bool {
        self.opens_at <= now && now < self.closes_at
    }
}

/// 球员相对某支球队的身份。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    /// 本队现役
    Current,
    /// 本队旧将（名单里有本队 active = false 的记录），不论现在是否在别队现役
    Former,
    /// 从未在本队、现于其他队现役
    Elsewhere,
    /// 以上都不是（不在任何现役名单里，也没在本队效力过）
    Unknown,
}

/// `rows` 为该球员在名单表里的 (team_id, active) 全部记录。
pub fn membership_from(team_id: i64, rows: &[(i64, bool)]) -> Membership {
    if rows.iter().any(|&(team, active)| team == team_id && active) {
        Membership::Current
    } else if rows.iter().any(|&(team, active)| team == team_id && !active) {
        Membership::Former
    } else if rows.iter().any(|&(_, active)| active) {
        Membership::Elsewhere
    } else {
        Membership::Unknown
    }
}

pub fn candidate_allowed(intent: TransferIntent, membership: Membership) -> bool {
    match intent {
        TransferIntent::Out | TransferIntent::Stay => membership == Membership::Current,
        TransferIntent::In => matches!(membership, Membership::Former | Membership::Elsewhere),
    }
}

pub fn is_returning(intent: TransferIntent, former_of_team: bool) -> bool {
    intent == TransferIntent::In && former_of_team
}

/// "走 / 留"卡上的球员已不在本队现役。
pub fn is_departed(intent: TransferIntent, active_in_team: bool) -> bool {
    matches!(intent, TransferIntent::Out | TransferIntent::Stay) && !active_in_team
}

/// 每人每期每种意向最多新建的提名卡数；为已存在的卡加票不占名额。
pub const NOMINATION_LIMIT: i64 = 3;

pub fn nomination_limit_reached(created: i64) -> bool {
    created >= NOMINATION_LIMIT
}

pub fn valid_share_code(code: &str) -> bool {
    code.len() == 8 && code.bytes().all(|b| b.is_ascii_alphanumeric())
}

pub fn generate_share_code() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::rng();
    (0..8)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferTeam {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardPlayer {
    pub player_id: i64,
    pub name: String,
    pub avatar_url: Option<String>,
    pub position: Option<String>,
    pub shirt_number: Option<i32>,
    pub current_team: Option<TransferTeam>,
    pub departed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominationCard {
    pub share_code: String,
    pub intent: TransferIntent,
    pub returning: bool,
    pub player: CardPlayer,
    pub up: i64,
    pub down: i64,
    /// -1 / 0 / 1；游客恒为 0
    pub my_vote: i16,
    /// 一票都没有的卡为 None（不进榜单）
    pub rank: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidatePlayer {
    pub player_id: i64,
    pub name: String,
    pub shirt_number: Option<i32>,
    pub position: Option<String>,
    pub avatar_url: Option<String>,
    pub current_team: Option<TransferTeam>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoteTally {
    pub up: i64,
    pub down: i64,
    pub my_vote: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransferError {
    #[error("参数无效")]
    Invalid,
    #[error("请登录后继续")]
    Unauthorized,
    #[error("只能为你的主队投票")]
    NotFavoriteTeam,
    #[error("这张提名不存在")]
    NotFound,
    #[error("本期投票已结束")]
    WindowClosed,
    #[error("这位球员不能这样提名")]
    InvalidCandidate,
    #[error("本期这一类最多提名 3 位")]
    NominationLimit,
    /// 内部：分享码撞车，由 application 重试，不应返回给用户
    #[error("分享码冲突")]
    ShareCodeTaken,
    #[error("服务暂时不可用，请稍后重试")]
    Unavailable,
}
```

- [ ] **Step 6: 运行，确认通过**

Run: `cargo test --lib transfer::domain 2>&1 | tail -5`
Expected: `test result: ok. 8 passed`

- [ ] **Step 7: 格式化并提交**

```bash
rustfmt --edition 2024 src/transfer/*.rs src/transfer/adapters/mod.rs src/lib.rs
git add migrations/20261020120000_add_transfer_votes.sql src/transfer src/lib.rs
git commit -m "feat(transfer): add transfer vote tables and domain rules

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: ports 与 `TransferService`

**Files:**
- Modify: `football_insight_service_backend_rs/src/transfer/ports.rs`
- Modify: `football_insight_service_backend_rs/src/transfer/application.rs`

**Interfaces:**
- Consumes: Task 1 全部 domain 类型。
- Produces（`crate::transfer::ports`）:

```rust
pub struct UserProfile { pub active: bool, pub favorite_team_id: Option<i64> }
pub struct CardContext { pub card: NominationCard, pub team: TransferTeam, pub window: TransferWindow }
pub struct NominateCommand { pub window_id: i64, pub team_id: i64, pub player_id: i64, pub intent: TransferIntent, pub user_id: Uuid, pub share_code: String, pub limit: i64 }
pub struct NominateOutcome { pub share_code: String, pub created: bool }

#[async_trait] pub trait TransferRepository: Send + Sync {
    async fn current_window(&self, now: DateTime<Utc>) -> Result<Option<TransferWindow>, TransferError>;
    async fn window_by_code(&self, code: &str) -> Result<Option<TransferWindow>, TransferError>;
    async fn board(&self, window_id: i64, team_id: i64, intent: TransferIntent, offset: i64, limit: i64, viewer: Option<Uuid>) -> Result<(Vec<NominationCard>, i64), TransferError>;
    async fn card(&self, share_code: &str, viewer: Option<Uuid>) -> Result<Option<CardContext>, TransferError>;
    async fn nominate(&self, command: NominateCommand) -> Result<NominateOutcome, TransferError>;
    async fn set_vote(&self, share_code: &str, user_id: Uuid, value: Option<VoteValue>) -> Result<VoteTally, TransferError>;
}
#[async_trait] pub trait TransferCandidateDirectory: Send + Sync {
    async fn find_team(&self, team_id: i64) -> Result<Option<TransferTeam>, TransferError>;
    async fn player_rows(&self, player_id: i64) -> Result<Vec<(i64, bool)>, TransferError>;
    async fn team_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError>;
    async fn former_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError>;
    async fn other_active_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError>;
}
#[async_trait] pub trait TransferUserAccess: Send + Sync {
    async fn profile(&self, user_id: Uuid) -> Result<Option<UserProfile>, TransferError>;
}
```

- Produces（`crate::transfer::application`）:

```rust
pub type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;
pub const BOARD_PAGE_SIZE: i64 = 20;
pub const SUMMARY_SIZE: i64 = 3;
pub struct WindowView { pub window: TransferWindow, pub open: bool }
pub struct BoardSlice { pub intent: TransferIntent, pub total: i64, pub items: Vec<NominationCard> }
pub struct TransferSummary { pub team: TransferTeam, pub window: Option<WindowView>, pub can_vote: bool, pub boards: Vec<BoardSlice> }
pub struct TransferBoard { pub team: TransferTeam, pub window: Option<WindowView>, pub intent: TransferIntent, pub page: i64, pub page_size: i64, pub total: i64, pub can_vote: bool, pub items: Vec<NominationCard> }
pub struct CardView { pub card: NominationCard, pub team: TransferTeam, pub window: WindowView, pub can_vote: bool }
pub struct CandidateGroup { pub key: String, pub title: String, pub players: Vec<CandidatePlayer> }
pub struct Candidates { pub intent: TransferIntent, pub groups: Vec<CandidateGroup> }
pub struct NominateResult { pub created: bool, pub view: CardView }

impl TransferService {
    pub fn new(repo: Arc<dyn TransferRepository>, directory: Arc<dyn TransferCandidateDirectory>, users: Arc<dyn TransferUserAccess>, clock: Clock) -> Self;
    pub async fn current_window(&self) -> Result<Option<WindowView>, TransferError>;
    pub async fn summary(&self, team_id: i64, window_code: Option<&str>, viewer: Option<Uuid>) -> Result<TransferSummary, TransferError>;
    pub async fn board(&self, team_id: i64, intent: &str, page: i64, window_code: Option<&str>, viewer: Option<Uuid>) -> Result<TransferBoard, TransferError>;
    pub async fn card(&self, share_code: &str, viewer: Option<Uuid>) -> Result<CardView, TransferError>;
    pub async fn candidates(&self, team_id: i64, intent: &str) -> Result<Candidates, TransferError>;
    pub async fn nominate(&self, user_id: Uuid, team_id: i64, player_id: i64, intent: &str) -> Result<NominateResult, TransferError>;
    pub async fn vote(&self, user_id: Uuid, share_code: &str, value: Option<VoteValue>) -> Result<VoteTally, TransferError>;
}
```

- [ ] **Step 1: 写 ports**

`src/transfer/ports.rs`（完整内容）：

```rust
use super::domain::{
    CandidatePlayer, NominationCard, TransferError, TransferIntent, TransferTeam, TransferWindow,
    VoteTally, VoteValue,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfile {
    pub active: bool,
    pub favorite_team_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardContext {
    pub card: NominationCard,
    pub team: TransferTeam,
    pub window: TransferWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominateCommand {
    pub window_id: i64,
    pub team_id: i64,
    pub player_id: i64,
    pub intent: TransferIntent,
    pub user_id: Uuid,
    pub share_code: String,
    /// 新建卡的上限；卡已存在时不检查
    pub limit: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominateOutcome {
    pub share_code: String,
    pub created: bool,
}

#[async_trait]
pub trait TransferRepository: Send + Sync {
    /// 开放中的一期；没有则取最近已结束的一期；都没有为 None。
    async fn current_window(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Option<TransferWindow>, TransferError>;
    async fn window_by_code(&self, code: &str) -> Result<Option<TransferWindow>, TransferError>;
    /// 只返回有票的卡（按名次），以及有票卡的总数。
    async fn board(
        &self,
        window_id: i64,
        team_id: i64,
        intent: TransferIntent,
        offset: i64,
        limit: i64,
        viewer: Option<Uuid>,
    ) -> Result<(Vec<NominationCard>, i64), TransferError>;
    /// 无论有没有票都能按分享码读到。
    async fn card(
        &self,
        share_code: &str,
        viewer: Option<Uuid>,
    ) -> Result<Option<CardContext>, TransferError>;
    /// 事务内：锁提名人 → 卡已存在则直接加票，否则检查上限并建卡 → 记 +1。
    async fn nominate(&self, command: NominateCommand) -> Result<NominateOutcome, TransferError>;
    /// `None` 表示取消投票。
    async fn set_vote(
        &self,
        share_code: &str,
        user_id: Uuid,
        value: Option<VoteValue>,
    ) -> Result<VoteTally, TransferError>;
}

#[async_trait]
pub trait TransferCandidateDirectory: Send + Sync {
    async fn find_team(&self, team_id: i64) -> Result<Option<TransferTeam>, TransferError>;
    /// 该球员在名单表里的全部 (team_id, active)。
    async fn player_rows(&self, player_id: i64) -> Result<Vec<(i64, bool)>, TransferError>;
    /// 本队现役
    async fn team_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError>;
    /// 本队旧将（current_team 为其现役所在队，可能为空）
    async fn former_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError>;
    /// 其他队现役，排除本队旧将（他们在旧将组里）
    async fn other_active_players(
        &self,
        team_id: i64,
    ) -> Result<Vec<CandidatePlayer>, TransferError>;
}

#[async_trait]
pub trait TransferUserAccess: Send + Sync {
    async fn profile(&self, user_id: Uuid) -> Result<Option<UserProfile>, TransferError>;
}
```

- [ ] **Step 2: 写 application 的失败测试（含假实现）**

`src/transfer/application.rs` 末尾：

```rust
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::transfer::domain::{CardPlayer, NominationCard};
    use chrono::TimeZone;
    use std::sync::Mutex;

    pub(crate) const FAN: Uuid = Uuid::from_u128(1);
    pub(crate) const RIVAL: Uuid = Uuid::from_u128(2);

    pub(crate) fn at(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, day, 12, 0, 0).unwrap()
    }
    pub(crate) fn window() -> TransferWindow {
        TransferWindow {
            id: 7,
            code: "2026-winter".into(),
            name: "2026 冬窗".into(),
            opens_at: at(1),
            closes_at: at(20),
        }
    }
    pub(crate) fn team(id: i64) -> TransferTeam {
        TransferTeam { team_id: id, team_name: format!("Team {id}"), avatar_url: None }
    }
    pub(crate) fn card(code: &str, intent: TransferIntent) -> NominationCard {
        NominationCard {
            share_code: code.into(),
            intent,
            returning: false,
            player: CardPlayer {
                player_id: 10,
                name: "Player 10".into(),
                avatar_url: None,
                position: Some("FW".into()),
                shirt_number: Some(10),
                current_team: Some(team(1)),
                departed: false,
            },
            up: 1,
            down: 0,
            my_vote: 1,
            rank: Some(1),
            created_at: at(2),
        }
    }

    #[derive(Default)]
    pub(crate) struct FakeRepo {
        pub nominations: Mutex<Vec<NominateCommand>>,
        pub votes: Mutex<Vec<(String, Uuid, Option<VoteValue>)>>,
        pub existing: Mutex<bool>,
        pub taken_once: Mutex<bool>,
        pub limit_hit: Mutex<bool>,
        pub card_team: Mutex<i64>,
    }
    #[async_trait::async_trait]
    impl TransferRepository for FakeRepo {
        async fn current_window(&self, _: DateTime<Utc>) -> Result<Option<TransferWindow>, TransferError> {
            Ok(Some(window()))
        }
        async fn window_by_code(&self, code: &str) -> Result<Option<TransferWindow>, TransferError> {
            Ok((code == "2026-winter").then(window))
        }
        async fn board(&self, _: i64, _: i64, intent: TransferIntent, offset: i64, limit: i64, _: Option<Uuid>) -> Result<(Vec<NominationCard>, i64), TransferError> {
            let all: Vec<_> = (0..5).map(|i| card(&format!("CODE000{i}"), intent)).collect();
            Ok((all.into_iter().skip(offset as usize).take(limit as usize).collect(), 5))
        }
        async fn card(&self, code: &str, _: Option<Uuid>) -> Result<Option<CardContext>, TransferError> {
            if code == "MISSING0" { return Ok(None); }
            let team_id = *self.card_team.lock().unwrap();
            Ok(Some(CardContext { card: card(code, TransferIntent::Out), team: team(if team_id == 0 { 1 } else { team_id }), window: window() }))
        }
        async fn nominate(&self, command: NominateCommand) -> Result<NominateOutcome, TransferError> {
            let mut taken = self.taken_once.lock().unwrap();
            if *taken { *taken = false; return Err(TransferError::ShareCodeTaken); }
            if *self.limit_hit.lock().unwrap() { return Err(TransferError::NominationLimit); }
            let created = !*self.existing.lock().unwrap();
            let code = if created { command.share_code.clone() } else { "EXIST001".into() };
            self.nominations.lock().unwrap().push(command);
            Ok(NominateOutcome { share_code: code, created })
        }
        async fn set_vote(&self, code: &str, user: Uuid, value: Option<VoteValue>) -> Result<VoteTally, TransferError> {
            self.votes.lock().unwrap().push((code.into(), user, value));
            Ok(VoteTally { up: 1, down: 0, my_vote: value.map_or(0, VoteValue::as_i16) })
        }
    }

    pub(crate) struct FakeDirectory;
    #[async_trait::async_trait]
    impl TransferCandidateDirectory for FakeDirectory {
        async fn find_team(&self, id: i64) -> Result<Option<TransferTeam>, TransferError> {
            Ok((id == 1 || id == 2).then(|| team(id)))
        }
        async fn player_rows(&self, player: i64) -> Result<Vec<(i64, bool)>, TransferError> {
            // 10 = 本队现役；20 = 本队旧将、现于 2 队；30 = 2 队现役；40 = 无效
            Ok(match player { 10 => vec![(1, true)], 20 => vec![(1, false), (2, true)], 30 => vec![(2, true)], _ => vec![] })
        }
        async fn team_players(&self, _: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
            Ok(vec![candidate(10, Some(team(1)))])
        }
        async fn former_players(&self, _: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
            Ok(vec![candidate(20, Some(team(2)))])
        }
        async fn other_active_players(&self, _: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
            Ok(vec![candidate(30, Some(team(2))), candidate(31, Some(team(3)))])
        }
    }
    fn candidate(id: i64, current_team: Option<TransferTeam>) -> CandidatePlayer {
        CandidatePlayer { player_id: id, name: format!("Player {id}"), shirt_number: None, position: None, avatar_url: None, current_team }
    }

    pub(crate) struct FakeUsers { pub favorite: Mutex<Option<i64>>, pub active: bool }
    #[async_trait::async_trait]
    impl TransferUserAccess for FakeUsers {
        async fn profile(&self, user: Uuid) -> Result<Option<UserProfile>, TransferError> {
            Ok((user == FAN || user == RIVAL).then(|| UserProfile {
                active: self.active,
                favorite_team_id: if user == FAN { *self.favorite.lock().unwrap() } else { Some(2) },
            }))
        }
    }

    pub(crate) fn service_at(now: DateTime<Utc>, active: bool) -> (TransferService, Arc<FakeRepo>, Arc<FakeUsers>) {
        let repo = Arc::new(FakeRepo::default());
        let users = Arc::new(FakeUsers { favorite: Mutex::new(Some(1)), active });
        let service = TransferService::new(repo.clone(), Arc::new(FakeDirectory), users.clone(), Arc::new(move || now));
        (service, repo, users)
    }

    #[tokio::test]
    async fn nominate_validates_candidates_per_intent() {
        let (service, repo, _) = service_at(at(5), true);
        for (player, intent, ok) in [
            (10, "out", true), (10, "stay", true), (10, "in", false),
            (20, "in", true), (30, "in", true), (30, "out", false), (40, "in", false),
        ] {
            let result = service.nominate(FAN, 1, player, intent).await;
            assert_eq!(result.is_ok(), ok, "{player} {intent}: {result:?}");
            if !ok { assert_eq!(result.unwrap_err(), TransferError::InvalidCandidate); }
        }
        assert_eq!(repo.nominations.lock().unwrap().len(), 4);
        assert!(repo.nominations.lock().unwrap().iter().all(|c| c.limit == NOMINATION_LIMIT && c.window_id == 7));
    }

    #[tokio::test]
    async fn nominate_reports_existing_cards_and_retries_share_code_collisions() {
        let (service, repo, _) = service_at(at(5), true);
        *repo.taken_once.lock().unwrap() = true;
        let created = service.nominate(FAN, 1, 10, "out").await.unwrap();
        assert!(created.created);
        assert_eq!(repo.nominations.lock().unwrap().len(), 1);
        *repo.existing.lock().unwrap() = true;
        let existing = service.nominate(FAN, 1, 10, "out").await.unwrap();
        assert!(!existing.created);
        assert_eq!(existing.view.card.share_code, "EXIST001");
        *repo.limit_hit.lock().unwrap() = true;
        assert_eq!(service.nominate(FAN, 1, 10, "stay").await.unwrap_err(), TransferError::NominationLimit);
    }

    #[tokio::test]
    async fn only_active_fans_of_the_team_can_write() {
        let (service, _, _) = service_at(at(5), true);
        assert_eq!(service.nominate(RIVAL, 1, 10, "out").await.unwrap_err(), TransferError::NotFavoriteTeam);
        assert_eq!(service.vote(RIVAL, "CODE0001", Some(VoteValue::Up)).await.unwrap_err(), TransferError::NotFavoriteTeam);
        assert_eq!(service.nominate(Uuid::from_u128(9), 1, 10, "out").await.unwrap_err(), TransferError::Unauthorized);
        let (disabled, _, _) = service_at(at(5), false);
        assert_eq!(disabled.vote(FAN, "CODE0001", Some(VoteValue::Up)).await.unwrap_err(), TransferError::Unauthorized);
        assert_eq!(disabled.summary(1, None, Some(FAN)).await.unwrap_err(), TransferError::Unauthorized);
    }

    #[tokio::test]
    async fn closed_window_rejects_votes_and_nominations() {
        let (service, _, _) = service_at(at(25), true);
        assert_eq!(service.nominate(FAN, 1, 10, "out").await.unwrap_err(), TransferError::WindowClosed);
        assert_eq!(service.vote(FAN, "CODE0001", Some(VoteValue::Down)).await.unwrap_err(), TransferError::WindowClosed);
        let summary = service.summary(1, None, Some(FAN)).await.unwrap();
        assert!(!summary.can_vote);
        assert!(!summary.window.unwrap().open);
    }

    #[tokio::test]
    async fn changing_favorite_team_keeps_votes_but_blocks_new_ones() {
        let (service, repo, users) = service_at(at(5), true);
        service.vote(FAN, "CODE0001", Some(VoteValue::Up)).await.unwrap();
        *users.favorite.lock().unwrap() = Some(2);
        assert_eq!(service.vote(FAN, "CODE0001", None).await.unwrap_err(), TransferError::NotFavoriteTeam);
        assert_eq!(repo.votes.lock().unwrap().len(), 1, "the earlier vote stays recorded");
        *repo.card_team.lock().unwrap() = 2;
        assert!(service.vote(FAN, "CODE0001", None).await.is_ok());
    }

    #[tokio::test]
    async fn reads_are_public_and_validate_inputs() {
        let (service, _, _) = service_at(at(5), true);
        let summary = service.summary(1, None, None).await.unwrap();
        assert!(!summary.can_vote);
        assert_eq!(summary.boards.iter().map(|b| (b.intent, b.items.len(), b.total)).collect::<Vec<_>>(),
            vec![(TransferIntent::Out, 3, 5), (TransferIntent::Stay, 3, 5), (TransferIntent::In, 3, 5)]);
        assert!(service.summary(1, None, Some(FAN)).await.unwrap().can_vote);
        assert_eq!(service.summary(99, None, None).await.unwrap_err(), TransferError::NotFound);
        assert_eq!(service.summary(1, Some("nope"), None).await.unwrap_err(), TransferError::NotFound);
        let board = service.board(1, "in", 1, Some("2026-winter"), None).await.unwrap();
        assert_eq!((board.page, board.page_size, board.total, board.items.len()), (1, BOARD_PAGE_SIZE, 5, 5));
        for (intent, page) in [("bad", 1), ("out", 0), ("out", -1)] {
            assert_eq!(service.board(1, intent, page, None, None).await.unwrap_err(), TransferError::Invalid);
        }
        assert_eq!(service.card("bad", None).await.unwrap_err(), TransferError::Invalid);
        assert_eq!(service.card("MISSING0", None).await.unwrap_err(), TransferError::NotFound);
        assert!(service.card("CODE0001", Some(FAN)).await.unwrap().can_vote);
    }

    #[tokio::test]
    async fn candidates_group_former_players_before_other_teams() {
        let (service, _, _) = service_at(at(5), true);
        let squad = service.candidates(1, "out").await.unwrap();
        assert_eq!(squad.groups.iter().map(|g| g.key.as_str()).collect::<Vec<_>>(), vec!["squad"]);
        let buy = service.candidates(1, "in").await.unwrap();
        assert_eq!(buy.groups.iter().map(|g| (g.key.as_str(), g.title.as_str(), g.players.len())).collect::<Vec<_>>(),
            vec![("former", "本队旧将", 1), ("team:2", "Team 2", 1), ("team:3", "Team 3", 1)]);
        assert_eq!(service.candidates(1, "x").await.unwrap_err(), TransferError::Invalid);
    }
}
```

- [ ] **Step 3: 运行，确认失败**

Run: `cargo test --lib transfer::application 2>&1 | tail -5`
Expected: 编译失败，`TransferService` 未定义。

- [ ] **Step 4: 实现 `TransferService`**

`src/transfer/application.rs` 测试模块之前：

```rust
use super::{
    domain::{
        CandidatePlayer, NOMINATION_LIMIT, NominationCard, TransferError, TransferIntent,
        TransferTeam, TransferWindow, VoteTally, VoteValue, candidate_allowed,
        generate_share_code, membership_from, valid_share_code,
    },
    ports::{
        NominateCommand, TransferCandidateDirectory, TransferRepository, TransferUserAccess,
        UserProfile,
    },
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;
pub const BOARD_PAGE_SIZE: i64 = 20;
pub const SUMMARY_SIZE: i64 = 3;
const SHARE_CODE_ATTEMPTS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowView {
    pub window: TransferWindow,
    pub open: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardSlice {
    pub intent: TransferIntent,
    pub total: i64,
    pub items: Vec<NominationCard>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferSummary {
    pub team: TransferTeam,
    pub window: Option<WindowView>,
    pub can_vote: bool,
    pub boards: Vec<BoardSlice>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferBoard {
    pub team: TransferTeam,
    pub window: Option<WindowView>,
    pub intent: TransferIntent,
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub can_vote: bool,
    pub items: Vec<NominationCard>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardView {
    pub card: NominationCard,
    pub team: TransferTeam,
    pub window: WindowView,
    pub can_vote: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGroup {
    pub key: String,
    pub title: String,
    pub players: Vec<CandidatePlayer>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidates {
    pub intent: TransferIntent,
    pub groups: Vec<CandidateGroup>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominateResult {
    pub created: bool,
    pub view: CardView,
}

pub struct TransferService {
    repo: Arc<dyn TransferRepository>,
    directory: Arc<dyn TransferCandidateDirectory>,
    users: Arc<dyn TransferUserAccess>,
    clock: Clock,
}

impl TransferService {
    pub fn new(
        repo: Arc<dyn TransferRepository>,
        directory: Arc<dyn TransferCandidateDirectory>,
        users: Arc<dyn TransferUserAccess>,
        clock: Clock,
    ) -> Self {
        Self { repo, directory, users, clock }
    }

    fn view(&self, window: TransferWindow) -> WindowView {
        let open = window.is_open((self.clock)());
        WindowView { window, open }
    }

    /// 浏览者：带 token 时账号必须正常（与战术板一致）；游客为 None。
    async fn viewer_profile(&self, viewer: Option<Uuid>) -> Result<Option<UserProfile>, TransferError> {
        let Some(user) = viewer else { return Ok(None) };
        match self.users.profile(user).await? {
            Some(profile) if profile.active => Ok(Some(profile)),
            _ => Err(TransferError::Unauthorized),
        }
    }

    /// 写操作：必须是正常账号、主队等于该队、该期开放。
    async fn require_voter(&self, user: Uuid, team_id: i64, window: &TransferWindow) -> Result<(), TransferError> {
        let profile = self.viewer_profile(Some(user)).await?.ok_or(TransferError::Unauthorized)?;
        if profile.favorite_team_id != Some(team_id) {
            return Err(TransferError::NotFavoriteTeam);
        }
        if !window.is_open((self.clock)()) {
            return Err(TransferError::WindowClosed);
        }
        Ok(())
    }

    fn can_vote(profile: &Option<UserProfile>, team_id: i64, window: Option<&WindowView>) -> bool {
        matches!(profile, Some(p) if p.favorite_team_id == Some(team_id))
            && window.is_some_and(|w| w.open)
    }

    async fn team(&self, team_id: i64) -> Result<TransferTeam, TransferError> {
        if team_id <= 0 {
            return Err(TransferError::Invalid);
        }
        self.directory.find_team(team_id).await?.ok_or(TransferError::NotFound)
    }

    async fn window(&self, code: Option<&str>) -> Result<Option<TransferWindow>, TransferError> {
        match code {
            Some(code) => self.repo.window_by_code(code).await?.map(Some).ok_or(TransferError::NotFound),
            None => self.repo.current_window((self.clock)()).await,
        }
    }

    pub async fn current_window(&self) -> Result<Option<WindowView>, TransferError> {
        Ok(self.repo.current_window((self.clock)()).await?.map(|w| self.view(w)))
    }

    pub async fn summary(&self, team_id: i64, window_code: Option<&str>, viewer: Option<Uuid>) -> Result<TransferSummary, TransferError> {
        let profile = self.viewer_profile(viewer).await?;
        let team = self.team(team_id).await?;
        let window = self.window(window_code).await?.map(|w| self.view(w));
        let mut boards = Vec::with_capacity(3);
        for intent in TransferIntent::ALL {
            let (items, total) = match &window {
                Some(w) => self.repo.board(w.window.id, team_id, intent, 0, SUMMARY_SIZE, viewer).await?,
                None => (Vec::new(), 0),
            };
            boards.push(BoardSlice { intent, total, items });
        }
        let can_vote = Self::can_vote(&profile, team_id, window.as_ref());
        Ok(TransferSummary { team, window, can_vote, boards })
    }

    pub async fn board(&self, team_id: i64, intent: &str, page: i64, window_code: Option<&str>, viewer: Option<Uuid>) -> Result<TransferBoard, TransferError> {
        let intent = TransferIntent::parse(intent).ok_or(TransferError::Invalid)?;
        if page < 1 {
            return Err(TransferError::Invalid);
        }
        let profile = self.viewer_profile(viewer).await?;
        let team = self.team(team_id).await?;
        let window = self.window(window_code).await?.map(|w| self.view(w));
        let (items, total) = match &window {
            Some(w) => self.repo.board(w.window.id, team_id, intent, (page - 1) * BOARD_PAGE_SIZE, BOARD_PAGE_SIZE, viewer).await?,
            None => (Vec::new(), 0),
        };
        let can_vote = Self::can_vote(&profile, team_id, window.as_ref());
        Ok(TransferBoard { team, window, intent, page, page_size: BOARD_PAGE_SIZE, total, can_vote, items })
    }

    pub async fn card(&self, share_code: &str, viewer: Option<Uuid>) -> Result<CardView, TransferError> {
        if !valid_share_code(share_code) {
            return Err(TransferError::Invalid);
        }
        let profile = self.viewer_profile(viewer).await?;
        let context = self.repo.card(share_code, viewer).await?.ok_or(TransferError::NotFound)?;
        let window = self.view(context.window);
        let can_vote = Self::can_vote(&profile, context.team.team_id, Some(&window));
        Ok(CardView { card: context.card, team: context.team, window, can_vote })
    }

    pub async fn candidates(&self, team_id: i64, intent: &str) -> Result<Candidates, TransferError> {
        let intent = TransferIntent::parse(intent).ok_or(TransferError::Invalid)?;
        let team = self.team(team_id).await?;
        let groups = match intent {
            TransferIntent::Out | TransferIntent::Stay => vec![CandidateGroup {
                key: "squad".into(),
                title: team.team_name.clone(),
                players: self.directory.team_players(team_id).await?,
            }],
            TransferIntent::In => {
                let mut groups = vec![CandidateGroup {
                    key: "former".into(),
                    title: "本队旧将".into(),
                    players: self.directory.former_players(team_id).await?,
                }];
                // 目录已按队名排序，这里按出现顺序分组。
                for player in self.directory.other_active_players(team_id).await? {
                    let Some(club) = player.current_team.clone() else { continue };
                    let key = format!("team:{}", club.team_id);
                    match groups.iter_mut().find(|g| g.key == key) {
                        Some(group) => group.players.push(player),
                        None => groups.push(CandidateGroup { key, title: club.team_name, players: vec![player] }),
                    }
                }
                groups
            }
        };
        Ok(Candidates { intent, groups })
    }

    pub async fn nominate(&self, user_id: Uuid, team_id: i64, player_id: i64, intent: &str) -> Result<NominateResult, TransferError> {
        let intent = TransferIntent::parse(intent).ok_or(TransferError::Invalid)?;
        if player_id <= 0 {
            return Err(TransferError::Invalid);
        }
        self.team(team_id).await?;
        let window = self.repo.current_window((self.clock)()).await?.ok_or(TransferError::WindowClosed)?;
        self.require_voter(user_id, team_id, &window).await?;
        let rows = self.directory.player_rows(player_id).await?;
        if !candidate_allowed(intent, membership_from(team_id, &rows)) {
            return Err(TransferError::InvalidCandidate);
        }
        for _ in 0..SHARE_CODE_ATTEMPTS {
            let command = NominateCommand {
                window_id: window.id,
                team_id,
                player_id,
                intent,
                user_id,
                share_code: generate_share_code(),
                limit: NOMINATION_LIMIT,
            };
            match self.repo.nominate(command).await {
                Ok(outcome) => {
                    let view = self.card(&outcome.share_code, Some(user_id)).await?;
                    return Ok(NominateResult { created: outcome.created, view });
                }
                Err(TransferError::ShareCodeTaken) => continue,
                Err(error) => return Err(error),
            }
        }
        Err(TransferError::Unavailable)
    }

    pub async fn vote(&self, user_id: Uuid, share_code: &str, value: Option<VoteValue>) -> Result<VoteTally, TransferError> {
        if !valid_share_code(share_code) {
            return Err(TransferError::Invalid);
        }
        let context = self.repo.card(share_code, Some(user_id)).await?.ok_or(TransferError::NotFound)?;
        self.require_voter(user_id, context.team.team_id, &context.window).await?;
        self.repo.set_vote(share_code, user_id, value).await
    }
}
```

注意 `require_voter` 先检查账号再检查主队，因此停用账号得到 401、非主队得到 403；`vote` 先读卡（404 优先于 403）。

- [ ] **Step 5: 运行，确认通过**

Run: `cargo test --lib transfer:: 2>&1 | tail -5`
Expected: `test result: ok. 15 passed`（domain 8 + application 7）

- [ ] **Step 6: 格式化并提交**

```bash
rustfmt --edition 2024 src/transfer/ports.rs src/transfer/application.rs
git add src/transfer/ports.rs src/transfer/application.rs
git commit -m "feat(transfer): add transfer service and ports

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: PostgreSQL 适配器与集成测试

**Files:**
- Create: `football_insight_service_backend_rs/src/transfer/adapters/persistence.rs`
- Modify: `football_insight_service_backend_rs/src/transfer/adapters/mod.rs`（`pub mod persistence;`）
- Create: `football_insight_service_backend_rs/tests/transfer_postgres.rs`

**Interfaces:**
- Consumes: Task 2 的三个端口。
- Produces: `PostgresTransferRepository::new(PgPool)`，同时实现 `TransferRepository`、`TransferCandidateDirectory`、`TransferUserAccess`。

- [ ] **Step 1: 写集成测试（先失败）**

`tests/transfer_postgres.rs`：

```rust
//! 只在显式配置的一次性本地数据库上运行：TRANSFER_TEST_DATABASE_URL（localhost，库名以 _test 结尾）。
use football_insight_service_backend_rs::transfer::{
    adapters::persistence::PostgresTransferRepository,
    domain::{TransferError, TransferIntent, VoteValue},
    ports::{NominateCommand, TransferCandidateDirectory, TransferRepository, TransferUserAccess},
};
use chrono::{TimeZone, Utc};
use sqlx::{PgPool, postgres::{PgConnectOptions, PgPoolOptions}};
use uuid::Uuid;

struct Db { admin: PgPool, pool: PgPool, schema: String }

async fn setup() -> Db {
    let url = std::env::var("TRANSFER_TEST_DATABASE_URL").expect("explicit disposable database required");
    let parsed = url::Url::parse(&url).unwrap();
    assert!(matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")));
    assert!(parsed.path().ends_with("_test"));
    let admin = PgPool::connect(&url).await.unwrap();
    let schema = format!("transfer_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}")).execute(&admin).await.unwrap();
    let options = url.parse::<PgConnectOptions>().unwrap().options([("search_path", schema.as_str())]);
    let pool = PgPoolOptions::new().max_connections(12).connect_with(options).await.unwrap();
    sqlx::raw_sql("CREATE TABLE f_i_users (id UUID PRIMARY KEY, status TEXT NOT NULL DEFAULT 'active', favorite_team_id BIGINT);
        CREATE TABLE f_i_teams (team_id BIGINT PRIMARY KEY, team_name TEXT NOT NULL, avatar_storage_url TEXT);
        INSERT INTO f_i_teams (team_id, team_name) VALUES (1, 'Alpha'), (2, 'Bravo'), (3, 'Charlie');
        CREATE TABLE f_i_squad_players (team_id BIGINT NOT NULL, player_id BIGINT NOT NULL, player_name TEXT NOT NULL,
            shirt_number INT, position TEXT, avatar_storage_url TEXT, last_season INT NOT NULL, active BOOLEAN NOT NULL, PRIMARY KEY (team_id, player_id));
        -- 10/11/12 本队现役；20 本队旧将现于 Bravo；30 Bravo 现役；31 Charlie 现役
        INSERT INTO f_i_squad_players VALUES
            (1, 10, 'Ten', 10, 'FW', NULL, 2026, true), (1, 11, 'Eleven', 11, 'MF', NULL, 2026, true),
            (1, 12, 'Twelve', 12, 'DF', NULL, 2026, true), (1, 20, 'Twenty', 20, 'GK', NULL, 2025, false),
            (2, 20, 'Twenty', 1, 'GK', NULL, 2026, true), (2, 30, 'Thirty', 9, 'FW', NULL, 2026, true),
            (3, 31, 'ThirtyOne', 7, 'MF', NULL, 2026, true);")
        .execute(&pool).await.unwrap();
    sqlx::raw_sql(include_str!("../migrations/20261020120000_add_transfer_votes.sql")).execute(&pool).await.unwrap();
    Db { admin, pool, schema }
}

async fn teardown(db: Db) {
    db.pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {} CASCADE", db.schema)).execute(&db.admin).await.unwrap();
    db.admin.close().await;
}

async fn fan(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO f_i_users (id, favorite_team_id) VALUES ($1, 1)").bind(id).execute(pool).await.unwrap();
    id
}

fn command(user: Uuid, player: i64, intent: TransferIntent, code: &str) -> NominateCommand {
    NominateCommand { window_id: 1, team_id: 1, player_id: player, intent, user_id: user, share_code: code.into(), limit: 3 }
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn windows_resolve_open_then_latest_closed() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let inside = Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap();
    let after = Utc.with_ymd_and_hms(2027, 4, 1, 0, 0, 0).unwrap();
    let before = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(repo.current_window(inside).await.unwrap().unwrap().code, "2026-winter");
    assert_eq!(repo.current_window(after).await.unwrap().unwrap().code, "2026-winter");
    assert!(repo.current_window(before).await.unwrap().is_none());
    assert!(repo.window_by_code("nope").await.unwrap().is_none());
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn nominating_creates_once_votes_up_and_ranks_by_net_support() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let (a, b, c) = (fan(&db.pool).await, fan(&db.pool).await, fan(&db.pool).await);
    let first = repo.nominate(command(a, 10, TransferIntent::Out, "OUT00010")).await.unwrap();
    assert!(first.created);
    let again = repo.nominate(command(b, 10, TransferIntent::Out, "OUT99999")).await.unwrap();
    assert_eq!((again.created, again.share_code.as_str()), (false, "OUT00010"));
    repo.nominate(command(c, 11, TransferIntent::Out, "OUT00011")).await.unwrap();
    repo.set_vote("OUT00011", a, Some(VoteValue::Up)).await.unwrap();
    repo.set_vote("OUT00010", c, Some(VoteValue::Down)).await.unwrap();
    // OUT00010: up 2 down 1 → net 1；OUT00011: up 2 down 0 → net 2
    let (items, total) = repo.board(1, 1, TransferIntent::Out, 0, 20, Some(c)).await.unwrap();
    assert_eq!(total, 2);
    assert_eq!(items.iter().map(|i| (i.share_code.as_str(), i.up, i.down, i.rank, i.my_vote)).collect::<Vec<_>>(),
        vec![("OUT00011", 2, 0, Some(1), 1), ("OUT00010", 2, 1, Some(2), -1)]);
    let guest = repo.board(1, 1, TransferIntent::Out, 0, 20, None).await.unwrap().0;
    assert!(guest.iter().all(|i| i.my_vote == 0));
    let tally = repo.set_vote("OUT00010", c, None).await.unwrap();
    assert_eq!((tally.up, tally.down, tally.my_vote), (2, 0, 0));
    let context = repo.card("OUT00010", Some(a)).await.unwrap().unwrap();
    assert_eq!((context.team.team_name.as_str(), context.window.code.as_str(), context.card.my_vote), ("Alpha", "2026-winter", 1));
    assert_eq!(context.card.player.name, "Ten");
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn nomination_limit_counts_only_new_cards_per_intent() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let (a, b) = (fan(&db.pool).await, fan(&db.pool).await);
    repo.nominate(command(b, 12, TransferIntent::Out, "BBBBBB12")).await.unwrap();
    for (player, code) in [(10, "AAAAAA10"), (11, "AAAAAA11")] {
        repo.nominate(command(a, player, TransferIntent::Out, code)).await.unwrap();
    }
    // 为别人建的卡加票不占名额
    assert!(!repo.nominate(command(a, 12, TransferIntent::Out, "AAAAAA12")).await.unwrap().created);
    let mut limited = command(a, 10, TransferIntent::Stay, "STAY0010");
    limited.limit = 1;
    repo.nominate(limited.clone()).await.unwrap();
    limited.player_id = 11;
    limited.share_code = "STAY0011".into();
    assert_eq!(repo.nominate(limited).await.unwrap_err(), TransferError::NominationLimit);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn concurrent_nominations_never_exceed_limit_or_duplicate_cards() {
    let db = setup().await;
    let a = fan(&db.pool).await;
    let mut tasks = Vec::new();
    for (index, player) in [10, 11, 12, 10, 11, 12, 10, 11].into_iter().enumerate() {
        let repo = PostgresTransferRepository::new(db.pool.clone());
        let mut cmd = command(a, player, TransferIntent::Out, &format!("CONC{index:04}"));
        cmd.limit = 2;
        tasks.push(tokio::spawn(async move { repo.nominate(cmd).await }));
    }
    for task in tasks { let _ = task.await.unwrap(); }
    let cards: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_transfer_nominations").fetch_one(&db.pool).await.unwrap();
    let votes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM f_i_transfer_votes").fetch_one(&db.pool).await.unwrap();
    assert_eq!(cards, 2);
    assert_eq!(votes, 2);
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn disabled_voters_are_not_counted_and_empty_cards_hide() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let (a, b) = (fan(&db.pool).await, fan(&db.pool).await);
    repo.nominate(command(a, 10, TransferIntent::Out, "HIDE0010")).await.unwrap();
    repo.set_vote("HIDE0010", b, Some(VoteValue::Up)).await.unwrap();
    sqlx::query("UPDATE f_i_users SET status = 'disabled' WHERE id = $1").bind(b).execute(&db.pool).await.unwrap();
    let card = repo.card("HIDE0010", None).await.unwrap().unwrap().card;
    assert_eq!((card.up, card.down), (1, 0));
    assert_eq!(repo.set_vote("HIDE0010", b, None).await.unwrap_err(), TransferError::Unauthorized);
    repo.set_vote("HIDE0010", a, None).await.unwrap();
    assert_eq!(repo.board(1, 1, TransferIntent::Out, 0, 20, None).await.unwrap(), (Vec::new(), 0));
    let hidden = repo.card("HIDE0010", None).await.unwrap().unwrap().card;
    assert_eq!((hidden.up, hidden.down, hidden.rank), (0, 0, None));
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn returning_departed_and_current_team_flags() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let a = fan(&db.pool).await;
    repo.nominate(command(a, 20, TransferIntent::In, "BACK0020")).await.unwrap();
    repo.nominate(command(a, 30, TransferIntent::In, "BUY00030")).await.unwrap();
    repo.nominate(command(a, 12, TransferIntent::Stay, "STAY0012")).await.unwrap();
    let back = repo.card("BACK0020", None).await.unwrap().unwrap().card;
    assert!(back.returning);
    assert_eq!(back.player.current_team.as_ref().map(|t| t.team_name.as_str()), Some("Bravo"));
    assert!(!repo.card("BUY00030", None).await.unwrap().unwrap().card.returning);
    sqlx::query("UPDATE f_i_squad_players SET active = false WHERE team_id = 1 AND player_id = 12").execute(&db.pool).await.unwrap();
    let stay = repo.card("STAY0012", None).await.unwrap().unwrap().card;
    assert!(stay.player.departed);
    assert!(stay.player.current_team.is_none());
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn card_survives_player_removed_from_all_squads() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    let a = fan(&db.pool).await;
    repo.nominate(command(a, 31, TransferIntent::In, "GONE0031")).await.unwrap();
    sqlx::query("DELETE FROM f_i_squad_players WHERE player_id = 31").execute(&db.pool).await.unwrap();
    let card = repo.card("GONE0031", None).await.unwrap().unwrap().card;
    assert_eq!((card.player.name.as_str(), card.player.avatar_url.clone(), card.player.current_team.clone()), ("未知球员", None, None));
    teardown(db).await;
}

#[tokio::test]
#[ignore = "requires explicit disposable local TRANSFER_TEST_DATABASE_URL"]
async fn candidate_directory_and_user_profile() {
    let db = setup().await;
    let repo = PostgresTransferRepository::new(db.pool.clone());
    assert_eq!(repo.find_team(2).await.unwrap().unwrap().team_name, "Bravo");
    assert!(repo.find_team(9).await.unwrap().is_none());
    let mut rows = repo.player_rows(20).await.unwrap();
    rows.sort();
    assert_eq!(rows, vec![(1, false), (2, true)]);
    assert_eq!(repo.team_players(1).await.unwrap().iter().map(|p| p.player_id).collect::<Vec<_>>(), vec![12, 11, 10]);
    let former = repo.former_players(1).await.unwrap();
    assert_eq!((former[0].player_id, former[0].current_team.as_ref().unwrap().team_id), (20, 2));
    assert_eq!(repo.other_active_players(1).await.unwrap().iter().map(|p| p.player_id).collect::<Vec<_>>(), vec![30, 31]);
    let a = fan(&db.pool).await;
    let profile = repo.profile(a).await.unwrap().unwrap();
    assert!(profile.active);
    assert_eq!(profile.favorite_team_id, Some(1));
    assert!(repo.profile(Uuid::new_v4()).await.unwrap().is_none());
    teardown(db).await;
}
```

（`team_players` 按 GK、DF、MF、FW 排序：12 号 DF 在前，11 号 MF，10 号 FW。`other_active_players` 排除本队旧将 20，按队名 Bravo、Charlie。）

- [ ] **Step 2: 运行，确认失败**

Run: `cargo test --test transfer_postgres --no-run 2>&1 | tail -5`
Expected: 编译失败，`adapters::persistence` 不存在。

- [ ] **Step 3: 实现 `PostgresTransferRepository`**

`src/transfer/adapters/mod.rs` 写入 `pub mod persistence;`（Task 4 再加 `dto`、`web`）。`src/transfer/adapters/persistence.rs`：

```rust
use crate::transfer::{
    domain::{
        CandidatePlayer, CardPlayer, NominationCard, TransferError, TransferIntent, TransferTeam,
        TransferWindow, VoteTally, VoteValue, is_departed, is_returning,
    },
    ports::{
        CardContext, NominateCommand, NominateOutcome, TransferCandidateDirectory,
        TransferRepository, TransferUserAccess, UserProfile,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

pub struct PostgresTransferRepository {
    pool: PgPool,
}

impl PostgresTransferRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn unavailable(error: sqlx::Error) -> TransferError {
    if let Some(db) = error.as_database_error()
        && db.code().as_deref() == Some("23505")
        && db.constraint() == Some("uq_f_i_transfer_nominations_share_code")
    {
        return TransferError::ShareCodeTaken;
    }
    tracing::warn!(error = %error, "transfer store unavailable");
    TransferError::Unavailable
}

/// $1 浏览者（可空）$2 期 $3 球队 $4 意向。只统计正常账号的票；只给有票的卡排名。
const RANKED: &str = "WITH tally AS (
    SELECT n.id, n.team_id, n.player_id, n.intent, n.share_code, n.created_at,
        COUNT(v.user_id) FILTER (WHERE v.value = 1) AS up,
        COUNT(v.user_id) FILTER (WHERE v.value = -1) AS down,
        COALESCE(MAX(v.value) FILTER (WHERE v.user_id = $1), 0)::SMALLINT AS my_vote
    FROM f_i_transfer_nominations n
    LEFT JOIN (f_i_transfer_votes v JOIN f_i_users vu ON vu.id = v.user_id AND vu.status = 'active')
        ON v.nomination_id = n.id
    WHERE n.window_id = $2 AND n.team_id = $3 AND n.intent = $4
    GROUP BY n.id
), ranked AS (
    SELECT t.*, CASE WHEN t.up + t.down > 0 THEN ROW_NUMBER() OVER (
        PARTITION BY t.up + t.down > 0
        ORDER BY t.up - t.down DESC, t.up DESC, t.created_at, t.share_code) END AS rank
    FROM tally t
)";

const CARD_SELECT: &str = "SELECT r.share_code, r.intent, r.player_id, r.up, r.down, r.my_vote, r.rank, r.created_at,
    p.player_name, p.avatar_storage_url AS avatar_url, p.position, p.shirt_number,
    ct.team_id AS current_team_id, ct.team_name AS current_team_name, ct.avatar_storage_url AS current_team_avatar,
    EXISTS(SELECT 1 FROM f_i_squad_players f WHERE f.player_id = r.player_id AND f.team_id = r.team_id AND NOT f.active) AS former_of_team,
    EXISTS(SELECT 1 FROM f_i_squad_players a WHERE a.player_id = r.player_id AND a.team_id = r.team_id AND a.active) AS active_in_team
FROM ranked r
LEFT JOIN LATERAL (SELECT sp.player_name, sp.avatar_storage_url, sp.position, sp.shirt_number
    FROM f_i_squad_players sp WHERE sp.player_id = r.player_id
    ORDER BY sp.active DESC, sp.last_season DESC LIMIT 1) p ON TRUE
LEFT JOIN LATERAL (SELECT t.team_id, t.team_name, t.avatar_storage_url
    FROM f_i_squad_players sp JOIN f_i_teams t ON t.team_id = sp.team_id
    WHERE sp.player_id = r.player_id AND sp.active LIMIT 1) ct ON TRUE";

#[derive(FromRow)]
struct CardRow {
    share_code: String,
    intent: String,
    player_id: i64,
    up: i64,
    down: i64,
    my_vote: i16,
    rank: Option<i64>,
    created_at: DateTime<Utc>,
    player_name: Option<String>,
    avatar_url: Option<String>,
    position: Option<String>,
    shirt_number: Option<i32>,
    current_team_id: Option<i64>,
    current_team_name: Option<String>,
    current_team_avatar: Option<String>,
    former_of_team: bool,
    active_in_team: bool,
}

impl CardRow {
    fn into_card(self) -> Result<NominationCard, TransferError> {
        let intent = TransferIntent::parse(&self.intent).ok_or(TransferError::Unavailable)?;
        let current_team = match (self.current_team_id, self.current_team_name) {
            (Some(team_id), Some(team_name)) => Some(TransferTeam { team_id, team_name, avatar_url: self.current_team_avatar }),
            _ => None,
        };
        Ok(NominationCard {
            share_code: self.share_code,
            intent,
            returning: is_returning(intent, self.former_of_team),
            player: CardPlayer {
                player_id: self.player_id,
                name: self.player_name.unwrap_or_else(|| "未知球员".into()),
                avatar_url: self.avatar_url,
                position: self.position,
                shirt_number: self.shirt_number,
                current_team,
                departed: is_departed(intent, self.active_in_team),
            },
            up: self.up,
            down: self.down,
            my_vote: self.my_vote,
            rank: self.rank,
            created_at: self.created_at,
        })
    }
}

#[derive(FromRow)]
struct WindowRow {
    id: i64,
    code: String,
    name: String,
    opens_at: DateTime<Utc>,
    closes_at: DateTime<Utc>,
}

impl From<WindowRow> for TransferWindow {
    fn from(row: WindowRow) -> Self {
        Self { id: row.id, code: row.code, name: row.name, opens_at: row.opens_at, closes_at: row.closes_at }
    }
}

const WINDOW_COLUMNS: &str = "id, code, name, opens_at, closes_at";

async fn lock_active_user(tx: &mut Transaction<'_, Postgres>, user_id: Uuid, exclusive: bool) -> Result<(), TransferError> {
    let lock = if exclusive { "FOR UPDATE" } else { "FOR SHARE" };
    sqlx::query_scalar::<_, Uuid>(&format!("SELECT id FROM f_i_users WHERE id = $1 AND status = 'active' {lock}"))
        .bind(user_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(unavailable)?
        .map(|_| ())
        .ok_or(TransferError::Unauthorized)
}

async fn tally(tx: &mut Transaction<'_, Postgres>, nomination_id: Uuid, user_id: Uuid) -> Result<VoteTally, TransferError> {
    let (up, down, my_vote) = sqlx::query_as::<_, (i64, i64, i16)>(
        "SELECT COUNT(*) FILTER (WHERE v.value = 1), COUNT(*) FILTER (WHERE v.value = -1),
            COALESCE(MAX(v.value) FILTER (WHERE v.user_id = $2), 0)::SMALLINT
         FROM f_i_transfer_votes v JOIN f_i_users u ON u.id = v.user_id AND u.status = 'active'
         WHERE v.nomination_id = $1",
    )
    .bind(nomination_id)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(unavailable)?;
    Ok(VoteTally { up, down, my_vote })
}

#[async_trait]
impl TransferRepository for PostgresTransferRepository {
    async fn current_window(&self, now: DateTime<Utc>) -> Result<Option<TransferWindow>, TransferError> {
        let open = sqlx::query_as::<_, WindowRow>(&format!(
            "SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows WHERE opens_at <= $1 AND $1 < closes_at ORDER BY opens_at DESC LIMIT 1"
        ))
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;
        if let Some(window) = open {
            return Ok(Some(window.into()));
        }
        sqlx::query_as::<_, WindowRow>(&format!(
            "SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows WHERE closes_at <= $1 ORDER BY closes_at DESC LIMIT 1"
        ))
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(unavailable)
    }

    async fn window_by_code(&self, code: &str) -> Result<Option<TransferWindow>, TransferError> {
        sqlx::query_as::<_, WindowRow>(&format!("SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows WHERE code = $1"))
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map(|row| row.map(Into::into))
            .map_err(unavailable)
    }

    async fn board(&self, window_id: i64, team_id: i64, intent: TransferIntent, offset: i64, limit: i64, viewer: Option<Uuid>) -> Result<(Vec<NominationCard>, i64), TransferError> {
        let rows = sqlx::query_as::<_, CardRow>(&format!(
            "{RANKED} {CARD_SELECT} WHERE r.rank IS NOT NULL ORDER BY r.rank LIMIT $5 OFFSET $6"
        ))
        .bind(viewer)
        .bind(window_id)
        .bind(team_id)
        .bind(intent.as_str())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;
        let total: i64 = sqlx::query_scalar(&format!("{RANKED} SELECT COUNT(*) FROM ranked WHERE rank IS NOT NULL"))
            .bind(viewer)
            .bind(window_id)
            .bind(team_id)
            .bind(intent.as_str())
            .fetch_one(&self.pool)
            .await
            .map_err(unavailable)?;
        let items = rows.into_iter().map(CardRow::into_card).collect::<Result<_, _>>()?;
        Ok((items, total))
    }

    async fn card(&self, share_code: &str, viewer: Option<Uuid>) -> Result<Option<CardContext>, TransferError> {
        let head = sqlx::query_as::<_, (i64, i64, String, String, Option<String>)>(
            "SELECT n.window_id, n.team_id, n.intent, t.team_name, t.avatar_storage_url
             FROM f_i_transfer_nominations n JOIN f_i_teams t ON t.team_id = n.team_id WHERE n.share_code = $1",
        )
        .bind(share_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;
        let Some((window_id, team_id, intent, team_name, team_avatar)) = head else { return Ok(None) };
        let window: TransferWindow = sqlx::query_as::<_, WindowRow>(&format!("SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows WHERE id = $1"))
            .bind(window_id)
            .fetch_one(&self.pool)
            .await
            .map_err(unavailable)?
            .into();
        let row = sqlx::query_as::<_, CardRow>(&format!("{RANKED} {CARD_SELECT} WHERE r.share_code = $5"))
            .bind(viewer)
            .bind(window_id)
            .bind(team_id)
            .bind(&intent)
            .bind(share_code)
            .fetch_one(&self.pool)
            .await
            .map_err(unavailable)?;
        Ok(Some(CardContext {
            card: row.into_card()?,
            team: TransferTeam { team_id, team_name, avatar_url: team_avatar },
            window,
        }))
    }

    async fn nominate(&self, command: NominateCommand) -> Result<NominateOutcome, TransferError> {
        let mut tx = self.pool.begin().await.map_err(unavailable)?;
        // 锁提名人：同一人并发提名时串行，名额计数不会被绕过。
        lock_active_user(&mut tx, command.user_id, true).await?;
        let find = "SELECT id, share_code FROM f_i_transfer_nominations
            WHERE window_id = $1 AND team_id = $2 AND player_id = $3 AND intent = $4";
        let existing = sqlx::query_as::<_, (Uuid, String)>(find)
            .bind(command.window_id).bind(command.team_id).bind(command.player_id).bind(command.intent.as_str())
            .fetch_optional(&mut *tx).await.map_err(unavailable)?;
        let (id, share_code, created) = match existing {
            Some((id, code)) => (id, code, false),
            None => {
                let created_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM f_i_transfer_nominations WHERE window_id = $1 AND created_by = $2 AND intent = $3",
                )
                .bind(command.window_id).bind(command.user_id).bind(command.intent.as_str())
                .fetch_one(&mut *tx).await.map_err(unavailable)?;
                // 上限规则在 domain（NOMINATION_LIMIT），由 application 传入；这里只服从传入值。
                if created_count >= command.limit {
                    return Err(TransferError::NominationLimit);
                }
                let inserted = sqlx::query_as::<_, (Uuid, String)>(
                    "INSERT INTO f_i_transfer_nominations (window_id, team_id, player_id, intent, share_code, created_by)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     ON CONFLICT (window_id, team_id, player_id, intent) DO NOTHING RETURNING id, share_code",
                )
                .bind(command.window_id).bind(command.team_id).bind(command.player_id).bind(command.intent.as_str())
                .bind(&command.share_code).bind(command.user_id)
                .fetch_optional(&mut *tx).await.map_err(unavailable)?;
                match inserted {
                    Some((id, code)) => (id, code, true),
                    // 另一位球迷同时提名了同一张卡：沿用对方的卡，给它加票。
                    None => {
                        let (id, code) = sqlx::query_as::<_, (Uuid, String)>(find)
                            .bind(command.window_id).bind(command.team_id).bind(command.player_id).bind(command.intent.as_str())
                            .fetch_one(&mut *tx).await.map_err(unavailable)?;
                        (id, code, false)
                    }
                }
            }
        };
        sqlx::query(
            "INSERT INTO f_i_transfer_votes (nomination_id, user_id, value) VALUES ($1, $2, 1)
             ON CONFLICT (nomination_id, user_id) DO UPDATE SET value = 1, updated_at = NOW()",
        )
        .bind(id).bind(command.user_id)
        .execute(&mut *tx).await.map_err(unavailable)?;
        tx.commit().await.map_err(unavailable)?;
        Ok(NominateOutcome { share_code, created })
    }

    async fn set_vote(&self, share_code: &str, user_id: Uuid, value: Option<VoteValue>) -> Result<VoteTally, TransferError> {
        let mut tx = self.pool.begin().await.map_err(unavailable)?;
        lock_active_user(&mut tx, user_id, false).await?;
        let id: Uuid = sqlx::query_scalar("SELECT id FROM f_i_transfer_nominations WHERE share_code = $1")
            .bind(share_code)
            .fetch_optional(&mut *tx).await.map_err(unavailable)?
            .ok_or(TransferError::NotFound)?;
        match value {
            Some(value) => sqlx::query(
                "INSERT INTO f_i_transfer_votes (nomination_id, user_id, value) VALUES ($1, $2, $3)
                 ON CONFLICT (nomination_id, user_id) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()",
            )
            .bind(id).bind(user_id).bind(value.as_i16()),
            None => sqlx::query("DELETE FROM f_i_transfer_votes WHERE nomination_id = $1 AND user_id = $2")
                .bind(id).bind(user_id),
        }
        .execute(&mut *tx).await.map_err(unavailable)?;
        let result = tally(&mut tx, id, user_id).await?;
        tx.commit().await.map_err(unavailable)?;
        Ok(result)
    }
}

const CANDIDATE_COLUMNS: &str = "sp.player_id, sp.player_name, sp.shirt_number, sp.position, sp.avatar_storage_url";
const POSITION_ORDER: &str = "CASE sp.position WHEN 'GK' THEN 0 WHEN 'DF' THEN 1 WHEN 'MF' THEN 2 WHEN 'FW' THEN 3 ELSE 4 END";

type CandidateRow = (i64, String, Option<i32>, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>);

fn candidate(row: CandidateRow) -> CandidatePlayer {
    let (player_id, name, shirt_number, position, avatar_url, team_id, team_name, team_avatar) = row;
    CandidatePlayer {
        player_id,
        name,
        shirt_number,
        position,
        avatar_url,
        current_team: match (team_id, team_name) {
            (Some(team_id), Some(team_name)) => Some(TransferTeam { team_id, team_name, avatar_url: team_avatar }),
            _ => None,
        },
    }
}

#[async_trait]
impl TransferCandidateDirectory for PostgresTransferRepository {
    async fn find_team(&self, team_id: i64) -> Result<Option<TransferTeam>, TransferError> {
        sqlx::query_as::<_, (i64, String, Option<String>)>("SELECT team_id, team_name, avatar_storage_url FROM f_i_teams WHERE team_id = $1")
            .bind(team_id)
            .fetch_optional(&self.pool)
            .await
            .map(|row| row.map(|(team_id, team_name, avatar_url)| TransferTeam { team_id, team_name, avatar_url }))
            .map_err(unavailable)
    }

    async fn player_rows(&self, player_id: i64) -> Result<Vec<(i64, bool)>, TransferError> {
        sqlx::query_as("SELECT team_id, active FROM f_i_squad_players WHERE player_id = $1")
            .bind(player_id)
            .fetch_all(&self.pool)
            .await
            .map_err(unavailable)
    }

    async fn team_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
        sqlx::query_as::<_, CandidateRow>(&format!(
            "SELECT {CANDIDATE_COLUMNS}, t.team_id, t.team_name, t.avatar_storage_url
             FROM f_i_squad_players sp JOIN f_i_teams t ON t.team_id = sp.team_id
             WHERE sp.team_id = $1 AND sp.active
             ORDER BY {POSITION_ORDER}, sp.shirt_number NULLS LAST, sp.player_id"
        ))
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(candidate).collect())
        .map_err(unavailable)
    }

    async fn former_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
        sqlx::query_as::<_, CandidateRow>(&format!(
            "SELECT {CANDIDATE_COLUMNS}, ct.team_id, ct.team_name, ct.avatar_storage_url
             FROM f_i_squad_players sp
             LEFT JOIN LATERAL (SELECT t.team_id, t.team_name, t.avatar_storage_url
                 FROM f_i_squad_players a JOIN f_i_teams t ON t.team_id = a.team_id
                 WHERE a.player_id = sp.player_id AND a.active LIMIT 1) ct ON TRUE
             WHERE sp.team_id = $1 AND NOT sp.active
             ORDER BY sp.last_season DESC, {POSITION_ORDER}, sp.player_id"
        ))
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(candidate).collect())
        .map_err(unavailable)
    }

    async fn other_active_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
        sqlx::query_as::<_, CandidateRow>(&format!(
            "SELECT {CANDIDATE_COLUMNS}, t.team_id, t.team_name, t.avatar_storage_url
             FROM f_i_squad_players sp JOIN f_i_teams t ON t.team_id = sp.team_id
             WHERE sp.active AND sp.team_id <> $1
               AND NOT EXISTS (SELECT 1 FROM f_i_squad_players f WHERE f.player_id = sp.player_id AND f.team_id = $1)
             ORDER BY t.team_name, {POSITION_ORDER}, sp.shirt_number NULLS LAST, sp.player_id"
        ))
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(candidate).collect())
        .map_err(unavailable)
    }
}

#[async_trait]
impl TransferUserAccess for PostgresTransferRepository {
    async fn profile(&self, user_id: Uuid) -> Result<Option<UserProfile>, TransferError> {
        sqlx::query_as::<_, (String, Option<i64>)>("SELECT status, favorite_team_id FROM f_i_users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map(|row| row.map(|(status, favorite_team_id)| UserProfile { active: status == "active", favorite_team_id }))
            .map_err(unavailable)
    }
}
```

- [ ] **Step 4: 在一次性数据库上运行集成测试**

```bash
docker run -d --rm --name transfer-pg -e POSTGRES_PASSWORD=pg -e POSTGRES_DB=transfer_test -p 55499:5432 postgres:16
sleep 4
TRANSFER_TEST_DATABASE_URL=postgres://postgres:pg@127.0.0.1:55499/transfer_test \
  cargo test --test transfer_postgres -- --ignored --test-threads=4 2>&1 | tail -15
docker stop transfer-pg
```

Expected: `test result: ok. 7 passed`。如果镜像需要 `gen_random_uuid()`：PostgreSQL 13+ 内置，无需扩展。

- [ ] **Step 5: 确认默认测试仍然跳过它们并全部通过**

Run: `cargo test 2>&1 | grep -E "^test result" | tail -5`
Expected: 各行均为 `ok`，`transfer_postgres` 显示 `7 ignored`。

- [ ] **Step 6: 格式化并提交**

```bash
rustfmt --edition 2024 src/transfer/adapters/persistence.rs src/transfer/adapters/mod.rs tests/transfer_postgres.rs
git add src/transfer/adapters tests/transfer_postgres.rs
git commit -m "feat(transfer): add postgres adapter for transfer votes

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Axum 路由、DTO 与装配

**Files:**
- Create: `football_insight_service_backend_rs/src/transfer/adapters/dto.rs`
- Create: `football_insight_service_backend_rs/src/transfer/adapters/web.rs`
- Modify: `football_insight_service_backend_rs/src/transfer/adapters/mod.rs`（加 `pub mod dto; pub mod web;`）
- Modify: `football_insight_service_backend_rs/src/transfer/bootstrap.rs`
- Modify: `football_insight_service_backend_rs/src/app.rs`

**Interfaces:**
- Consumes: Task 2 `TransferService` 与其返回类型；`crate::auth::ports::token_port::TokenPort`。
- Produces: `pub fn transfer_routes(state: Arc<TransferWebState>) -> Router`；`pub struct TransferWebState { pub service: Arc<TransferService>, pub token_port: Arc<dyn TokenPort> }`；`pub fn build_transfer_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router`。
- HTTP 契约（供 Task 5 之后的小程序使用，字段名以此为准）：见 spec 第 6 节；JSON 字段全部 snake_case。

- [ ] **Step 1: 写 DTO**

`src/transfer/adapters/dto.rs`：

```rust
use crate::transfer::{
    application::{BoardSlice, CandidateGroup, Candidates, CardView, NominateResult, TransferBoard, TransferSummary, WindowView},
    domain::{CandidatePlayer, NominationCard, TransferTeam, VoteTally},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WindowQuery {
    pub window: Option<String>,
}
#[derive(Deserialize)]
pub struct BoardQuery {
    pub intent: String,
    #[serde(default = "first_page")]
    pub page: i64,
    pub window: Option<String>,
}
fn first_page() -> i64 {
    1
}
#[derive(Deserialize)]
pub struct IntentQuery {
    pub intent: String,
}
#[derive(Deserialize)]
pub struct NominateBody {
    pub player_id: i64,
    pub intent: String,
}
#[derive(Deserialize)]
pub struct VoteBody {
    pub value: i16,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct TeamDto { pub team_id: i64, pub team_name: String, pub avatar_url: Option<String> }
impl From<TransferTeam> for TeamDto {
    fn from(t: TransferTeam) -> Self { Self { team_id: t.team_id, team_name: t.team_name, avatar_url: t.avatar_url } }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct WindowDto { pub code: String, pub name: String, pub opens_at: DateTime<Utc>, pub closes_at: DateTime<Utc>, pub open: bool }
impl From<WindowView> for WindowDto {
    fn from(v: WindowView) -> Self {
        Self { code: v.window.code, name: v.window.name, opens_at: v.window.opens_at, closes_at: v.window.closes_at, open: v.open }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct PlayerDto {
    pub player_id: i64, pub name: String, pub avatar_url: Option<String>, pub position: Option<String>,
    pub shirt_number: Option<i32>, pub current_team: Option<TeamDto>, pub departed: bool,
}
#[derive(Serialize, Debug, PartialEq)]
pub struct CardDto {
    pub share_code: String, pub intent: &'static str, pub returning: bool, pub player: PlayerDto,
    pub up: i64, pub down: i64, pub my_vote: i16, pub rank: Option<i64>, pub created_at: DateTime<Utc>,
}
impl From<NominationCard> for CardDto {
    fn from(c: NominationCard) -> Self {
        Self {
            share_code: c.share_code, intent: c.intent.as_str(), returning: c.returning,
            player: PlayerDto {
                player_id: c.player.player_id, name: c.player.name, avatar_url: c.player.avatar_url,
                position: c.player.position, shirt_number: c.player.shirt_number,
                current_team: c.player.current_team.map(Into::into), departed: c.player.departed,
            },
            up: c.up, down: c.down, my_vote: c.my_vote, rank: c.rank, created_at: c.created_at,
        }
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CurrentWindowDto { pub window: Option<WindowDto> }

#[derive(Serialize, Debug, PartialEq)]
pub struct SliceDto { pub intent: &'static str, pub total: i64, pub items: Vec<CardDto> }
impl From<BoardSlice> for SliceDto {
    fn from(s: BoardSlice) -> Self { Self { intent: s.intent.as_str(), total: s.total, items: s.items.into_iter().map(Into::into).collect() } }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct SummaryDto { pub team: TeamDto, pub window: Option<WindowDto>, pub can_vote: bool, pub boards: Vec<SliceDto> }
impl From<TransferSummary> for SummaryDto {
    fn from(s: TransferSummary) -> Self {
        Self { team: s.team.into(), window: s.window.map(Into::into), can_vote: s.can_vote, boards: s.boards.into_iter().map(Into::into).collect() }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct BoardDto {
    pub team: TeamDto, pub window: Option<WindowDto>, pub intent: &'static str, pub page: i64,
    pub page_size: i64, pub total: i64, pub can_vote: bool, pub items: Vec<CardDto>,
}
impl From<TransferBoard> for BoardDto {
    fn from(b: TransferBoard) -> Self {
        Self {
            team: b.team.into(), window: b.window.map(Into::into), intent: b.intent.as_str(), page: b.page,
            page_size: b.page_size, total: b.total, can_vote: b.can_vote, items: b.items.into_iter().map(Into::into).collect(),
        }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct CardViewDto { pub card: CardDto, pub team: TeamDto, pub window: WindowDto, pub can_vote: bool }
impl From<CardView> for CardViewDto {
    fn from(v: CardView) -> Self { Self { card: v.card.into(), team: v.team.into(), window: v.window.into(), can_vote: v.can_vote } }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct NominateDto { pub created: bool, pub view: CardViewDto }
impl From<NominateResult> for NominateDto {
    fn from(r: NominateResult) -> Self { Self { created: r.created, view: r.view.into() } }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct CandidateDto {
    pub player_id: i64, pub name: String, pub shirt_number: Option<i32>, pub position: Option<String>,
    pub avatar_url: Option<String>, pub current_team: Option<TeamDto>,
}
impl From<CandidatePlayer> for CandidateDto {
    fn from(p: CandidatePlayer) -> Self {
        Self { player_id: p.player_id, name: p.name, shirt_number: p.shirt_number, position: p.position, avatar_url: p.avatar_url, current_team: p.current_team.map(Into::into) }
    }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct GroupDto { pub key: String, pub title: String, pub players: Vec<CandidateDto> }
impl From<CandidateGroup> for GroupDto {
    fn from(g: CandidateGroup) -> Self { Self { key: g.key, title: g.title, players: g.players.into_iter().map(Into::into).collect() } }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct CandidatesDto { pub intent: &'static str, pub groups: Vec<GroupDto> }
impl From<Candidates> for CandidatesDto {
    fn from(c: Candidates) -> Self { Self { intent: c.intent.as_str(), groups: c.groups.into_iter().map(Into::into).collect() } }
}
#[derive(Serialize, Debug, PartialEq)]
pub struct TallyDto { pub up: i64, pub down: i64, pub my_vote: i16 }
impl From<VoteTally> for TallyDto {
    fn from(t: VoteTally) -> Self { Self { up: t.up, down: t.down, my_vote: t.my_vote } }
}
```

- [ ] **Step 2: 写路由的失败测试**

`src/transfer/adapters/web.rs` 末尾（实现见 Step 4）：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::ports::token_port::{AuthTokenClaims, WechatBindTokenClaims, WechatBindTokenPayload},
        transfer::application::tests::{FAN, at, service_at},
    };
    use axum::{body::{Body, to_bytes}, http::Request};
    use chrono::{DateTime, Utc};
    use tower::ServiceExt;

    struct Tokens;
    impl TokenPort for Tokens {
        fn issue_token(&self, _: Uuid, _: &str, _: DateTime<Utc>) -> anyhow::Result<String> { unimplemented!() }
        fn verify_token(&self, token: &str) -> anyhow::Result<AuthTokenClaims> {
            anyhow::ensure!(token == "good", "invalid");
            Ok(AuthTokenClaims { sub: FAN, account_identifier: "fan".into(), exp: Utc::now() + chrono::Duration::hours(1) })
        }
        fn issue_wechat_bind_token(&self, _: WechatBindTokenPayload, _: DateTime<Utc>) -> anyhow::Result<String> { unimplemented!() }
        fn verify_wechat_bind_token(&self, _: &str) -> anyhow::Result<WechatBindTokenClaims> { unimplemented!() }
    }

    async fn call(day: u32, method: &str, path: &str, auth: Option<&str>, body: Option<&str>) -> (StatusCode, String) {
        let (service, _, _) = service_at(at(day), true);
        let app = transfer_routes(Arc::new(TransferWebState { service: Arc::new(service), token_port: Arc::new(Tokens) }));
        let mut request = Request::builder().method(method).uri(path);
        if let Some(header) = auth { request = request.header(AUTHORIZATION, header); }
        if body.is_some() { request = request.header("content-type", "application/json"); }
        let response = app.oneshot(request.body(body.map(|b| Body::from(b.to_owned())).unwrap_or_else(Body::empty)).unwrap()).await.unwrap();
        assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
        let status = response.status();
        (status, String::from_utf8(to_bytes(response.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap())
    }

    #[tokio::test]
    async fn reads_are_anonymous_and_shaped_for_the_mini_program() {
        let (status, body) = call(5, "GET", "/api/v1/transfers/teams/1/summary", None, None).await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["window"]["code"], "2026-winter");
        assert_eq!(json["window"]["open"], true);
        assert_eq!(json["can_vote"], false);
        assert_eq!(json["boards"][0]["intent"], "out");
        assert_eq!(json["boards"][0]["items"][0]["player"]["current_team"]["team_id"], 1);
        let (status, body) = call(5, "GET", "/api/v1/transfers/teams/1/board?intent=in&page=1", Some("Bearer good"), None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["can_vote"], true);
        assert_eq!(call(5, "GET", "/api/v1/transfers/windows/current", None, None).await.0, StatusCode::OK);
        assert_eq!(call(5, "GET", "/api/v1/transfers/nominations/CODE0001", None, None).await.0, StatusCode::OK);
        assert_eq!(call(5, "GET", "/api/v1/transfers/teams/1/candidates?intent=in", None, None).await.0, StatusCode::OK);
    }

    #[tokio::test]
    async fn writes_require_login_and_map_domain_errors() {
        for (method, path, body) in [
            ("POST", "/api/v1/transfers/teams/1/nominations", Some(r#"{"player_id":10,"intent":"out"}"#)),
            ("PUT", "/api/v1/transfers/nominations/CODE0001/vote", Some(r#"{"value":1}"#)),
            ("DELETE", "/api/v1/transfers/nominations/CODE0001/vote", None),
        ] {
            assert_eq!(call(5, method, path, None, body).await.0, StatusCode::UNAUTHORIZED, "{path}");
            assert_eq!(call(5, method, path, Some("Bearer bad"), body).await.0, StatusCode::UNAUTHORIZED, "{path}");
        }
        let (status, body) = call(5, "POST", "/api/v1/transfers/teams/1/nominations", Some("Bearer good"), Some(r#"{"player_id":10,"intent":"out"}"#)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["created"], true);
        let (status, body) = call(25, "PUT", "/api/v1/transfers/nominations/CODE0001/vote", Some("Bearer good"), Some(r#"{"value":-1}"#)).await;
        assert_eq!((status, body.as_str()), (StatusCode::CONFLICT, "本期投票已结束"));
        assert_eq!(call(5, "PUT", "/api/v1/transfers/nominations/CODE0001/vote", Some("Bearer good"), Some(r#"{"value":2}"#)).await.0, StatusCode::BAD_REQUEST);
        assert_eq!(call(5, "POST", "/api/v1/transfers/teams/1/nominations", Some("Bearer good"), Some(r#"{"player_id":40,"intent":"in"}"#)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn every_error_has_a_status() {
        for (error, status) in [
            (TransferError::Invalid, 400), (TransferError::Unauthorized, 401), (TransferError::NotFavoriteTeam, 403),
            (TransferError::NotFound, 404), (TransferError::WindowClosed, 409), (TransferError::InvalidCandidate, 422),
            (TransferError::NominationLimit, 429), (TransferError::ShareCodeTaken, 503), (TransferError::Unavailable, 503),
        ] {
            assert_eq!(map_error(error).0.as_u16(), status);
        }
    }
}
```

- [ ] **Step 3: 运行，确认失败**

Run: `cargo test --lib transfer::adapters::web 2>&1 | tail -5`
Expected: 编译失败，`transfer_routes` 未定义。

- [ ] **Step 4: 实现路由**

`src/transfer/adapters/web.rs` 测试之前：

```rust
use super::dto::{
    BoardDto, BoardQuery, CandidatesDto, CardViewDto, CurrentWindowDto, IntentQuery, NominateBody,
    NominateDto, SummaryDto, TallyDto, VoteBody, WindowQuery,
};
use crate::{
    auth::ports::token_port::TokenPort,
    transfer::{application::TransferService, domain::{TransferError, VoteValue}},
};
use axum::{
    Json, Router,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header::{AUTHORIZATION, CACHE_CONTROL}},
    middleware::{self, Next},
    response::Response,
    routing::{get, post, put},
};
use std::sync::Arc;
use uuid::Uuid;

pub struct TransferWebState {
    pub service: Arc<TransferService>,
    pub token_port: Arc<dyn TokenPort>,
}

type WebError = (StatusCode, String);
type Shared = State<Arc<TransferWebState>>;

pub fn transfer_routes(state: Arc<TransferWebState>) -> Router {
    Router::new()
        .route("/api/v1/transfers/windows/current", get(current_window))
        .route("/api/v1/transfers/teams/{team_id}/summary", get(summary))
        .route("/api/v1/transfers/teams/{team_id}/board", get(board))
        .route("/api/v1/transfers/teams/{team_id}/candidates", get(candidates))
        .route("/api/v1/transfers/teams/{team_id}/nominations", post(nominate))
        .route("/api/v1/transfers/nominations/{share_code}", get(card))
        .route("/api/v1/transfers/nominations/{share_code}/vote", put(vote).delete(unvote))
        .layer(middleware::from_fn(no_store))
        .with_state(state)
}

async fn no_store(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

pub(super) fn map_error(error: TransferError) -> WebError {
    let status = match error {
        TransferError::Invalid => StatusCode::BAD_REQUEST,
        TransferError::Unauthorized => StatusCode::UNAUTHORIZED,
        TransferError::NotFavoriteTeam => StatusCode::FORBIDDEN,
        TransferError::NotFound => StatusCode::NOT_FOUND,
        TransferError::WindowClosed => StatusCode::CONFLICT,
        TransferError::InvalidCandidate => StatusCode::UNPROCESSABLE_ENTITY,
        TransferError::NominationLimit => StatusCode::TOO_MANY_REQUESTS,
        TransferError::ShareCodeTaken | TransferError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    };
    let message = match error {
        TransferError::ShareCodeTaken => TransferError::Unavailable.to_string(),
        other => other.to_string(),
    };
    (status, message)
}

fn viewer(headers: &HeaderMap, tokens: &dyn TokenPort) -> Result<Option<Uuid>, WebError> {
    let Some(header) = headers.get(AUTHORIZATION) else { return Ok(None) };
    let token = header.to_str().ok().and_then(|v| v.strip_prefix("Bearer ")).ok_or_else(|| map_error(TransferError::Unauthorized))?;
    tokens.verify_token(token).map(|claims| Some(claims.sub)).map_err(|_| map_error(TransferError::Unauthorized))
}

fn required_viewer(headers: &HeaderMap, tokens: &dyn TokenPort) -> Result<Uuid, WebError> {
    viewer(headers, tokens)?.ok_or_else(|| map_error(TransferError::Unauthorized))
}

async fn current_window(State(state): Shared) -> Result<Json<CurrentWindowDto>, WebError> {
    state.service.current_window().await.map(|w| Json(CurrentWindowDto { window: w.map(Into::into) })).map_err(map_error)
}

async fn summary(State(state): Shared, headers: HeaderMap, Path(team_id): Path<i64>, Query(query): Query<WindowQuery>) -> Result<Json<SummaryDto>, WebError> {
    let viewer = viewer(&headers, state.token_port.as_ref())?;
    state.service.summary(team_id, query.window.as_deref(), viewer).await.map(|s| Json(s.into())).map_err(map_error)
}

async fn board(State(state): Shared, headers: HeaderMap, Path(team_id): Path<i64>, Query(query): Query<BoardQuery>) -> Result<Json<BoardDto>, WebError> {
    let viewer = viewer(&headers, state.token_port.as_ref())?;
    state.service.board(team_id, &query.intent, query.page, query.window.as_deref(), viewer).await.map(|b| Json(b.into())).map_err(map_error)
}

async fn candidates(State(state): Shared, Path(team_id): Path<i64>, Query(query): Query<IntentQuery>) -> Result<Json<CandidatesDto>, WebError> {
    state.service.candidates(team_id, &query.intent).await.map(|c| Json(c.into())).map_err(map_error)
}

async fn card(State(state): Shared, headers: HeaderMap, Path(code): Path<String>) -> Result<Json<CardViewDto>, WebError> {
    let viewer = viewer(&headers, state.token_port.as_ref())?;
    state.service.card(&code, viewer).await.map(|v| Json(v.into())).map_err(map_error)
}

async fn nominate(State(state): Shared, headers: HeaderMap, Path(team_id): Path<i64>, Json(body): Json<NominateBody>) -> Result<Json<NominateDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state.service.nominate(user, team_id, body.player_id, &body.intent).await.map(|r| Json(r.into())).map_err(map_error)
}

async fn vote(State(state): Shared, headers: HeaderMap, Path(code): Path<String>, Json(body): Json<VoteBody>) -> Result<Json<TallyDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    let value = VoteValue::from_i16(body.value).ok_or_else(|| map_error(TransferError::Invalid))?;
    state.service.vote(user, &code, Some(value)).await.map(|t| Json(t.into())).map_err(map_error)
}

async fn unvote(State(state): Shared, headers: HeaderMap, Path(code): Path<String>) -> Result<Json<TallyDto>, WebError> {
    let user = required_viewer(&headers, state.token_port.as_ref())?;
    state.service.vote(user, &code, None).await.map(|t| Json(t.into())).map_err(map_error)
}
```

说明：Axum 先运行全部提取器（`Json` 在最后），再进入函数体做 `required_viewer`。所以 body 不合法时直接得到 Axum 的 4xx；body 合法但没有 token 时得到 401。测试里无 token 的写请求都带合法 body，因此断言 401。

- [ ] **Step 5: 装配与注册**

`src/transfer/bootstrap.rs`：

```rust
use super::{
    adapters::{persistence::PostgresTransferRepository, web::{TransferWebState, transfer_routes}},
    application::TransferService,
};
use crate::auth::ports::token_port::TokenPort;
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_transfer_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router {
    let store = Arc::new(PostgresTransferRepository::new(pool));
    let service = TransferService::new(store.clone(), store.clone(), store, Arc::new(chrono::Utc::now));
    transfer_routes(Arc::new(TransferWebState { service: Arc::new(service), token_port }))
}
```

`src/transfer/adapters/mod.rs`：

```rust
pub mod dto;
pub mod persistence;
pub mod web;
```

`src/app.rs`：在 `let lineup_routes = …;` 之后加

```rust
    let transfer_routes =
        crate::transfer::bootstrap::build_transfer_routes(pool.clone(), auth.token_port.clone());
```

并在 `.merge(lineup_routes)` 之后加 `.merge(transfer_routes)`。

- [ ] **Step 6: 运行全部后端测试**

Run: `cargo test 2>&1 | grep -E "^test result|FAILED|panicked" | tail -8`
Expected: 全部 `ok`；`transfer::` 新增 3 个路由测试通过。

- [ ] **Step 7: 本地启动核对路由（只读）**

```bash
cd .. && ./deploy/local-test/up.sh
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:18092/api/v1/transfers/windows/current
```

Expected: 本地库尚未执行迁移时为 `503`（表不存在，日志有 `transfer store unavailable`）——这是预期的；**不要**对共享生产库执行迁移，等负责人确认。

- [ ] **Step 8: 格式化并提交**

```bash
cd football_insight_service_backend_rs
rustfmt --edition 2024 src/transfer/adapters/*.rs src/transfer/bootstrap.rs src/app.rs
git add src/transfer src/app.rs
git commit -m "feat(transfer): expose transfer vote routes

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: `FiReactionBar` 改名与踩数显示

**Files:**
- Rename: `football_insight_mini/src/components/FiLineupReactionBar.vue` → `src/components/FiReactionBar.vue`
- Modify: `src/components/FiLineupCommunityCard.vue`（2 处使用 + import）
- Modify: `src/pages/lineup/view.vue`（1 处使用 + import）
- Test: `football_insight_mini/scripts/reaction-bar.test.ts`

**Interfaces:**
- Produces: `FiReactionBar` 属性 `{ count: number; liked: boolean; disliked?: boolean; pending: boolean; size?: 'sm' | 'md'; dislikeCount?: number }`，事件 `like` / `dislike`。

- [ ] **Step 1: 写失败测试**

`scripts/reaction-bar.test.ts`：

```ts
import { expect, test } from 'bun:test'
import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

const SRC = join(import.meta.dir, '../src')
const read = (path: string) => readFileSync(join(SRC, path), 'utf8')

test('the reaction capsule is a shared, generically named component', () => {
  expect(existsSync(join(SRC, 'components/FiReactionBar.vue'))).toBe(true)
  expect(existsSync(join(SRC, 'components/FiLineupReactionBar.vue'))).toBe(false)
  for (const file of ['components/FiLineupCommunityCard.vue', 'pages/lineup/view.vue']) {
    expect(read(file).includes('FiLineupReactionBar')).toBe(false)
    expect(read(file).includes('FiReactionBar')).toBe(true)
  }
})

test('dislike count is opt-in and rendered next to the dislike icon', () => {
  const source = read('components/FiReactionBar.vue')
  expect(/dislikeCount\?: number/.test(source)).toBe(true)
  expect(/v-if="dislikeCount !== undefined"[^>]*>\{\{ dislikeCount \}\}/.test(source)).toBe(true)
  // 战术板不传踩数，外观不变
  expect(read('components/FiLineupCommunityCard.vue').includes('dislike-count')).toBe(false)
})
```

- [ ] **Step 2: 运行，确认失败**

Run: `cd football_insight_mini && bun test scripts/reaction-bar.test.ts 2>&1 | tail -4`
Expected: `2 fail`

- [ ] **Step 3: 改名并加踩数**

```bash
git mv src/components/FiLineupReactionBar.vue src/components/FiReactionBar.vue
sed -i 's/FiLineupReactionBar/FiReactionBar/g' src/components/FiLineupCommunityCard.vue src/pages/lineup/view.vue
```

`src/components/FiReactionBar.vue`：在点踩按钮里图标之后加一行；props 增加 `dislikeCount?: number`；首行注释改为通用描述：

```vue
  <!-- 浅底描边胶囊：左赞（带赞数）右踩（可选踩数），互斥；sm 用于列表行，md 用于详情页按钮行。战术板与转会投票共用。 -->
```

```vue
      <image class="reaction__icon" :src="disliked ? ICONS.dislikeActive : ICONS.dislikeIdle" mode="aspectFit" />
      <text v-if="dislikeCount !== undefined" class="reaction__count">{{ dislikeCount }}</text>
```

```ts
const props = withDefaults(defineProps<{ count: number; liked: boolean; disliked?: boolean; pending: boolean; size?: 'sm' | 'md'; dislikeCount?: number }>(), {
  disliked: false,
  size: 'sm',
  dislikeCount: undefined,
})
```

点踩按钮在显示数字时需要与左侧同样的内边距（否则数字挤在固定内边距里）：在样式里加

```css
.reaction--dislike-count { padding: 0 var(--fi-space-16); }
.reactions--md .reaction--dislike-count { padding: 0 var(--fi-space-24); }
```

并给点踩按钮的 `:class` 加 `'reaction--dislike-count': dislikeCount !== undefined`。

- [ ] **Step 4: 运行测试与检查**

Run: `bun test scripts/reaction-bar.test.ts scripts/style-token-audit.test.mjs 2>&1 | tail -3 && bun run type-check 2>&1 | tail -1 && bun run verify:mp-components | tail -1`
Expected: 全部通过；`All mini-program custom components are registered.`

- [ ] **Step 5: 提交**

```bash
git add -A src/components/FiReactionBar.vue src/components/FiLineupReactionBar.vue src/components/FiLineupCommunityCard.vue src/pages/lineup/view.vue scripts/reaction-bar.test.ts
git commit -m "refactor(mini): share the reaction capsule and allow a dislike count

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: 类型、接口、链接工具与赞踩组合函数

**Files:**
- Create: `football_insight_mini/src/types/transfer.ts`
- Create: `football_insight_mini/src/api/transfer.ts`
- Create: `football_insight_mini/src/utils/transferLinks.ts`
- Create: `football_insight_mini/src/utils/transferLinks.test.ts`
- Create: `football_insight_mini/src/composables/useTransferVotes.ts`
- Create: `football_insight_mini/src/composables/useTransferVotes.test.ts`

**Interfaces:**
- Consumes: Task 4 HTTP 契约。
- Produces:
  - 类型：`TransferIntent`、`TransferTeam`、`TransferWindow`、`TransferPlayer`、`TransferCard`、`TransferTally`、`TransferBoardSlice`、`TransferSummary`、`TransferBoard`、`TransferCardView`、`TransferCandidate`、`TransferCandidateGroup`、`TransferCandidates`、`TransferNominateResult`、`TransferVoteValue = 1 | -1`
  - API：`getTransferSummary(teamId)`、`getTransferBoard(teamId, intent, page)`、`getTransferCard(code)`、`getTransferCandidates(teamId, intent)`、`nominateTransfer(teamId, playerId, intent)`、`setTransferVote(code, value: 1 | -1 | 0)`
  - 链接：`TRANSFER_INTENTS`、`TRANSFER_BOARD_LABELS`、`buildTransferBoardUrl(teamId, intent?, options?)`、`buildTransferViewUrl(code)`、`parseTransferBoardQuery(query)`、`parseTransferCode(query)`、`validTransferCode(code)`、`transferStance(card)`、`transferBoardShareTitle(teamName, windowName, intent)`、`transferCardShareTitle(card)`、`applyTally(card, tally)`、`positionLabel(position)`
  - 组合函数：`createTransferVotes(deps)` → `{ pending: Ref<string[]>, toggle(card, value, options), reset() }`；`transferVoteDeps(overrides?)`

- [ ] **Step 1: 写类型与接口**

`src/types/transfer.ts`：

```ts
export type TransferIntent = 'out' | 'stay' | 'in'
export type TransferVoteValue = 1 | -1
export interface TransferTeam { team_id: number; team_name: string; avatar_url: string | null }
export interface TransferWindow { code: string; name: string; opens_at: string; closes_at: string; open: boolean }
export interface TransferPlayer {
  player_id: number
  name: string
  avatar_url: string | null
  position: 'GK' | 'DF' | 'MF' | 'FW' | null
  shirt_number: number | null
  current_team: TransferTeam | null
  departed: boolean
}
export interface TransferCard {
  share_code: string
  intent: TransferIntent
  returning: boolean
  player: TransferPlayer
  up: number
  down: number
  my_vote: -1 | 0 | 1
  rank: number | null
  created_at: string
}
export interface TransferTally { up: number; down: number; my_vote: -1 | 0 | 1 }
export interface TransferBoardSlice { intent: TransferIntent; total: number; items: TransferCard[] }
export interface TransferSummary { team: TransferTeam; window: TransferWindow | null; can_vote: boolean; boards: TransferBoardSlice[] }
export interface TransferBoard {
  team: TransferTeam
  window: TransferWindow | null
  intent: TransferIntent
  page: number
  page_size: number
  total: number
  can_vote: boolean
  items: TransferCard[]
}
export interface TransferCardView { card: TransferCard; team: TransferTeam; window: TransferWindow; can_vote: boolean }
export interface TransferCandidate {
  player_id: number
  name: string
  shirt_number: number | null
  position: TransferPlayer['position']
  avatar_url: string | null
  current_team: TransferTeam | null
}
export interface TransferCandidateGroup { key: string; title: string; players: TransferCandidate[] }
export interface TransferCandidates { intent: TransferIntent; groups: TransferCandidateGroup[] }
export interface TransferNominateResult { created: boolean; view: TransferCardView }
```

`src/api/transfer.ts`：

```ts
import type { TransferBoard, TransferCandidates, TransferCardView, TransferIntent, TransferNominateResult, TransferSummary, TransferTally } from '../types/transfer'
import { request } from '../utils/request'

// auth: true 表示"有 token 就带上"：游客也能读，登录后多返回 my_vote / can_vote。
export function getTransferSummary(teamId: number): Promise<TransferSummary> {
  return request({ url: `/transfers/teams/${teamId}/summary`, auth: true })
}
export function getTransferBoard(teamId: number, intent: TransferIntent, page = 1): Promise<TransferBoard> {
  return request({ url: `/transfers/teams/${teamId}/board?intent=${intent}&page=${page}`, auth: true })
}
export function getTransferCard(code: string): Promise<TransferCardView> {
  return request({ url: `/transfers/nominations/${encodeURIComponent(code)}`, auth: true })
}
export function getTransferCandidates(teamId: number, intent: TransferIntent): Promise<TransferCandidates> {
  return request({ url: `/transfers/teams/${teamId}/candidates?intent=${intent}` })
}
export function nominateTransfer(teamId: number, playerId: number, intent: TransferIntent): Promise<TransferNominateResult> {
  return request({ url: `/transfers/teams/${teamId}/nominations`, method: 'POST', auth: true, data: { player_id: playerId, intent } })
}
/** 0 表示取消投票。 */
export function setTransferVote(code: string, value: 1 | -1 | 0): Promise<TransferTally> {
  const url = `/transfers/nominations/${encodeURIComponent(code)}/vote`
  return value === 0
    ? request({ url, method: 'DELETE', auth: true })
    : request({ url, method: 'PUT', auth: true, data: { value } })
}
```

- [ ] **Step 2: 写链接工具的失败测试**

`src/utils/transferLinks.test.ts`：

```ts
import { describe, expect, test } from 'bun:test'
import type { TransferCard } from '../types/transfer'
import {
  applyTally, buildTransferBoardUrl, buildTransferViewUrl, parseTransferBoardQuery, parseTransferCode,
  positionLabel, transferBoardShareTitle, transferCardShareTitle, transferStance, validTransferCode,
} from './transferLinks'

function card(overrides: Partial<TransferCard> = {}): TransferCard {
  return {
    share_code: 'ABCD1234', intent: 'out', returning: false, up: 3, down: 1, my_vote: 0, rank: 1, created_at: '',
    player: { player_id: 1, name: '某某', avatar_url: null, position: 'FW', shirt_number: 9, current_team: null, departed: false },
    ...overrides,
  }
}

describe('transfer links', () => {
  test('board and view urls round trip', () => {
    expect(buildTransferBoardUrl(77680, 'in')).toBe('/pages/transfer/board?teamId=77680&intent=in')
    expect(buildTransferBoardUrl(77680, 'out', { nominate: true })).toBe('/pages/transfer/board?teamId=77680&intent=out&nominate=1')
    expect(buildTransferBoardUrl(0)).toBe('/pages/transfer/board')
    expect(parseTransferBoardQuery({ teamId: '77680', intent: 'stay', nominate: '1' })).toEqual({ teamId: 77680, intent: 'stay', nominate: true })
    expect(parseTransferBoardQuery({ teamId: 'x', intent: 'nope' })).toEqual({ teamId: 0, intent: 'out', nominate: false })
    expect(buildTransferViewUrl('ABCD1234')).toBe('/pages/transfer/view?code=ABCD1234')
    expect(buildTransferViewUrl('bad')).toBe('')
    expect(parseTransferCode({ code: 'ABCD1234' })).toBe('ABCD1234')
    expect(parseTransferCode({ code: '../x' })).toBe('')
    expect(validTransferCode('abc')).toBe(false)
  })

  test('stance wording covers returning former players', () => {
    expect(transferStance(card())).toBe('离队')
    expect(transferStance(card({ intent: 'stay' }))).toBe('留下')
    expect(transferStance(card({ intent: 'in' }))).toBe('加盟')
    expect(transferStance(card({ intent: 'in', returning: true }))).toBe('回归')
  })

  test('share titles', () => {
    expect(transferBoardShareTitle('成都蓉城', '2026 冬窗', 'out')).toBe('成都蓉城球迷最想送走谁？2026 冬窗转会榜')
    expect(transferBoardShareTitle('成都蓉城', '2026 冬窗', 'stay')).toBe('成都蓉城球迷最想留下谁？2026 冬窗转会榜')
    expect(transferBoardShareTitle('成都蓉城', '2026 冬窗', 'in')).toBe('成都蓉城球迷最想买入谁？2026 冬窗转会榜')
    expect(transferCardShareTitle(card({ intent: 'in', returning: true }))).toBe('我希望 某某 回归，你同意吗？')
  })

  test('tally replaces counts and my vote only', () => {
    const next = applyTally(card(), { up: 4, down: 0, my_vote: 1 })
    expect([next.up, next.down, next.my_vote, next.rank, next.share_code]).toEqual([4, 0, 1, 1, 'ABCD1234'])
  })

  test('position labels', () => {
    expect([positionLabel('GK'), positionLabel('DF'), positionLabel('MF'), positionLabel('FW'), positionLabel(null)]).toEqual(['门将', '后卫', '中场', '前锋', ''])
  })
})
```

- [ ] **Step 3: 运行，确认失败**

Run: `bun test src/utils/transferLinks.test.ts 2>&1 | tail -3`
Expected: 失败，模块不存在。

- [ ] **Step 4: 实现链接工具**

`src/utils/transferLinks.ts`：

```ts
import type { TransferCard, TransferIntent, TransferPlayer, TransferTally } from '../types/transfer'

/** 转会投票页面路径与文案：洞察页、首页、分包页面共用，页面之间不互相引用。 */
export const TRANSFER_INTENTS: TransferIntent[] = ['out', 'stay', 'in']
export const TRANSFER_BOARD_LABELS: Record<TransferIntent, string> = { out: '离队榜', stay: '留队榜', in: '引援榜' }
const SHARE_VERBS: Record<TransferIntent, string> = { out: '送走', stay: '留下', in: '买入' }
const POSITION_LABELS: Record<NonNullable<TransferPlayer['position']>, string> = { GK: '门将', DF: '后卫', MF: '中场', FW: '前锋' }

export function validTransferCode(code: unknown): code is string {
  return typeof code === 'string' && /^[a-zA-Z0-9]{8}$/.test(code)
}
function parseIntent(value: unknown): TransferIntent {
  return value === 'stay' || value === 'in' ? value : 'out'
}

export function buildTransferBoardUrl(teamId: number, intent: TransferIntent = 'out', options: { nominate?: boolean } = {}): string {
  if (!Number.isInteger(teamId) || teamId <= 0) return '/pages/transfer/board'
  return `/pages/transfer/board?teamId=${teamId}&intent=${intent}${options.nominate ? '&nominate=1' : ''}`
}
export function parseTransferBoardQuery(query: Record<string, unknown> | undefined): { teamId: number; intent: TransferIntent; nominate: boolean } {
  const raw = typeof query?.teamId === 'string' ? Number(query.teamId) : 0
  return { teamId: Number.isInteger(raw) && raw > 0 ? raw : 0, intent: parseIntent(query?.intent), nominate: query?.nominate === '1' }
}
export function buildTransferViewUrl(code: string): string {
  return validTransferCode(code) ? `/pages/transfer/view?code=${code}` : ''
}
export function parseTransferCode(query: Record<string, unknown> | undefined): string {
  return validTransferCode(query?.code) ? (query!.code as string) : ''
}

/** "我希望 X ___"里的动词。 */
export function transferStance(card: Pick<TransferCard, 'intent' | 'returning'>): string {
  if (card.intent === 'out') return '离队'
  if (card.intent === 'stay') return '留下'
  return card.returning ? '回归' : '加盟'
}
export function transferBoardShareTitle(teamName: string, windowName: string, intent: TransferIntent): string {
  return `${teamName}球迷最想${SHARE_VERBS[intent]}谁？${windowName}转会榜`
}
export function transferCardShareTitle(card: TransferCard): string {
  return `我希望 ${card.player.name} ${transferStance(card)}，你同意吗？`
}
export function applyTally(card: TransferCard, tally: TransferTally): TransferCard {
  return { ...card, up: tally.up, down: tally.down, my_vote: tally.my_vote }
}
export function positionLabel(position: TransferPlayer['position']): string {
  return position ? POSITION_LABELS[position] : ''
}
```

- [ ] **Step 5: 运行，确认通过**

Run: `bun test src/utils/transferLinks.test.ts 2>&1 | tail -3`
Expected: `5 pass`

- [ ] **Step 6: 写赞踩组合函数的失败测试**

`src/composables/useTransferVotes.test.ts`：

```ts
import { describe, expect, test } from 'bun:test'
import type { TransferCard, TransferTally } from '../types/transfer'
import { ApiRequestError } from '../utils/apiError'
import { createTransferVotes, type TransferVoteDeps } from './useTransferVotes'

function card(my_vote: -1 | 0 | 1 = 0): TransferCard {
  return {
    share_code: 'ABCD1234', intent: 'out', returning: false, up: 2, down: 1, my_vote, rank: 1, created_at: '',
    player: { player_id: 1, name: 'X', avatar_url: null, position: null, shirt_number: null, current_team: null, departed: false },
  }
}
function deps(overrides: Partial<TransferVoteDeps> = {}) {
  const calls: Array<[string, number]> = [], logins: string[] = [], invalidated: number[] = []
  let revision = 0
  const base: TransferVoteDeps = {
    vote: async (code, value) => { calls.push([code, value]); return { up: 3, down: 0, my_vote: value } as TransferTally },
    loggedIn: () => true,
    revision: () => revision,
    invalidate: r => { invalidated.push(r) },
    login: url => { logins.push(url) },
    ...overrides,
  }
  return { base, calls, logins, invalidated, bump: () => { revision++ } }
}
const always = () => true

describe('transfer votes', () => {
  test('tapping the same side clears it, the other side replaces it', async () => {
    const d = deps(); const votes = createTransferVotes(d.base); const published: TransferTally[] = []
    await votes.toggle(card(0), 1, { returnUrl: '/u', current: always, publish: t => published.push(t) })
    await votes.toggle(card(1), 1, { returnUrl: '/u', current: always, publish: t => published.push(t) })
    await votes.toggle(card(1), -1, { returnUrl: '/u', current: always, publish: t => published.push(t) })
    expect(d.calls).toEqual([['ABCD1234', 1], ['ABCD1234', 0], ['ABCD1234', -1]])
    expect(published.length).toBe(3)
  })

  test('guest_vote_remembers_the_card_url', async () => {
    const d = deps({ loggedIn: () => false }); const votes = createTransferVotes(d.base)
    expect(await votes.toggle(card(), 1, { returnUrl: '/pages/transfer/view?code=ABCD1234', current: always, publish: () => {} })).toBe('')
    expect(d.logins).toEqual(['/pages/transfer/view?code=ABCD1234'])
    expect(d.calls).toEqual([])
  })

  test('a card is locked while its request is in flight', async () => {
    let release: (t: TransferTally) => void = () => {}
    const d = deps({ vote: async () => new Promise<TransferTally>(r => { release = r }) })
    const votes = createTransferVotes(d.base)
    const first = votes.toggle(card(), 1, { returnUrl: '/u', current: always, publish: () => {} })
    expect(votes.pending.value).toEqual(['ABCD1234'])
    expect(await votes.toggle(card(), -1, { returnUrl: '/u', current: always, publish: () => {} })).toBe('')
    release({ up: 1, down: 0, my_vote: 1 }); await first
    expect(votes.pending.value).toEqual([])
  })

  test('server_rejection_keeps_tally_and_returns_message', async () => {
    const d = deps({ vote: async () => { throw new ApiRequestError('本期投票已结束', 409) } })
    const votes = createTransferVotes(d.base); const published: TransferTally[] = []
    expect(await votes.toggle(card(), 1, { returnUrl: '/u', current: always, publish: t => published.push(t) })).toBe('本期投票已结束')
    expect(published).toEqual([])
  })

  test('401 invalidates the session and asks to log in', async () => {
    const d = deps({ vote: async () => { throw new ApiRequestError('请登录后继续', 401) } })
    const votes = createTransferVotes(d.base)
    expect(await votes.toggle(card(), 1, { returnUrl: '/u', current: always, publish: () => {} })).toBe('')
    expect(d.invalidated).toEqual([0]); expect(d.logins).toEqual(['/u'])
  })

  test('a response from an older identity or context is dropped', async () => {
    let release: (t: TransferTally) => void = () => {}
    const d = deps({ vote: async () => new Promise<TransferTally>(r => { release = r }) })
    const votes = createTransferVotes(d.base); const published: TransferTally[] = []
    const pending = votes.toggle(card(), 1, { returnUrl: '/u', current: always, publish: t => published.push(t) })
    d.bump(); release({ up: 1, down: 0, my_vote: 1 }); await pending
    expect(published).toEqual([])
    let stillCurrent = true
    const second = votes.toggle(card(), 1, { returnUrl: '/u', current: () => stillCurrent, publish: t => published.push(t) })
    stillCurrent = false; release({ up: 1, down: 0, my_vote: 1 }); await second
    expect(published).toEqual([])
  })
})
```

- [ ] **Step 7: 运行，确认失败**

Run: `bun test src/composables/useTransferVotes.test.ts 2>&1 | tail -3`
Expected: 失败，模块不存在。

- [ ] **Step 8: 实现组合函数**

`src/composables/useTransferVotes.ts`：

```ts
import { ref } from 'vue'
import { setTransferVote } from '../api/transfer'
import { useAppSession } from '../stores/appSession'
import type { TransferCard, TransferTally, TransferVoteValue } from '../types/transfer'
import { extractApiErrorMessage, isUnauthorizedError } from '../utils/apiError'
import { rememberPostLoginRedirect } from '../utils/postLoginRedirect'

export interface TransferVoteDeps {
  vote: (code: string, value: 1 | -1 | 0) => Promise<TransferTally>
  loggedIn: () => boolean
  revision: () => number
  invalidate: (revision: number) => void
  login: (returnUrl: string) => void
}

export function transferVoteDeps(overrides: Partial<TransferVoteDeps> = {}): TransferVoteDeps {
  const session = overrides.revision ? null : useAppSession()
  return {
    vote: setTransferVote,
    // 登录过程中旧 token 可能还在，此时不发写请求。
    loggedIn: () => Boolean(session?.hasAccessToken.value) && session?.status.value !== 'loading',
    revision: () => session?.revision.value ?? 0,
    invalidate: revision => session?.invalidateSession(revision),
    login: returnUrl => {
      rememberPostLoginRedirect({ type: returnUrl === '/pages/insights/index' ? 'switchTab' : 'navigateTo', url: returnUrl })
      uni.showToast({ title: '请登录后继续', icon: 'none' })
      uni.switchTab({ url: '/pages/user/index' })
    },
    ...overrides,
  }
}

export interface ToggleOptions {
  returnUrl: string
  /** 调用方上下文（球队、页面）是否仍有效 */
  current: () => boolean
  publish: (tally: TransferTally) => void
}

/** 赞踩：点同一侧取消，点另一侧改投；只发布服务端确认的结果，失败保留原状态。 */
export function createTransferVotes(deps: TransferVoteDeps) {
  const pending = ref<string[]>([])
  let generation = 0
  function reset(): void { generation++; pending.value = [] }
  async function toggle(card: TransferCard, value: TransferVoteValue, options: ToggleOptions): Promise<string> {
    const code = card.share_code
    if (pending.value.includes(code)) return ''
    if (!deps.loggedIn()) { deps.login(options.returnUrl); return '' }
    const owner = generation, revision = deps.revision()
    const valid = () => options.current() && generation === owner && deps.revision() === revision
    pending.value = [...pending.value, code]
    try {
      const tally = await deps.vote(code, card.my_vote === value ? 0 : value)
      if (valid()) options.publish(tally)
      return ''
    } catch (error) {
      if (!valid()) return ''
      if (isUnauthorizedError(error)) { deps.invalidate(revision); deps.login(options.returnUrl); return '' }
      return extractApiErrorMessage(error, '投票失败，请重试')
    } finally {
      if (owner === generation) pending.value = pending.value.filter(item => item !== code)
    }
  }
  return { pending, toggle, reset }
}
```

- [ ] **Step 9: 运行，确认通过并检查边界**

Run: `bun test src/composables/useTransferVotes.test.ts src/utils/transferLinks.test.ts 2>&1 | tail -3 && bun run type-check 2>&1 | tail -1 && bun run check:boundaries | tail -1`
Expected: `11 pass`；类型通过；`✅ 依赖边界检查通过`

- [ ] **Step 10: 提交**

```bash
git add src/types/transfer.ts src/api/transfer.ts src/utils/transferLinks.ts src/utils/transferLinks.test.ts src/composables/useTransferVotes.ts src/composables/useTransferVotes.test.ts
git commit -m "feat(mini): add transfer vote api, links and vote composable

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: `FiTransferCard` 与洞察页"转会"胶囊

**Files:**
- Create: `football_insight_mini/src/components/FiTransferCard.vue`
- Create: `football_insight_mini/src/pages/insights/useInsightsTransfer.ts`
- Create: `football_insight_mini/src/pages/insights/useInsightsTransfer.test.ts`
- Create: `football_insight_mini/src/pages/insights/components/InsightsTransferPanel.vue`
- Modify: `football_insight_mini/src/pages/insights/index.vue`

**Interfaces:**
- Consumes: Task 5 `FiReactionBar`；Task 6 类型、API、`transferLinks`、`createTransferVotes`、`transferVoteDeps`。
- Produces:
  - `FiTransferCard` 属性 `{ card: TransferCard; pending: boolean; showRank?: boolean; size?: 'sm' | 'md' }`，事件 `open`、`vote(value: 1 | -1)`。
  - `useInsightsTransfer({ session, selectedTeamId, active, deps? })` → `{ summary, loading, error, hint, pending, refresh(), vote(card, value), dispose() }`
  - `InsightsTransferPanel` 属性 `{ summary: TransferSummary | null; loading: boolean; error: string; hint: string; pending: string[] }`，事件 `retry`、`vote(card, value)`、`open(code)`、`more(intent)`、`nominate`。

- [ ] **Step 1: 写 `FiTransferCard`**

`src/components/FiTransferCard.vue`：

```vue
<template>
  <!-- 一行转会提名：名次、头像、名字与位置/现属球队、标签、赞踩（显示赞数与踩数）。洞察页与转会分包共用。 -->
  <view class="transfer-card" :class="`transfer-card--${size}`" hover-class="transfer-card--pressed" @tap="emit('open')">
    <text v-if="showRank && card.rank !== null" class="transfer-card__rank">{{ card.rank }}</text>
    <image v-if="card.player.avatar_url" class="transfer-card__avatar" :src="card.player.avatar_url" mode="aspectFill" />
    <view v-else class="transfer-card__avatar transfer-card__avatar--fallback"><text>{{ card.player.name.slice(0, 1) }}</text></view>
    <view class="transfer-card__body">
      <view class="transfer-card__title">
        <text class="transfer-card__name">{{ card.player.name }}</text>
        <text v-if="card.returning" class="transfer-card__tag transfer-card__tag--accent">希望回归</text>
        <text v-if="card.player.departed" class="transfer-card__tag">已离队</text>
      </view>
      <text class="transfer-card__meta">{{ meta }}</text>
    </view>
    <view class="transfer-card__reaction">
      <FiReactionBar :size="size" :count="card.up" :dislike-count="card.down" :liked="card.my_vote === 1" :disliked="card.my_vote === -1"
        :pending="pending" @like="emit('vote', 1)" @dislike="emit('vote', -1)" />
    </view>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import FiReactionBar from './FiReactionBar.vue'
import type { TransferCard } from '../types/transfer'
import { positionLabel } from '../utils/transferLinks'
const props = withDefaults(defineProps<{ card: TransferCard; pending: boolean; showRank?: boolean; size?: 'sm' | 'md' }>(), { showRank: true, size: 'sm' })
const emit = defineEmits<{ (e: 'open'): void; (e: 'vote', value: 1 | -1): void }>()
const meta = computed(() => [positionLabel(props.card.player.position), props.card.player.current_team?.team_name ?? '暂无球队'].filter(Boolean).join(' · '))
</script>
<style scoped>
.transfer-card { display: flex; align-items: center; gap: var(--fi-space-16); padding: var(--fi-space-16) 0; }
.transfer-card--pressed { opacity: var(--fi-opacity-pressed); }
.transfer-card__rank { flex-shrink: 0; width: var(--fi-space-32); color: var(--fi-color-primary); font-size: var(--fi-font-26); font-weight: var(--fi-weight-extrabold); text-align: center; }
.transfer-card__avatar { flex-shrink: 0; width: var(--fi-space-64); height: var(--fi-space-64); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); }
.transfer-card--sm .transfer-card__avatar { width: var(--fi-space-48); height: var(--fi-space-48); }
.transfer-card__avatar--fallback { display: flex; align-items: center; justify-content: center; color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); font-weight: var(--fi-weight-bold); }
.transfer-card__body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: var(--fi-space-4); }
.transfer-card__title { display: flex; align-items: center; gap: var(--fi-space-8); min-width: 0; }
.transfer-card__name { min-width: 0; overflow: hidden; color: var(--fi-color-text-strong); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); text-overflow: ellipsis; white-space: nowrap; }
.transfer-card--md .transfer-card__name { font-size: var(--fi-font-30); }
.transfer-card__tag { flex-shrink: 0; padding: var(--fi-space-2) var(--fi-space-10); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); color: var(--fi-color-text-secondary); font-size: var(--fi-font-20); font-weight: var(--fi-weight-bold); }
.transfer-card__tag--accent { background: var(--fi-color-reaction-like-bg); color: var(--fi-color-primary); }
.transfer-card__meta { overflow: hidden; color: var(--fi-color-text-muted); font-size: var(--fi-font-22); text-overflow: ellipsis; white-space: nowrap; }
.transfer-card__reaction { flex-shrink: 0; }
</style>
```

- [ ] **Step 2: 写 `useInsightsTransfer` 的失败测试**

`src/pages/insights/useInsightsTransfer.test.ts`：

```ts
import { describe, expect, test } from 'bun:test'
import { nextTick, ref } from 'vue'
import type { TransferSummary } from '../../types/transfer'
import { useInsightsTransfer, type InsightsTransferOptions } from './useInsightsTransfer'

function summary(teamId: number, canVote = true): TransferSummary {
  return {
    team: { team_id: teamId, team_name: `T${teamId}`, avatar_url: null },
    window: { code: 'w', name: '2026 冬窗', opens_at: '', closes_at: '', open: true },
    can_vote: canVote,
    boards: [{ intent: 'out', total: 1, items: [{
      share_code: 'ABCD1234', intent: 'out', returning: false, up: 1, down: 0, my_vote: 0, rank: 1, created_at: '',
      player: { player_id: 1, name: 'X', avatar_url: null, position: null, shirt_number: null, current_team: null, departed: false },
    }] }, { intent: 'stay', total: 0, items: [] }, { intent: 'in', total: 0, items: [] }],
  }
}
// 只提供 useInsightsTransfer 用到的会话字段；整体断言为参数类型以通过 type-check（测试文件在 tsconfig 范围内）。
function session(loggedIn = true): InsightsTransferOptions['session'] {
  return {
    currentUser: ref(loggedIn ? { id: 'u' } : null), hasAccessToken: ref(loggedIn),
    status: ref(loggedIn ? 'authenticated' : 'guest'),
    revision: ref(0), invalidateSession: () => {},
  } as unknown as InsightsTransferOptions['session']
}
const flush = async () => { await nextTick(); await Promise.resolve(); await Promise.resolve(); await nextTick() }

describe('insights transfer', () => {
  test('loads only while active and follows the selected team', async () => {
    const calls: number[] = []
    const active = ref(false), team = ref<number | null>(1)
    const state = useInsightsTransfer({ session: session(), selectedTeamId: team, active, deps: { summary: async id => { calls.push(id); return summary(id) } } })
    await flush(); expect(calls).toEqual([])
    active.value = true; await flush(); expect(calls).toEqual([1])
    team.value = 2; await flush(); expect(calls).toEqual([1, 2])
    expect(state.summary.value?.team.team_id).toBe(2)
    state.dispose()
  })

  test('a slow response for the previous team is dropped', async () => {
    let releaseFirst: (s: TransferSummary) => void = () => {}
    const team = ref<number | null>(1)
    const state = useInsightsTransfer({ session: session(), selectedTeamId: team, active: ref(true), deps: {
      summary: id => id === 1 ? new Promise(r => { releaseFirst = r }) : Promise.resolve(summary(2)),
    } })
    await flush(); team.value = 2; await flush()
    releaseFirst(summary(1)); await flush()
    expect(state.summary.value?.team.team_id).toBe(2)
    state.dispose()
  })

  test('votes patch the card; non-fans get a hint instead of a request', async () => {
    const votes: Array<[string, number]> = [], notices: string[] = []
    const fan = useInsightsTransfer({ session: session(), selectedTeamId: ref(1), active: ref(true), deps: {
      summary: async id => summary(id), vote: async (code, value) => { votes.push([code, value]); return { up: 2, down: 0, my_vote: 1 } }, notify: m => notices.push(m),
    } })
    await flush()
    await fan.vote(fan.summary.value!.boards[0].items[0], 1)
    expect(votes).toEqual([['ABCD1234', 1]])
    expect(fan.summary.value!.boards[0].items[0].up).toBe(2)
    fan.dispose()
    const rival = useInsightsTransfer({ session: session(), selectedTeamId: ref(1), active: ref(true), deps: {
      summary: async id => summary(id, false), vote: async () => { throw new Error('should not be called') }, notify: m => notices.push(m),
    } })
    await flush()
    expect(rival.hint.value).toBe('只能为你的主队投票')
    await rival.vote(rival.summary.value!.boards[0].items[0], 1)
    expect(notices).toEqual(['只能为你的主队投票'])
    rival.dispose()
  })

  test('guests see a login hint and voting asks to log in', async () => {
    const logins: string[] = []
    const guest = useInsightsTransfer({ session: session(false), selectedTeamId: ref(1), active: ref(true), deps: {
      summary: async id => summary(id, false), login: url => logins.push(url),
    } })
    await flush()
    expect(guest.hint.value).toBe('登录并设置主队后可以投票')
    await guest.vote(guest.summary.value!.boards[0].items[0], -1)
    expect(logins).toEqual(['/pages/insights/index'])
    guest.dispose()
  })
})
```

- [ ] **Step 3: 运行，确认失败**

Run: `bun test src/pages/insights/useInsightsTransfer.test.ts 2>&1 | tail -3`
Expected: 失败，模块不存在。

- [ ] **Step 4: 实现 `useInsightsTransfer`**

`src/pages/insights/useInsightsTransfer.ts`：

```ts
import { computed, ref, watch, type Ref } from 'vue'
import { getTransferSummary, setTransferVote } from '../../api/transfer'
import { createTransferVotes, transferVoteDeps } from '../../composables/useTransferVotes'
import type { AppSession } from '../../stores/appSession'
import type { TransferCard, TransferSummary, TransferTally, TransferVoteValue } from '../../types/transfer'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { applyTally } from '../../utils/transferLinks'

const RETURN_URL = '/pages/insights/index'

export interface InsightsTransferOptions {
  session: Pick<AppSession, 'currentUser' | 'hasAccessToken' | 'status' | 'revision' | 'invalidateSession'>
  selectedTeamId: Readonly<Ref<number | null>>
  active: Readonly<Ref<boolean>>
  deps?: {
    summary?: typeof getTransferSummary
    vote?: (code: string, value: 1 | -1 | 0) => Promise<TransferTally>
    login?: (returnUrl: string) => void
    notify?: (message: string) => void
  }
}

/** 洞察页"转会"胶囊：只在激活时加载，跟随页面顶部选中的球队。 */
export function useInsightsTransfer(options: InsightsTransferOptions) {
  const { session, selectedTeamId, active } = options
  const read = options.deps?.summary ?? getTransferSummary
  const notify = options.deps?.notify ?? ((title: string) => uni.showToast({ title, icon: 'none' }))
  const summary = ref<TransferSummary | null>(null)
  const loading = ref(false), error = ref('')
  const loggedIn = () => session.hasAccessToken.value && session.status.value !== 'loading'
  const votes = createTransferVotes(transferVoteDeps({
    vote: options.deps?.vote ?? setTransferVote,
    loggedIn,
    revision: () => session.revision.value,
    invalidate: revision => session.invalidateSession(revision),
    ...(options.deps?.login ? { login: options.deps.login } : {}),
  }))
  let generation = 0, disposed = false

  const teamId = () => {
    const id = selectedTeamId.value
    return id !== null && Number.isInteger(id) && id > 0 ? id : 0
  }
  const identity = () => `${session.revision.value}:${session.currentUser.value ? 'user' : 'guest'}`
  const enabled = () => !disposed && active.value && Boolean(teamId())

  const hint = computed(() => {
    const value = summary.value
    if (!value || !value.window?.open || value.can_vote) return ''
    return loggedIn() ? '只能为你的主队投票' : '登录并设置主队后可以投票'
  })

  async function refresh(): Promise<void> {
    if (!enabled()) return
    const owner = ++generation, id = teamId(), revision = session.revision.value
    const current = () => enabled() && owner === generation && teamId() === id
    votes.reset()
    loading.value = true; error.value = ''
    if (summary.value?.team.team_id !== id) summary.value = null
    try {
      const result = await read(id)
      if (current()) summary.value = result
    } catch (cause) {
      if (!current()) return
      if (isUnauthorizedError(cause)) session.invalidateSession(revision)
      error.value = extractApiErrorMessage(cause, '转会投票加载失败，请重试')
    } finally {
      if (current()) loading.value = false
    }
  }

  async function vote(card: TransferCard, value: TransferVoteValue): Promise<void> {
    const value_ = summary.value
    if (!value_ || !enabled()) return
    if (loggedIn() && !value_.can_vote) {
      notify(value_.window?.open ? '只能为你的主队投票' : '本期投票已结束')
      return
    }
    const id = teamId()
    const message = await votes.toggle(card, value, {
      returnUrl: RETURN_URL,
      current: () => enabled() && teamId() === id,
      publish: tally => {
        const target = summary.value
        if (!target) return
        summary.value = { ...target, boards: target.boards.map(board => ({
          ...board, items: board.items.map(item => item.share_code === card.share_code ? applyTally(item, tally) : item),
        })) }
      },
    })
    if (message) notify(message)
  }

  const stop = watch(() => [teamId(), active.value, identity()] as const, () => {
    if (!enabled()) { generation++; loading.value = false; votes.reset(); return }
    void refresh()
  }, { immediate: true })

  function dispose(): void { disposed = true; generation++; stop(); votes.reset() }
  return { summary, loading, error, hint, pending: votes.pending, refresh, vote, dispose }
}
```

- [ ] **Step 5: 运行，确认通过**

Run: `bun test src/pages/insights/useInsightsTransfer.test.ts 2>&1 | tail -3`
Expected: `4 pass`

- [ ] **Step 6: 写 `InsightsTransferPanel`**

`src/pages/insights/components/InsightsTransferPanel.vue`：

```vue
<template>
  <view class="transfer-panel">
    <view class="transfer-panel__header">
      <text class="transfer-panel__title">转会投票</text>
      <text v-if="summary?.window" class="transfer-panel__status">{{ summary.window.name }} · {{ summary.window.open ? '投票中' : '已结束' }}</text>
    </view>
    <text v-if="hint" class="transfer-panel__hint">{{ hint }}</text>
    <view v-if="loading && !summary" class="transfer-panel__skeletons">
      <view v-for="index in 3" :key="index" class="fi-skeleton transfer-panel__skeleton" />
    </view>
    <view v-else-if="error && !summary" class="transfer-panel__state">
      <text>{{ error }}</text>
      <button class="transfer-panel__action" @tap="emit('retry')">重试</button>
    </view>
    <text v-else-if="summary && !summary.window" class="transfer-panel__state">暂无转会投票</text>
    <template v-else-if="summary">
      <view v-for="board in summary.boards" :key="board.intent" class="transfer-panel__board">
        <view class="transfer-panel__board-head" hover-class="transfer-panel__board-head--pressed" @tap="emit('more', board.intent)">
          <text class="transfer-panel__board-title">{{ TRANSFER_BOARD_LABELS[board.intent] }}<text class="transfer-panel__count">{{ board.total }}</text></text>
          <text class="transfer-panel__more">查看全部 ›</text>
        </view>
        <text v-if="!board.items.length" class="transfer-panel__empty">还没有人提名，来做第一个</text>
        <view v-for="card in board.items" :key="card.share_code" class="transfer-panel__item">
          <FiTransferCard :card="card" :pending="pending.includes(card.share_code)" @open="emit('open', card.share_code)" @vote="emit('vote', card, $event)" />
        </view>
      </view>
      <button v-if="summary.window?.open" class="transfer-panel__action transfer-panel__action--primary" @tap="emit('nominate')">提名球员</button>
    </template>
  </view>
</template>
<script setup lang="ts">
import FiTransferCard from '../../../components/FiTransferCard.vue'
import type { TransferCard, TransferIntent, TransferSummary } from '../../../types/transfer'
import { TRANSFER_BOARD_LABELS } from '../../../utils/transferLinks'
defineProps<{ summary: TransferSummary | null; loading: boolean; error: string; hint: string; pending: string[] }>()
const emit = defineEmits<{
  (e: 'retry' | 'nominate'): void
  (e: 'vote', card: TransferCard, value: 1 | -1): void
  (e: 'open', code: string): void
  (e: 'more', intent: TransferIntent): void
}>()
</script>
<style scoped>
@import '../../../styles/fi-skeleton.css';
.transfer-panel { display: flex; flex-direction: column; gap: var(--fi-space-20); }
.transfer-panel__header { display: flex; align-items: baseline; justify-content: space-between; gap: var(--fi-space-12); }
.transfer-panel__title { color: var(--fi-color-text-primary); font-size: var(--fi-font-28); font-weight: var(--fi-weight-extrabold); }
.transfer-panel__status { color: var(--fi-color-text-caption); font-size: var(--fi-font-22); }
.transfer-panel__hint { padding: var(--fi-space-12) var(--fi-space-16); border-radius: var(--fi-radius-md); background: var(--fi-color-page-soft); color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); }
.transfer-panel__board { padding-top: var(--fi-space-16); border-top: var(--fi-border-width) solid var(--fi-color-divider); }
.transfer-panel__board-head { display: flex; align-items: center; justify-content: space-between; }
.transfer-panel__board-head--pressed { opacity: var(--fi-opacity-pressed); }
.transfer-panel__board-title { color: var(--fi-color-text-strong); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.transfer-panel__count { margin-left: var(--fi-space-8); color: var(--fi-color-text-caption); font-size: var(--fi-font-22); font-weight: normal; }
.transfer-panel__more { color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); }
.transfer-panel__empty, .transfer-panel__state { padding: var(--fi-space-20) 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-24); text-align: center; }
.transfer-panel__item + .transfer-panel__item { border-top: var(--fi-border-width) solid var(--fi-color-divider-soft); }
.transfer-panel__action { margin: var(--fi-space-12) 0 0; padding: var(--fi-space-16); border-radius: var(--fi-radius-round); background: var(--fi-color-surface-panel); color: var(--fi-color-primary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.transfer-panel__action::after { border: 0; }
.transfer-panel__action--primary { background: var(--fi-color-primary); color: var(--fi-color-surface); }
.transfer-panel__skeletons { display: flex; flex-direction: column; gap: var(--fi-space-12); }
.transfer-panel__skeleton { height: calc(var(--fi-space-64) * 1.5); border-radius: var(--fi-radius-md); }
</style>
```

- [ ] **Step 7: 接入洞察页**

`src/pages/insights/index.vue`：

1. 模板中 `<InsightsTacticsPanel v-if="mainTab === 'tactics'" … />` 之后、`<template v-else>` 之前插入：

```vue
              <InsightsTransferPanel
                v-else-if="mainTab === 'transfer'"
                :summary="transferSummary" :loading="transferLoading || teamLoading" :error="teamError || transferError"
                :hint="transferHint" :pending="transferPending"
                @retry="retryTransfer" @vote="transfer.vote" @open="openTransferCard" @more="openTransferBoard" @nominate="openTransferNominate"
              />
```

2. script：

```ts
import InsightsTransferPanel from './components/InsightsTransferPanel.vue'
import { useInsightsTransfer } from './useInsightsTransfer'
import { buildTransferBoardUrl, buildTransferViewUrl, transferBoardShareTitle } from '../../utils/transferLinks'
import type { TransferIntent } from '../../types/transfer'
```

把 `mainTab` 相关三行替换为：

```ts
const mainTab = ref<'data' | 'tactics' | 'transfer'>('data')
const mainTabs = [{ key: 'data', label: '数据洞察' }, { key: 'tactics', label: '战术' }, { key: 'transfer', label: '转会' }]
function changeMainTab(value: string): void {
  if (value === 'data' || value === 'tactics' || value === 'transfer') mainTab.value = value
}
```

在 `const tactics = …` 之后加：

```ts
const transfer = useInsightsTransfer({ session, selectedTeamId, active: computed(() => pageVisible.value && mainTab.value === 'transfer') })
const { summary: transferSummary, loading: transferLoading, error: transferError, hint: transferHint, pending: transferPending } = transfer
function retryTransfer(): void {
  if (teamError.value) void teamBrowser.load()
  else void transfer.refresh()
}
function openTransferCard(code: string): void {
  const url = buildTransferViewUrl(code)
  if (url) uni.navigateTo({ url })
}
function openTransferBoard(intent: TransferIntent): void {
  if (selectedTeamId.value !== null) uni.navigateTo({ url: buildTransferBoardUrl(selectedTeamId.value, intent) })
}
function openTransferNominate(): void {
  if (selectedTeamId.value !== null) uni.navigateTo({ url: buildTransferBoardUrl(selectedTeamId.value, 'out', { nominate: true }) })
}
```

3. `onUnload` 里加 `transfer.dispose();`；`onShow` 中 `void tactics.refresh()` 之后加 `void transfer.refresh()`。

4. 分享：把 `onShareAppMessage` 的 `title` 改为先判断转会胶囊：

```ts
onShareAppMessage(() => {
  const teamName = selectableTeams.value.find(team => team.team_id === selectedTeamId.value)?.team_name ?? '球队'
  const windowName = transferSummary.value?.window?.name
  if (mainTab.value === 'transfer' && windowName && selectedTeamId.value !== null) {
    return { title: transferBoardShareTitle(teamName, windowName, 'out'), path: buildSharePath(buildTransferBoardUrl(selectedTeamId.value, 'out'), 'transfer_board') }
  }
  return {
    title: mainTab.value === 'tactics' ? `${teamName}战术，看看大家怎么排` : selectedInsight.value
      ? `${selectedInsight.value.team_name} 的贡献结构，看看这队靠谁赢球`
      : '球队洞察：不是榜单，而是贡献结构',
    path: buildSharePath('/pages/insights/index', 'insights'),
  }
})
```

- [ ] **Step 8: 运行洞察页相关测试与检查**

Run: `bun test src/pages/insights 2>&1 | grep -E "^ *[0-9]+ (pass|fail)" && bun run type-check 2>&1 | tail -1 && bun run check:boundaries | tail -1`
Expected: 全部 pass；类型通过；边界通过。

- [ ] **Step 9: 提交**

```bash
git add src/components/FiTransferCard.vue src/pages/insights
git commit -m "feat(mini): add the transfer tab to insights

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: 分包页面（完整榜单、单卡、提名弹层）

**Files:**
- Create: `football_insight_mini/src/pages/transfer/helpers.ts`
- Create: `football_insight_mini/src/pages/transfer/helpers.test.ts`
- Create: `football_insight_mini/src/pages/transfer/useTransferBoard.ts`
- Create: `football_insight_mini/src/pages/transfer/useTransferBoard.test.ts`
- Create: `football_insight_mini/src/pages/transfer/useTransferNominate.ts`
- Create: `football_insight_mini/src/pages/transfer/useTransferNominate.test.ts`
- Create: `football_insight_mini/src/pages/transfer/components/TransferNominateSheet.vue`
- Create: `football_insight_mini/src/pages/transfer/transfer.css`
- Create: `football_insight_mini/src/pages/transfer/board.vue`
- Create: `football_insight_mini/src/pages/transfer/view.vue`
- Create: `football_insight_mini/scripts/transfer-style-audit.test.mjs`
- Modify: `football_insight_mini/src/pages.json`

**Interfaces:**
- Consumes: Task 6、Task 7 的类型、API、链接、`createTransferVotes`、`transferVoteDeps`、`FiTransferCard`。
- Produces:
  - `filterCandidateGroups(groups, keyword)`、`NOMINATE_INTENT_ITEMS`
  - `createTransferBoard(deps?)` → `{ teamId, intent, board, items, loading, error, pending, load(teamId, intent), more(), changeIntent(intent), vote(card, value), dispose() }`
  - `createTransferNominate(deps?)` → `{ visible, intent, groups, loading, error, submitting, open(teamId, intent), changeIntent(intent), retry(), submit(playerId): Promise<TransferNominateResult | null> }`

- [ ] **Step 1: 写 helpers 的失败测试**

`src/pages/transfer/helpers.test.ts`：

```ts
import { expect, test } from 'bun:test'
import type { TransferCandidateGroup } from '../../types/transfer'
import { NOMINATE_INTENT_ITEMS, filterCandidateGroups } from './helpers'

const player = (id: number, name: string) => ({ player_id: id, name, shirt_number: null, position: null, avatar_url: null, current_team: null })
const groups: TransferCandidateGroup[] = [
  { key: 'former', title: '本队旧将', players: [player(1, '张三'), player(2, '李四')] },
  { key: 'team:2', title: '上海海港', players: [player(3, '张飞')] },
]

test('keyword filters players and drops empty groups; blank keeps everything', () => {
  expect(filterCandidateGroups(groups, '  ')).toEqual(groups)
  expect(filterCandidateGroups(groups, '张').map(g => [g.key, g.players.map(p => p.player_id)])).toEqual([['former', [1]], ['team:2', [3]]])
  expect(filterCandidateGroups(groups, '李').map(g => g.key)).toEqual(['former'])
  expect(filterCandidateGroups(groups, '王')).toEqual([])
})

test('nominate intents are labelled as stances', () => {
  expect(NOMINATE_INTENT_ITEMS).toEqual([
    { key: 'out', label: '希望离队' }, { key: 'stay', label: '希望留下' }, { key: 'in', label: '希望买入' },
  ])
})
```

- [ ] **Step 2: 运行确认失败，再实现 helpers**

Run: `bun test src/pages/transfer/helpers.test.ts 2>&1 | tail -3` → Expected: 失败。

`src/pages/transfer/helpers.ts`：

```ts
import type { TransferCandidateGroup, TransferIntent } from '../../types/transfer'

export const NOMINATE_INTENT_ITEMS: Array<{ key: TransferIntent; label: string }> = [
  { key: 'out', label: '希望离队' }, { key: 'stay', label: '希望留下' }, { key: 'in', label: '希望买入' },
]

/** 本地按名字过滤候选（关键字不提交后端）；过滤后为空的分组不显示。 */
export function filterCandidateGroups(groups: TransferCandidateGroup[], keyword: string): TransferCandidateGroup[] {
  const word = keyword.trim()
  if (!word) return groups
  return groups
    .map(group => ({ ...group, players: group.players.filter(player => player.name.includes(word)) }))
    .filter(group => group.players.length > 0)
}
```

Run: `bun test src/pages/transfer/helpers.test.ts 2>&1 | tail -3` → Expected: `2 pass`

- [ ] **Step 3: 写 `useTransferBoard` 的失败测试**

`src/pages/transfer/useTransferBoard.test.ts`：

```ts
import { describe, expect, test } from 'bun:test'
import type { TransferBoard, TransferIntent } from '../../types/transfer'
import { createTransferBoard } from './useTransferBoard'

function page(intent: TransferIntent, pageNo: number, total = 25): TransferBoard {
  const start = (pageNo - 1) * 20
  return {
    team: { team_id: 1, team_name: 'T1', avatar_url: null },
    window: { code: 'w', name: '2026 冬窗', opens_at: '', closes_at: '', open: true },
    intent, page: pageNo, page_size: 20, total, can_vote: true,
    items: Array.from({ length: Math.max(0, Math.min(20, total - start)) }, (_, i) => ({
      share_code: `C${String(start + i).padStart(7, '0')}`, intent, returning: false, up: 1, down: 0, my_vote: 0 as const, rank: start + i + 1, created_at: '',
      player: { player_id: start + i, name: 'X', avatar_url: null, position: null, shirt_number: null, current_team: null, departed: false },
    })),
  }
}
const base = { revision: () => 0, loggedIn: () => true, invalidate: () => {}, login: () => {}, notify: () => {} }

describe('transfer board page', () => {
  test('paginates without duplicates and stops at total', async () => {
    const calls: Array<[TransferIntent, number]> = []
    const state = createTransferBoard({ ...base, board: async (_, intent, p) => { calls.push([intent, p]); return page(intent, p) } })
    await state.load(1, 'in')
    await state.more(); await state.more()
    expect(calls).toEqual([['in', 1], ['in', 2]])
    expect(state.items.value.length).toBe(25)
  })

  test('switching intent drops the old list and late responses', async () => {
    let releaseOut: (b: TransferBoard) => void = () => {}
    const state = createTransferBoard({ ...base, board: (_, intent, p) => intent === 'out' ? new Promise(r => { releaseOut = r }) : Promise.resolve(page(intent, p, 3)) })
    const first = state.load(1, 'out')
    await state.changeIntent('stay')
    releaseOut(page('out', 1)); await first
    expect(state.intent.value).toBe('stay')
    expect(state.items.value.every(item => item.intent === 'stay')).toBe(true)
  })

  test('votes patch in place; closed windows notify instead of writing', async () => {
    const notices: string[] = [], votes: string[] = []
    const state = createTransferBoard({ ...base, notify: m => notices.push(m),
      board: async (_, intent, p) => ({ ...page(intent, p, 2), can_vote: false, window: { code: 'w', name: 'w', opens_at: '', closes_at: '', open: false } }),
      vote: async code => { votes.push(code); return { up: 9, down: 0, my_vote: 1 } } })
    await state.load(1, 'out')
    await state.vote(state.items.value[0], 1)
    expect(votes).toEqual([]); expect(notices).toEqual(['本期投票已结束'])
    const open = createTransferBoard({ ...base, board: async (_, intent, p) => page(intent, p, 2), vote: async () => ({ up: 9, down: 0, my_vote: 1 }) })
    await open.load(1, 'out')
    await open.vote(open.items.value[1], 1)
    expect(open.items.value[1].up).toBe(9)
  })
})
```

- [ ] **Step 4: 运行确认失败，再实现 `useTransferBoard`**

Run: `bun test src/pages/transfer/useTransferBoard.test.ts 2>&1 | tail -3` → Expected: 失败。

`src/pages/transfer/useTransferBoard.ts`：

```ts
import { computed, ref } from 'vue'
import { getTransferBoard, setTransferVote } from '../../api/transfer'
import { createTransferVotes, transferVoteDeps, type TransferVoteDeps } from '../../composables/useTransferVotes'
import type { TransferBoard, TransferCard, TransferIntent, TransferVoteValue } from '../../types/transfer'
import { extractApiErrorMessage } from '../../utils/apiError'
import { applyTally, buildTransferBoardUrl } from '../../utils/transferLinks'

export interface TransferBoardDeps extends Partial<TransferVoteDeps> {
  board?: typeof getTransferBoard
  notify?: (message: string) => void
}

export function createTransferBoard(deps: TransferBoardDeps = {}) {
  const read = deps.board ?? getTransferBoard
  const notify = deps.notify ?? ((title: string) => uni.showToast({ title, icon: 'none' }))
  const { board: _board, notify: _notify, ...voteOverrides } = deps
  const voteDeps = transferVoteDeps({ vote: setTransferVote, ...voteOverrides })
  const votes = createTransferVotes(voteDeps)
  const teamId = ref(0), intent = ref<TransferIntent>('out')
  const board = ref<TransferBoard | null>(null), items = ref<TransferCard[]>([])
  const loading = ref(false), error = ref('')
  let generation = 0, disposed = false
  const hasMore = computed(() => !board.value || items.value.length < board.value.total)

  async function fetch(pageNo: number): Promise<void> {
    if (disposed || !teamId.value) return
    const owner = ++generation, id = teamId.value, which = intent.value
    loading.value = true; error.value = ''
    try {
      const result = await read(id, which, pageNo)
      if (owner !== generation || disposed) return
      board.value = result
      const known = new Set(pageNo === 1 ? [] : items.value.map(item => item.share_code))
      items.value = [...(pageNo === 1 ? [] : items.value), ...result.items.filter(item => !known.has(item.share_code))]
    } catch (cause) {
      if (owner === generation && !disposed) error.value = extractApiErrorMessage(cause, '转会榜加载失败，请重试')
    } finally {
      if (owner === generation) loading.value = false
    }
  }

  function load(id: number, which: TransferIntent): Promise<void> {
    teamId.value = id; intent.value = which
    board.value = null; items.value = []; votes.reset()
    return fetch(1)
  }
  function more(): Promise<void> {
    if (loading.value || !hasMore.value || !board.value) return Promise.resolve()
    return fetch(board.value.page + 1)
  }
  function changeIntent(which: TransferIntent): Promise<void> {
    if (which === intent.value && board.value && !error.value) return Promise.resolve()
    return load(teamId.value, which)
  }
  async function vote(card: TransferCard, value: TransferVoteValue): Promise<void> {
    const current = board.value
    if (!current) return
    // 已登录但不能投（非主队或已截止）给提示；游客交给 toggle 去登录。
    if (voteDeps.loggedIn() && !current.can_vote) {
      notify(current.window?.open ? '只能为你的主队投票' : '本期投票已结束')
      return
    }
    const owner = generation
    const message = await votes.toggle(card, value, {
      returnUrl: buildTransferBoardUrl(teamId.value, intent.value),
      current: () => owner === generation && !disposed,
      publish: tally => { items.value = items.value.map(item => item.share_code === card.share_code ? applyTally(item, tally) : item) },
    })
    if (message) notify(message)
  }
  function dispose(): void { disposed = true; generation++; votes.reset() }
  return { teamId, intent, board, items, loading, error, hasMore, pending: votes.pending, load, more, changeIntent, vote, dispose }
}
```

Run: `bun test src/pages/transfer/useTransferBoard.test.ts 2>&1 | tail -3` → Expected: `3 pass`

- [ ] **Step 5: 写 `useTransferNominate` 的失败测试**

`src/pages/transfer/useTransferNominate.test.ts`：

```ts
import { describe, expect, test } from 'bun:test'
import type { TransferCandidates, TransferIntent, TransferNominateResult } from '../../types/transfer'
import { ApiRequestError } from '../../utils/apiError'
import { createTransferNominate } from './useTransferNominate'

const candidates = (intent: TransferIntent): TransferCandidates => ({ intent, groups: [{ key: 'squad', title: 'T', players: [] }] })
const result = (created: boolean) => ({ created, view: { card: { share_code: 'ABCD1234' } } }) as unknown as TransferNominateResult

describe('transfer nominate', () => {
  test('candidates are cached per intent and the sheet reopens on the chosen intent', async () => {
    const calls: TransferIntent[] = []
    const state = createTransferNominate({ candidates: async (_, intent) => { calls.push(intent); return candidates(intent) }, loggedIn: () => true })
    await state.open(1, 'in'); await state.changeIntent('out'); await state.changeIntent('in')
    expect(calls).toEqual(['in', 'out'])
    expect(state.visible.value).toBe(true)
    expect(state.intent.value).toBe('in')
  })

  test('submit reports created vs merged and server messages', async () => {
    const notices: string[] = []
    let mode: 'new' | 'merge' | 'limit' = 'new'
    const state = createTransferNominate({
      candidates: async (_, intent) => candidates(intent), loggedIn: () => true, notify: m => notices.push(m),
      nominate: async () => {
        if (mode === 'limit') throw new ApiRequestError('本期这一类最多提名 3 位', 429)
        return result(mode === 'new')
      },
    })
    await state.open(1, 'out')
    expect((await state.submit(9))?.created).toBe(true)
    mode = 'merge'; expect((await state.submit(9))?.created).toBe(false)
    mode = 'limit'; expect(await state.submit(9)).toBe(null)
    expect(notices).toEqual(['提名成功', '已有球迷提名，已为你加一票', '本期这一类最多提名 3 位'])
    // 成功后关闭弹层；之后的失败不会重新打开它
    expect(state.visible.value).toBe(false)
  })

  test('guests are sent to log in before the sheet opens', async () => {
    const logins: string[] = []
    const state = createTransferNominate({ candidates: async (_, intent) => candidates(intent), loggedIn: () => false, login: url => logins.push(url) })
    await state.open(1, 'stay')
    expect(state.visible.value).toBe(false)
    expect(logins).toEqual(['/pages/transfer/board?teamId=1&intent=stay&nominate=1'])
  })
})
```

- [ ] **Step 6: 运行确认失败，再实现 `useTransferNominate`**

Run: `bun test src/pages/transfer/useTransferNominate.test.ts 2>&1 | tail -3` → Expected: 失败。

`src/pages/transfer/useTransferNominate.ts`：

```ts
import { ref } from 'vue'
import { getTransferCandidates, nominateTransfer } from '../../api/transfer'
import { transferVoteDeps } from '../../composables/useTransferVotes'
import type { TransferCandidateGroup, TransferIntent, TransferNominateResult } from '../../types/transfer'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { buildTransferBoardUrl } from '../../utils/transferLinks'

export interface TransferNominateDeps {
  candidates?: typeof getTransferCandidates
  nominate?: typeof nominateTransfer
  loggedIn?: () => boolean
  login?: (returnUrl: string) => void
  notify?: (message: string) => void
}

export function createTransferNominate(deps: TransferNominateDeps = {}) {
  const session = transferVoteDeps({
    ...(deps.loggedIn ? { loggedIn: deps.loggedIn, revision: () => 0, invalidate: () => {} } : {}),
    ...(deps.login ? { login: deps.login } : {}),
  })
  const readCandidates = deps.candidates ?? getTransferCandidates
  const write = deps.nominate ?? nominateTransfer
  const notify = deps.notify ?? ((title: string) => uni.showToast({ title, icon: 'none' }))
  const visible = ref(false), intent = ref<TransferIntent>('out')
  const groups = ref<TransferCandidateGroup[]>([]), loading = ref(false), error = ref(''), submitting = ref(false)
  const cache = new Map<string, TransferCandidateGroup[]>()
  let teamId = 0, generation = 0

  async function loadGroups(): Promise<void> {
    const key = `${teamId}:${intent.value}`
    const cached = cache.get(key)
    if (cached) { groups.value = cached; error.value = ''; return }
    const owner = ++generation
    loading.value = true; error.value = ''; groups.value = []
    try {
      const result = await readCandidates(teamId, intent.value)
      cache.set(key, result.groups)
      if (owner === generation) groups.value = result.groups
    } catch (cause) {
      if (owner === generation) error.value = extractApiErrorMessage(cause, '候选球员加载失败，请重试')
    } finally {
      if (owner === generation) loading.value = false
    }
  }

  async function open(id: number, which: TransferIntent): Promise<void> {
    teamId = id; intent.value = which
    if (!session.loggedIn()) { session.login(buildTransferBoardUrl(id, which, { nominate: true })); return }
    visible.value = true
    await loadGroups()
  }
  function changeIntent(which: TransferIntent): Promise<void> {
    intent.value = which
    return loadGroups()
  }
  function retry(): Promise<void> { cache.delete(`${teamId}:${intent.value}`); return loadGroups() }

  async function submit(playerId: number): Promise<TransferNominateResult | null> {
    if (submitting.value) return null
    submitting.value = true
    try {
      const result = await write(teamId, playerId, intent.value)
      notify(result.created ? '提名成功' : '已有球迷提名，已为你加一票')
      visible.value = false
      return result
    } catch (cause) {
      if (isUnauthorizedError(cause)) { visible.value = false; session.login(buildTransferBoardUrl(teamId, intent.value, { nominate: true })); return null }
      notify(extractApiErrorMessage(cause, '提名失败，请重试'))
      return null
    } finally {
      submitting.value = false
    }
  }
  return { visible, intent, groups, loading, error, submitting, open, changeIntent, retry, submit }
}
```

Run: `bun test src/pages/transfer 2>&1 | grep -E "^ *[0-9]+ (pass|fail)"` → Expected: 全部 pass。

- [ ] **Step 7: 写提名弹层组件**

`src/pages/transfer/components/TransferNominateSheet.vue`：

```vue
<template>
  <FiBottomSheet :visible="visible" eyebrow="提名球员" title="你希望谁…" height="80vh" @update:visible="emit('update:visible', $event)">
    <view class="nominate__intents">
      <FiSegmented :items="NOMINATE_INTENT_ITEMS" :model-value="intent" aria-label="提名意向" @update:model-value="emit('update:intent', $event as TransferIntent)" />
    </view>
    <!-- 搜索只在本地过滤候选，不提交到后端。 -->
    <input v-model="keyword" class="nominate__search" placeholder="按名字查找" confirm-type="search" />
    <view v-if="loading" class="nominate__state"><text>候选球员加载中…</text></view>
    <view v-else-if="error" class="nominate__state">
      <text>{{ error }}</text>
      <button class="nominate__retry" @tap="emit('retry')">重试</button>
    </view>
    <view v-else-if="!visibleGroups.length" class="nominate__state"><text>没有符合的球员</text></view>
    <view v-for="group in visibleGroups" :key="group.key" class="nominate__group">
      <text class="nominate__group-title">{{ group.title }}</text>
      <view v-for="player in group.players" :key="player.player_id" class="nominate__row" hover-class="nominate__row--pressed" @tap="!submitting && emit('pick', player.player_id)">
        <image v-if="player.avatar_url" class="nominate__avatar" :src="player.avatar_url" mode="aspectFill" />
        <view v-else class="nominate__avatar nominate__avatar--fallback"><text>{{ player.name.slice(0, 1) }}</text></view>
        <view class="nominate__body">
          <text class="nominate__name">{{ player.name }}</text>
          <text class="nominate__meta">{{ [positionLabel(player.position), player.current_team?.team_name ?? ''].filter(Boolean).join(' · ') }}</text>
        </view>
      </view>
    </view>
  </FiBottomSheet>
</template>
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import FiSegmented from '../../../components/FiSegmented.vue'
import type { TransferCandidateGroup, TransferIntent } from '../../../types/transfer'
import { positionLabel } from '../../../utils/transferLinks'
import { NOMINATE_INTENT_ITEMS, filterCandidateGroups } from '../helpers'
const props = defineProps<{ visible: boolean; intent: TransferIntent; groups: TransferCandidateGroup[]; loading: boolean; error: string; submitting: boolean }>()
const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'update:intent', value: TransferIntent): void
  (e: 'pick', playerId: number): void
  (e: 'retry'): void
}>()
const keyword = ref('')
const visibleGroups = computed(() => filterCandidateGroups(props.groups, keyword.value))
watch(() => props.visible, open => { if (open) keyword.value = '' })
</script>
<style scoped>
.nominate__intents { margin-bottom: var(--fi-space-16); }
.nominate__search { box-sizing: border-box; width: 100%; height: var(--fi-space-72); margin-bottom: var(--fi-space-16); padding: 0 var(--fi-space-24); border: var(--fi-border-card); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); color: var(--fi-color-text-strong); font-size: var(--fi-font-26); }
.nominate__state { padding: var(--fi-space-32) 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-24); text-align: center; }
.nominate__retry { margin: var(--fi-space-12) auto 0; padding: var(--fi-space-12) var(--fi-space-32); border-radius: var(--fi-radius-round); background: var(--fi-color-surface-panel); color: var(--fi-color-primary); font-size: var(--fi-font-24); }
.nominate__retry::after { border: 0; }
.nominate__group { margin-bottom: var(--fi-space-16); }
.nominate__group-title { display: block; padding: var(--fi-space-12) var(--fi-space-4); color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); }
.nominate__row { display: flex; align-items: center; gap: var(--fi-space-18); padding: var(--fi-space-14) var(--fi-space-4); border-top: var(--fi-border-width) solid var(--fi-color-divider-soft); }
.nominate__row--pressed { opacity: var(--fi-opacity-pressed); }
.nominate__avatar { flex-shrink: 0; width: var(--fi-space-64); height: var(--fi-space-64); border-radius: var(--fi-radius-round); background: var(--fi-color-page); }
.nominate__avatar--fallback { display: flex; align-items: center; justify-content: center; color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.nominate__body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: var(--fi-space-4); }
.nominate__name { color: var(--fi-color-text-strong); font-size: var(--fi-font-28); font-weight: var(--fi-weight-bold); }
.nominate__meta { color: var(--fi-color-text-muted); font-size: var(--fi-font-22); }
</style>
```

- [ ] **Step 8: 写分包样式与两个页面**

`src/pages/transfer/transfer.css`：

```css
.transfer-page { min-height: 100vh; box-sizing: border-box; padding: calc(var(--fi-brand-nav-height) + var(--fi-space-24)) var(--fi-space-24) calc(var(--fi-space-64) * 2); background: var(--fi-color-page-soft); display: flex; flex-direction: column; gap: var(--fi-space-24); }
.transfer-title { color: var(--fi-color-text-strong); font-size: var(--fi-font-32); font-weight: var(--fi-weight-bold); }
.transfer-caption { color: var(--fi-color-text-muted); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.transfer-error { color: var(--fi-color-primary-deep); font-size: var(--fi-font-24); }
.transfer-panel { padding: var(--fi-space-8) var(--fi-space-24); border-radius: var(--fi-radius-md); background: var(--fi-color-surface); }
.transfer-row { display: flex; align-items: center; gap: var(--fi-space-16); }
.transfer-action { margin: 0; padding: var(--fi-space-20); border-radius: var(--fi-radius-round); background: var(--fi-color-surface); color: var(--fi-color-text-primary); font-size: var(--fi-font-26); line-height: var(--fi-leading-normal); border: var(--fi-border-card); }
.transfer-row > .transfer-action { flex: 1; }
.transfer-action::after { border: 0; }
.transfer-action--active { color: var(--fi-color-surface); background: var(--fi-color-primary); }
.transfer-action[disabled] { opacity: var(--fi-opacity-pressed); }
.transfer-item + .transfer-item { border-top: var(--fi-border-width) solid var(--fi-color-divider-soft); }
.transfer-dock { position: fixed; left: var(--fi-space-24); right: var(--fi-space-24); bottom: calc(var(--fi-space-24) + env(safe-area-inset-bottom)); }
```

`src/pages/transfer/board.vue`：

```vue
<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBack" />
  <view class="transfer-page">
    <text class="transfer-title">{{ board?.team.team_name ?? '' }}转会榜</text>
    <text v-if="!teamId" class="transfer-caption">先在首页选择你的主队，再来给转会表态。</text>
    <template v-else>
      <text v-if="board?.window" class="transfer-caption">{{ board.window.name }} · {{ board.window.open ? '投票中' : '本期投票已结束' }} · {{ board.total }} 张提名</text>
      <FiSegmented :items="intentItems" :model-value="intent" aria-label="转会榜" @update:model-value="changeIntent($event as TransferIntent)" />
      <text v-if="error" class="transfer-error">{{ error }}</text>
      <FiLoading v-if="loading && !items.length" title="转会榜加载中" caption="正在读取球迷的提名。" />
      <view v-if="items.length" class="transfer-panel">
        <view v-for="card in items" :key="card.share_code" class="transfer-item">
          <FiTransferCard :card="card" :pending="pending.includes(card.share_code)" @open="openCard(card.share_code)" @vote="vote(card, $event)" />
        </view>
      </view>
      <text v-if="!loading && !error && !items.length" class="transfer-caption">还没有人提名，来做第一个。</text>
      <button v-if="error || hasMore" class="transfer-action" :disabled="loading" @tap="retry">{{ loading ? '加载中…' : error ? '重试加载' : '加载更多' }}</button>
      <view v-if="board?.window?.open" class="transfer-dock">
        <button class="transfer-action transfer-action--active" @tap="openNominate">提名球员</button>
      </view>
    </template>
    <TransferNominateSheet v-model:visible="nominate.visible.value" :intent="nominate.intent.value" :groups="nominate.groups.value"
      :loading="nominate.loading.value" :error="nominate.error.value" :submitting="nominate.submitting.value"
      @update:intent="nominate.changeIntent" @retry="nominate.retry()" @pick="pick" />
  </view>
</template>
<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { onLoad, onReachBottom, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiLoading from '../../components/FiLoading.vue'
import FiSegmented from '../../components/FiSegmented.vue'
import FiTransferCard from '../../components/FiTransferCard.vue'
import TransferNominateSheet from './components/TransferNominateSheet.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import type { TransferIntent } from '../../types/transfer'
import { buildSharePath } from '../../utils/shareSource'
import { TRANSFER_BOARD_LABELS, TRANSFER_INTENTS, buildTransferBoardUrl, buildTransferViewUrl, parseTransferBoardQuery, transferBoardShareTitle } from '../../utils/transferLinks'
import { createTransferBoard } from './useTransferBoard'
import { createTransferNominate } from './useTransferNominate'

const { navigationPageStyle } = useNavigationLayout()
const state = createTransferBoard()
const { teamId, intent, board, items, loading, error, hasMore, pending, vote } = state
const nominate = createTransferNominate()
const intentItems = TRANSFER_INTENTS.map(key => ({ key, label: TRANSFER_BOARD_LABELS[key] }))
const wantsNominate = ref(false)
let started = false

onLoad(query => {
  const parsed = parseTransferBoardQuery(query as Record<string, unknown>)
  teamId.value = parsed.teamId; intent.value = parsed.intent; wantsNominate.value = parsed.nominate
})
onShow(() => {
  if (!teamId.value) return
  // 返回本页（例如从单卡页赞踩后）重新读当前榜，保持名次与票数一致。
  void state.load(teamId.value, intent.value).then(() => {
    if (!started && wantsNominate.value && board.value?.window?.open) void nominate.open(teamId.value, intent.value)
    started = true
  })
})
onReachBottom(() => { void state.more() })
onShareAppMessage(() => ({
  title: board.value?.window ? transferBoardShareTitle(board.value.team.team_name, board.value.window.name, intent.value) : '球迷转会榜',
  path: buildSharePath(buildTransferBoardUrl(teamId.value, intent.value), 'transfer_board'),
}))
onBeforeUnmount(state.dispose)

function changeIntent(which: TransferIntent): void { void state.changeIntent(which) }
function retry(): void { if (error.value) void state.load(teamId.value, intent.value); else void state.more() }
function openCard(code: string): void { const url = buildTransferViewUrl(code); if (url) uni.navigateTo({ url }) }
function openNominate(): void { void nominate.open(teamId.value, intent.value) }
async function pick(playerId: number): Promise<void> {
  const result = await nominate.submit(playerId)
  if (result) void state.load(teamId.value, nominate.intent.value)
}
function goBack(): void { uni.navigateBack({ fail: () => uni.switchTab({ url: '/pages/insights/index' }) }) }
</script>
<style scoped src="./transfer.css"></style>
```

`src/pages/transfer/view.vue`：

```vue
<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBack" />
  <view class="transfer-page">
    <FiLoading v-if="loading && !view" title="提名加载中" caption="正在读取这张提名。" />
    <text v-else-if="error && !view" class="transfer-error">{{ error }}</text>
    <template v-else-if="view">
      <text class="transfer-title">{{ view.team.team_name }}球迷希望 {{ view.card.player.name }} {{ transferStance(view.card) }}</text>
      <text class="transfer-caption">{{ view.window.name }} · {{ TRANSFER_BOARD_LABELS[view.card.intent] }}{{ view.card.rank ? ` 第 ${view.card.rank}` : '' }}{{ view.window.open ? '' : ' · 本期投票已结束' }}</text>
      <view class="transfer-panel">
        <FiTransferCard size="md" :card="view.card" :show-rank="false" :pending="pending.includes(view.card.share_code)" @vote="vote" />
      </view>
      <text v-if="hint" class="transfer-caption">{{ hint }}</text>
      <view class="transfer-row">
        <button class="transfer-action transfer-action--active" open-type="share">分享拉票</button>
        <button class="transfer-action" @tap="openBoard">看完整榜单</button>
      </view>
      <button v-if="view.window.open" class="transfer-action" @tap="openNominate">我也提名</button>
    </template>
  </view>
</template>
<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { onLoad, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiLoading from '../../components/FiLoading.vue'
import FiTransferCard from '../../components/FiTransferCard.vue'
import { getTransferCard } from '../../api/transfer'
import { createTransferVotes, transferVoteDeps } from '../../composables/useTransferVotes'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import type { TransferCardView, TransferVoteValue } from '../../types/transfer'
import { extractApiErrorMessage } from '../../utils/apiError'
import { buildSharePath } from '../../utils/shareSource'
import { TRANSFER_BOARD_LABELS, applyTally, buildTransferBoardUrl, buildTransferViewUrl, parseTransferCode, transferCardShareTitle, transferStance } from '../../utils/transferLinks'

const { navigationPageStyle } = useNavigationLayout()
const session = useAppSession()
const voteDeps = transferVoteDeps()
const votes = createTransferVotes(voteDeps)
const pending = votes.pending
const code = ref(''), view = ref<TransferCardView | null>(null), loading = ref(false), error = ref('')
let generation = 0, loadedRevision = -1
const hint = computed(() => {
  const value = view.value
  if (!value || !value.window.open || value.can_vote) return ''
  return voteDeps.loggedIn() ? '只能为你的主队投票' : '登录并设置主队后可以投票'
})

async function load(): Promise<void> {
  if (!code.value) { error.value = '分享链接无效'; return }
  const owner = ++generation
  loading.value = true; error.value = ''
  try {
    const result = await getTransferCard(code.value)
    if (owner === generation) view.value = result
  } catch (cause) {
    if (owner === generation) error.value = extractApiErrorMessage(cause, '这张提名暂时打不开，请稍后重试')
  } finally {
    if (owner === generation) loading.value = false
  }
}
async function vote(value: TransferVoteValue): Promise<void> {
  const current = view.value
  if (!current) return
  if (voteDeps.loggedIn() && !current.can_vote) {
    uni.showToast({ title: current.window.open ? '只能为你的主队投票' : '本期投票已结束', icon: 'none' })
    return
  }
  const owner = generation
  const message = await votes.toggle(current.card, value, {
    returnUrl: buildTransferViewUrl(code.value),
    current: () => owner === generation,
    publish: tally => { if (view.value) view.value = { ...view.value, card: applyTally(view.value.card, tally) } },
  })
  if (message) uni.showToast({ title: message, icon: 'none' })
}

onLoad(query => { code.value = parseTransferCode(query as Record<string, unknown>) })
onShow(() => {
  if (loadedRevision !== session.revision.value) { loadedRevision = session.revision.value; void load() }
})
onShareAppMessage(() => ({
  title: view.value ? transferCardShareTitle(view.value.card) : '球迷转会投票',
  path: buildSharePath(buildTransferViewUrl(code.value), 'transfer_card'),
}))
onBeforeUnmount(() => { generation++; votes.reset() })

function openBoard(): void { if (view.value) uni.navigateTo({ url: buildTransferBoardUrl(view.value.team.team_id, view.value.card.intent) }) }
function openNominate(): void { if (view.value) uni.navigateTo({ url: buildTransferBoardUrl(view.value.team.team_id, view.value.card.intent, { nominate: true }) }) }
function goBack(): void { uni.navigateBack({ fail: () => uni.switchTab({ url: '/pages/insights/index' }) }) }
</script>
<style scoped src="./transfer.css"></style>
```

- [ ] **Step 9: 注册分包与预加载**

`src/pages.json`：在 `subPackages` 数组末尾加

```json
{
  "root": "pages/transfer",
  "pages": [
    { "path": "board", "style": { "navigationBarTitleText": "转会榜", "navigationStyle": "custom" } },
    { "path": "view", "style": { "navigationBarTitleText": "转会提名", "navigationStyle": "custom" } }
  ]
}
```

`preloadRule` 中 `pages/insights/index` 与 `pages/home/index` 的 `packages` 都加入 `"pages/transfer"`。

- [ ] **Step 10: 写分包样式审计**

`scripts/transfer-style-audit.test.mjs`（规则与 `lineup-style-audit.test.mjs` 相同，范围为分包目录与两个主包组件）：

```js
import { describe, expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

// 转会投票样式规范：无渐变、无散写 hex、无数字字重。
const SRC = fileURLToPath(new URL('../src', import.meta.url))
const listFiles = (dir, ext) => readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
  const path = join(dir, entry.name)
  if (entry.isDirectory()) return listFiles(path, ext)
  return ext.some((e) => entry.name.endsWith(e)) ? [path] : []
})
const FILES = [
  ...listFiles(join(SRC, 'pages/transfer'), ['.vue', '.css']),
  join(SRC, 'components/FiTransferCard.vue'),
  join(SRC, 'pages/insights/components/InsightsTransferPanel.vue'),
]
function styleOf(path) {
  const source = readFileSync(path, 'utf8')
  return path.endsWith('.css') ? source : [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]).join('\n')
}
function violations(style) {
  const found = []
  style.split('\n').forEach((line, index) => {
    const code = line.replace(/\/\*.*?\*\//g, '')
    if (/gradient/i.test(code)) found.push(`${index + 1}: gradient`)
    if (/#[0-9a-fA-F]{3,8}\b/.test(code)) found.push(`${index + 1}: hex`)
    if (/font-weight:\s*\d/.test(code)) found.push(`${index + 1}: numeric font-weight`)
  })
  return found
}

describe('transfer style audit', () => {
  test('covers the subpackage and shared components', () => {
    expect(FILES.length >= 6).toBe(true)
  })
  for (const file of FILES) {
    test(`${file.replace(`${SRC}/`, '')} follows token rules`, () => {
      expect(violations(styleOf(file))).toEqual([])
    })
  }
})
```

- [ ] **Step 11: 全量检查与构建**

Run:

```bash
bun test > /tmp/transfer-mini.log 2>&1; grep -E "^\(fail|^ *[0-9]+ (pass|fail)" /tmp/transfer-mini.log
bun run type-check 2>&1 | tail -1
bun run check:boundaries | tail -1
bun run build:mp-weixin > /tmp/transfer-build.log 2>&1; echo build=$?
bun run verify:mp-components | tail -1
ls dist/build/mp-weixin/pages/transfer
```

Expected: 0 fail；类型、边界通过；`build=0`；组件已注册；分包目录含 `board.*`、`view.*`。若构建后 `manifest.json` / `generatedMiniProgramVersion.ts` 被改动，用 `git checkout --` 还原（版本号只由发版脚本处理）。

- [ ] **Step 12: 提交**

```bash
git add src/pages/transfer src/pages.json scripts/transfer-style-audit.test.mjs
git commit -m "feat(mini): add transfer board, card page and nominate sheet

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: 首页入口与文档

**Files:**
- Modify: `football_insight_mini/src/pages/home/components/HomeSupportPanel.vue`
- Modify: `football_insight_mini/src/pages/home/index.vue`
- Modify: `CHANGELOG.md`、`FEATURES.md`、`DEPLOYMENT.md`

**Interfaces:**
- Consumes: Task 6 `buildTransferBoardUrl`。
- Produces: `HomeSupportPanel` 新事件 `open-transfer`。

- [ ] **Step 1: 首页卡片加入口**

`HomeSupportPanel.vue`：在"排阵容"入口 `</view>` 之后加同样结构的第二行（复用同一组样式类，加一个修饰类去掉上边框与上外边距，使两行共用一条分割线）：

```vue
    <view
      v-if="!showLoading && !errorMessage"
      class="support-home-panel__lineup support-home-panel__lineup--follow"
      hover-class="support-home-panel__lineup--pressed"
      @click="emit('open-transfer')"
    >
      <text class="support-home-panel__lineup-title">{{ favoriteTeam ? `${favoriteTeam.team_name}` : '' }}转会投票</text>
      <text class="support-home-panel__lineup-action">去表态</text>
    </view>
```

`defineEmits` 增加 `(e: 'open-transfer'): void`；样式加

```css
.support-home-panel__lineup--follow { margin-top: var(--fi-space-16); padding-top: 0; border-top: 0; }
```

`src/pages/home/index.vue`：`<HomeSupportPanel>` 上加 `@open-transfer="openTransferBoard"`；script 加

```ts
import { buildTransferBoardUrl } from '../../utils/transferLinks'
function openTransferBoard() {
  uni.navigateTo({ url: buildTransferBoardUrl(supportFavoriteTeam.value?.team_id ?? 0) })
}
```

- [ ] **Step 2: 检查与构建**

Run: `bun test > /tmp/transfer-home.log 2>&1; grep -E "^\(fail|^ *[0-9]+ (pass|fail)" /tmp/transfer-home.log && bun run type-check 2>&1 | tail -1 && bun run check:boundaries | tail -1 && bun run build:mp-weixin > /tmp/transfer-build2.log 2>&1; echo build=$? && bun run verify:mp-components | tail -1`
Expected: 全部通过。

- [ ] **Step 3: 文档**

- `CHANGELOG.md` 顶部新增一条"2026-10-20 · 转会投票第一期（代码完成，待迁移与发布）"：变更、原因（休赛期粘性，spec 路径）、验证（各测试数量与集成测试结果，写实际数字）、发布状态（迁移未执行、本地后端已重启但迁移未执行所以接口为 503、生产未部署、小程序未上传）。
- `FEATURES.md` "已完成"下新增"转会投票第一期"小节，三到四行概述行为与边界（无文字、只投主队、每期 3 张、往期只读、审核不隐藏）。
- `DEPLOYMENT.md` 增加上线顺序：①peiqian `cargo run --bin run_migrations` 应用 `20261020120000`（会写入第一期"2026 冬窗"）→ ②部署后端 → ③上传小程序；并注明新开一期的 SQL：

```sql
INSERT INTO f_i_transfer_windows (code, name, opens_at, closes_at)
VALUES ('2027-summer', '2027 夏窗', '2027-06-01T00:00:00+08:00', '2027-08-01T00:00:00+08:00');
```

- [ ] **Step 4: 提交并推送**

```bash
git add football_insight_mini/src/pages/home CHANGELOG.md FEATURES.md DEPLOYMENT.md
git commit -m "feat(mini): add the home entry for transfer votes and document the release

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git pull --rebase && git push
```

---

## Self-Review 记录

- **Spec 覆盖**：第 4 节三表与第一期 → Task 1；第 5 节规则 1–8 → Task 1（纯规则）、Task 2（编排）、Task 3（SQL：上限、空卡、排序、实时信息、停用账号）；第 6 节接口（除第二期 wxacode）→ Task 4；第 7 节结构 → Task 1–4；第 8.1 → Task 5–7、9；第 8.2（海报除外）→ Task 8；第 8.3 状态 → Task 7、8（hint、已结束、加载/出错、空榜）；第 9 节好友卡片分享 → Task 7（洞察页）、Task 8（榜单与单卡）；海报 → 第二期，不在本计划；第 11 节测试 → 各任务。
- **类型一致性**：`TransferCard.my_vote` 前后端均为 -1/0/1；`setTransferVote(code, 0)` 对应 `DELETE`；`can_vote` 在 summary、board、card view 三处一致；`FiTransferCard` 事件 `vote(1|-1)` 与 `createTransferVotes.toggle` 的 `TransferVoteValue` 一致。
- **测试文件参与 type-check**：`tsconfig.json` 包含 `src/**/*.ts`，所以 `src/` 下的测试模拟对象都写成可通过类型检查的形式。
- **已知取舍**：错误体为纯文本（见"已裁定的实现细节"）；排名在 SQL 内完成；只统计正常账号的票。
