# 公众号「下一场」比赛 ID 会员门控设计

日期：2026-10-16
状态：用户已确认方案，待交付实现
范围：`football_insight_service_backend_rs`（auth、match_id_unlock 模块）
不涉及：小程序（无需发版）、数据库迁移、微信后台配置、`football_insight_admin_android`

## 1. 背景与决定

公众号里用户回复「下一场」或「下一场id」，后端会把当前下一场比赛的 ID 回复给**任何关注者**，不检查会员等级。生产里开关 `wechat_next_match_reply_enabled=true`（自 2026-08-26 起）。小程序里同一个信息要"V6 及以上免费看，或 ¥5 单场解锁"，两边规则不一致，公众号等于把付费点免费送出。

**用户决定**：公众号的「下一场」保留，但要先判断这个用户是不是 V6 及以上会员：是则回复比赛 ID，不是则告诉他去小程序开通 V6 及以上会员。

### 1.1 已核实的事实（均为只读查询/读代码）

- 回复逻辑在 `src/auth/application/handle_wechat_webhook.rs`：命令 `下一场id` / `下一场` 命中后读开关，开着就调 `CurrentStandardMatchPort::fetch_current_match_id()`，把 ID 拼进 `next_match_id_message`。全程不看用户。
- 公众号消息里只有**公众号 openid**（`incoming.from_user_name`）。库里能对应到用户的只有 `f_i_users.official_wechat_open_id`（注册时由公众号发的邀请码带入）。当前 `AuthRepository` 没有"按公众号 openid 查用户"的方法。
- 小程序弹层的规则（`match_id_unlock`）：生效等级 ≥ `MATCH_ID_UNLOCK_MINIMUM_TIER`（`"V6"`），或该用户买过本场解锁（`f_i_user_match_id_unlocks`）。会员过期按 `resolve_effective_membership_tier` 回落到 V3。公众号必须和它**同一规则**。
- 数据：39 个有效 V6+ 会员中，**7 个（约 18%）没有 `official_wechat_open_id`**，公众号无法识别他们。生产里 `f_i_user_match_id_unlocks` 为 0 行（¥5 解锁目前买不了）。
- 比赛 ID 本身在公开接口（`GET /api/v1/ticket-watch/matches` 等）里无需登录就能拿到，所以这是"便利性"门控，不是保密。这是已知并接受的取舍，本期不处理。

## 2. 目标行为

所有触发「下一场」的入口（文字命令及其数字快捷方式，实现者需读代码确认命令归一化逻辑，**所有入口必须走同一个门控**）：

| 情形 | 回复 |
|---|---|
| 开关关闭 | 保持现状（`next_match_missing_message`） |
| 当前没有下一场 / 获取失败 | 保持现状（`next_match_missing_message`），**不暴露任何会员信息** |
| 识别到用户，且生效等级 ≥ V6 或已解锁本场 | 保持现状：回复比赛 ID（`next_match_id_message`，文案不变） |
| 识别到用户，但不满足 | **非会员文案**（见 2.1） |
| 公众号 openid 找不到对应用户 | **未识别文案**（见 2.1） |
| 身份查询出错 | **fail closed**：回复"暂时无法确认"文案，**绝不回复 ID** |

判断顺序：开关 → 取当前比赛 ID（没有就走 missing，不做身份判断）→ 身份判断 → 回复。

### 2.1 文案（定稿，实现者直接使用，均在末尾保留现有 `with_product_intro`）

**非会员**（识别到用户但不满足条件）：

```
【足球洞察】当前比赛信息

下一场比赛 ID 是会员权益，V6 及以上会员可直接查询。

开通方式：打开小程序「足球洞察」，在「我的」页点会员卡上的「升级」，选择 V6 及以上档位即可。

常用指令：
- 回复“邀请码”或“1”获取注册邀请码
```

**未识别**（公众号 openid 没有对应账号）：

```
【足球洞察】当前比赛信息

下一场比赛 ID 是会员权益，V6 及以上会员可直接查询。

暂未识别到你的会员账号。已是 V6 及以上会员的话，可以直接在小程序「余票监控」页点「获取比赛 ID」查看；还不是会员的话，在小程序「我的」页点会员卡上的「升级」即可开通。

常用指令：
- 回复“邀请码”或“1”获取注册邀请码
```

**身份查询出错**：

```
【足球洞察】当前比赛信息

暂时无法确认你的会员身份，请稍后再回复“下一场”重试。
```

文案里**不得**提 ¥5 解锁（目前买不了，商品映射未配置），也**不得**包含比赛 ID。

## 3. 设计

遵循 `domain / ports / application / adapters`，业务规则放 domain，SQL 放 adapter，不要写进 web handler。

### 3.1 domain（纯函数，放 `src/match_id_unlock/domain/`）

```rust
pub enum OfficialAccountMatchIdAccess { Granted, NotMember, Unrecognized }

pub struct OfficialAccountUser {
    pub membership_tier: String,
    pub membership_expires_at: Option<DateTime<Utc>>,
    pub has_unlock_for_match: bool,
}

pub fn resolve_official_account_access(
    user: Option<&OfficialAccountUser>, now: DateTime<Utc>,
) -> OfficialAccountMatchIdAccess
```

规则：`None` → `Unrecognized`；否则用 `resolve_effective_membership_tier(tier, expires_at, now)` 得生效等级，`membership_tier_rank(生效等级) >= membership_tier_rank(MATCH_ID_UNLOCK_MINIMUM_TIER)` 或 `has_unlock_for_match` → `Granted`，其余 `NotMember`。必须复用现有的等级函数和常量，不要重新写 V6 的判断。

### 3.2 port（定义在 `src/auth/ports/`，由 auth 消费，避免 auth 依赖 match_id_unlock 的内部类型）

```rust
#[async_trait]
pub trait OfficialAccountMatchIdAccessPort: Send + Sync {
    async fn check(&self, official_open_id: &str, match_id: Option<i64>)
        -> anyhow::Result<OfficialAccountMatchIdAccess>;
}
```

`OfficialAccountMatchIdAccess` 放 auth 的 domain（枚举很小），match_id_unlock 的 domain 函数返回同一个类型或做一次转换，两者二选一，但**不要出现 auth 与 match_id_unlock 互相 use 对方内部模块的循环依赖**；由 `bootstrap.rs` / `app.rs` 装配实现。

### 3.3 adapter（放 `src/match_id_unlock/adapters/persistence/`）

一条只读 SQL：按 `f_i_users.official_wechat_open_id = $1 AND status = 'active'` 取 `membership_tier`、`membership_expires_at`，并 `EXISTS` 查 `f_i_user_match_id_unlocks`（`match_id` 为 `None` 时不查，`has_unlock_for_match = false`）。没有行 → 传 `None` 给 domain 函数。取不到/出错 → 返回 `Err`，由调用方 fail closed。

注意微信被动回复必须在 **5 秒内**返回：只走这一条数据库查询，**不要**再调用 `known_match_id` / `fetch_all_matches`（当前比赛 ID 本来就来自票务监控的 current-standard，是可信的）。

### 3.4 webhook 改动（`handle_wechat_webhook.rs`）

- `HandleWechatWebhookUseCase::new` 增加 `Arc<dyn OfficialAccountMatchIdAccessPort>`，`bootstrap.rs` 装配。
- 把「下一场」回复提成一个私有方法，按 2 节的顺序处理；新增三个文案函数（非会员、未识别、查询出错）。
- 比赛 ID 字符串解析为 `i64` 失败：`match_id = None`（仍可凭等级放行，不能凭解锁放行），不要 panic。
- 日志：记录判定结果（granted / not_member / unrecognized / error），**不要把比赛 ID 写进日志**，已有的 open_id 日志保持现状。

### 3.5 不需要的东西

- 不新增迁移、不新增配置键。现有的 `wechat_next_match_reply_enabled` 继续作为总开关（想临时全关就设 `false`）。
- 不改小程序、不发版。

## 4. 测试要求

**domain 单测**（表驱动）：V5 → NotMember；V6/V7/V8/V9 → Granted；V9 但已过期（回落 V3）→ NotMember；V3 + 已解锁本场 → Granted；V9 过期 + 已解锁 → Granted；`None` → Unrecognized；`match_id` 不可解析时仅凭等级放行。

**webhook 用例测试**（沿用该文件已有的 fake 写法，`CurrentStandardMatchPort` 与 `SystemConfigPort` 的 fake 已存在）：
1. 会员（Granted）→ 回复里**包含**比赛 ID。
2. 非会员 → 回复含"V6 及以上"和开通指引，且**不包含**比赛 ID（断言字符串里没有该 ID）。
3. 未识别 → 含未识别文案，不含比赛 ID。
4. 身份查询返回 `Err` → 含"暂时无法确认"文案，不含比赛 ID（fail closed）。
5. 开关关闭、没有下一场 → 仍是 `next_match_missing_message`，且**不调用**身份端口（用计数 fake 断言）。
6. 两个文字命令（以及数字快捷方式）都走同一门控。
7. 回复仍按原有加密/明文模式正确构建（沿用现有测试断言）。

**adapter**：SQL 必须在生产库的**回滚事务**里真实验证一遍：`BEGIN` → `SET LOCAL lock_timeout='2s'` → 用一个**已存在用户**的 `official_wechat_open_id`（只读取，不修改）执行查询 → `ROLLBACK`。验证：有 openid 的用户返回等级与过期时间；不存在的 openid 返回无行；`match_id=None` 不查解锁表。脚本放会话 scratchpad，用 `uv run python` 的 psycopg，从 `football_insight_service_backend_rs/.env` 读 `DATABASE_URL`，**不要输出连接串**。

## 5. 硬性约束

1. 只有生产这一个数据库，本地后端连的就是生产 `football_data`：严禁写入、修改、删除任何用户、订单、会员数据；严禁发起真实支付；严禁往公众号发消息。
2. 不得对**任何错误路径**回复比赛 ID（fail closed 是硬性要求，审查会重点看）。
3. 不改 `wechat_next_match_reply_enabled` 配置的现有值，不改系统配置表。
4. 后端代码改动后，本地验证前先用 `./deploy/local-test/up.sh` 重启本地后端；汇报时分开写：本地后端已重启 / 生产未部署 / 生产已部署并重启。
5. **不要部署生产**（部署由审查后统一执行）。生产部署方式备忘：peiqian 上构建镜像很慢，用「本机 `docker build`、`docker save | ssh peiqian docker load`、在 peiqian 换容器」的方式。
6. 提交信息沿用风格：`feat(wechat): ...`，subject 用英文，末尾加 Co-Authored-By 行；直接提交并推送 `main`，不新建分支。同步在 `CHANGELOG.md`、`FEATURES.md` 各补一条并写明原因。

## 6. 业务预期与效果衡量

**预期（用户提出）**：用户离不开这个比赛 ID，门控之后同时拉动两个数字——**公众号使用人数**（会员要在公众号里问才能拿到 ID）和**小程序使用人数**（非会员被引导去小程序登录、开通；未识别的会员被引导去小程序「余票监控」页查看）。

这是一个待验证的假设，不是已知结果，所以先记下**基线**（2026-10-16 查询）：

| 指标 | 基线 | 来源 |
|---|---|---|
| 公众号关注者 | 556 人（当前有效 508 人） | `f_i_wechat_followers`（`unsubscribed_at IS NULL`） |
| 注册用户 | 913 人 | `f_i_users` |
| 小程序近 7 天活跃 / 近 30 天活跃 | 257 / 288 人 | `f_i_user_activity_snapshots.last_active_at` |
| 余票监控页累计访问 | 28,124 次 | `f_i_user_page_activity_snapshots`（`page_key='ticket_watch'`） |

**观察窗口**：下一个主场是 2026-10-24，开票在 10 月 22 日前后，是最合适的第一次对比；上线后第一个开票窗口结束时，用同样的查询复盘一次。

**实现者需要做的最小埋点（无需迁移）**：每次「下一场」回复在日志里输出一行结构化判定，例如 `next_match_id_reply decision=granted|not_member|unrecognized|error|disabled|no_match`（**不含比赛 ID、不含 openid**）。持久化的统计表不在本期范围，后续如需要再单独做。

**需要警惕的反面**：门控收紧了一个原本免费的行为。如果"离不开"变成"被卡住"，可能表现为取关增多或小程序没有跟着涨。复盘时同时看取关数（`unsubscribed_at` 新增），不要只看增长。

## 7. 已知局限与后续（本期不做）

- 约 18% 的 V6+ 会员没有公众号 openid，会看到"未识别"文案；文案已给出备选路径（小程序「余票监控」页）。根治需要用 `union_id` 打通公众号与小程序账号，另立任务。
- 比赛 ID 在公开接口可取，本门控不防抓包；是否收紧公开接口需要另行评估，因为小程序其他功能依赖这些字段。
- 小程序弹层里的 ¥5 解锁商品尚未配置，本期文案不提及。
- 老用户习惯的免费回复被收紧，属预期的行为变化。

## 8. 审查清单（Review 逐条核对）

1. 门控判断只有一处，所有「下一场」触发入口（含数字快捷方式）都经过它；顺序为 开关 → 取当前比赛 → 身份 → 回复。
2. 任何非 `Granted` 的路径（非会员、未识别、查询出错、ID 解析失败仅凭解锁）都**不包含比赛 ID**，测试里有字符串断言。
3. fail closed：身份端口返回 `Err` 时回复"暂时无法确认"，不回 ID。
4. 开关关闭或没有下一场时，不调用身份端口，也不暴露会员信息。
5. domain 函数复用 `resolve_effective_membership_tier`、`membership_tier_rank`、`MATCH_ID_UNLOCK_MINIMUM_TIER`，没有重写 V6 判断；表驱动单测覆盖过期回落与已解锁。
6. 没有新增迁移或配置键；SQL 只读；未对生产写数据；SQL 已在回滚事务里真实验证。
7. 只有一条数据库查询，没有额外的赛程拉取（5 秒被动回复限制）。
8. 日志里没有比赛 ID。
9. 文案与第 2.1 节逐字一致，不含 ¥5。
10. 没有 auth ↔ match_id_unlock 的循环依赖；业务逻辑不在 handler 或 SQL 里。
11. 每次「下一场」回复都有一行结构化判定日志，且日志里没有比赛 ID 和 openid。
