# 余票看板"比赛 ID 解锁"设计（match-id-unlock）

日期：2026-08-26
状态：已与用户确认（方案 A + 软门槛）

## 目标

在余票看板页面（`football_insight_mini/src/pages/ticket-watch/index.vue`）的当前比赛操作区新增"比赛 ID"按钮，用于查看当前比赛的内部 `match_id`：

- 会员等级 V6 及以上：免费查看，可复制。
- 低于 V6：弹窗提示两个选项——升级到 V6，或按次付费 5 元解锁本场比赛 ID（接入微信支付，真实交易）。

## 已确认的产品决策

| 决策点 | 结论 | 备注 |
| --- | --- | --- |
| 付费范围 | 真接微信支付 | 复用现有 wxpay + 订单链路 |
| id 含义 | 内部 `match_id`（`f_i_matches.match_id`，业务唯一键，数字） | 非 `external_match_id` |
| 计费口径 | 按场次 | 5 元解锁一场，永久可见；换场次需重新付费；V6+ 全部免费 |
| 门槛强度 | 软门槛 | 看板接口继续明文返回 `match_id`，老客户端零影响；详见"向后兼容" |

价格为一口价 500 分（常量），本期不做可配置。

## 方案对比

- **方案 A（采纳）：独立小模块 + 复用支付链路。** 新建 Rust 模块 `match_id_unlock`，复用 payment 模块暴露的 `OrderRepository`、`WechatPayPort`、`UserMembershipPort`、`TokenPort`；微信回调按 `product_type` 新增结算分支。概念清晰、改动面小、完全增量。
- **方案 B（否决）：塞进 reflux_subscription 当一种套餐。** 语义不符（回流订阅是提醒产品），product_type 编码混乱。
- **方案 C（否决）：通用"次数包/余额"体系。** 目前只有一个按次场景，过度设计（YAGNI）。

## 后端设计（football_insight_service_backend_rs）

### 数据表

新迁移 `migrations/20260826140000_add_match_id_unlocks.sql`（纯增量；版本号需避开已占用的 `20260826120000_add_mini_review_statuses.sql` 与 `20260826130000_seed_wechat_next_match_reply_enabled.sql`）：

```sql
CREATE TABLE IF NOT EXISTS f_i_user_match_id_unlocks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL,
  match_id BIGINT NOT NULL,
  order_no TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (user_id, match_id),
  UNIQUE (order_no)
);

CREATE INDEX IF NOT EXISTS idx_f_i_user_match_id_unlocks_user
  ON f_i_user_match_id_unlocks (user_id);
```

### 模块布局（仿 mini_review / reflux_subscription）

```
src/match_id_unlock/
  domain/match_id_unlock.rs        # 常量 MATCH_ID_UNLOCK_PRICE_CENTS = 500、via 枚举
  ports/match_id_unlock_repository.rs
  application/get_match_id_entitlement.rs
  application/create_match_id_order.rs
  adapters/web/{dto.rs, handlers.rs, routes.rs}
  adapters/persistence/postgres_match_id_unlock_repository.rs
  bootstrap.rs
```

`bootstrap.rs` 暴露 `build_match_id_unlock_routes(pool, order_repository, wechat_pay_port, user_membership_port, token_port) -> Router`，在 `app.rs` merge。仓库 trait 提供两个能力：

- `find_unlock(user_id, match_id) -> Option<StoredUnlock>`（含 `match_exists(match_id)` 存在性检查，直接查 `f_i_matches`）
- `insert_unlock(user_id, match_id, order_no)`（结算侧使用）

### 端点（均需 Bearer JWT，鉴权方式仿 payment handlers 内 `authenticate_user`）

**1. 查询解锁状态**

```
GET /api/v1/match-id/entitlement?match_id={id}
```

响应 200：

```json
{ "unlocked": true, "via": "membership" | "purchase" | null, "effective_tier": "V6" }
```

判定顺序：

1. JWT 解析用户（401 未登录）。
2. `match_exists` 校验（404 比赛不存在）。
3. `get_user_membership_tier(user_id)`（该 port 实现已含 `resolve_effective_membership_tier` 过期回退 V3 逻辑）→ `membership_tier_rank >= membership_tier_rank("V6")` → `via: "membership"`。
4. 否则查 unlocks 表，命中 → `via: "purchase"`。
5. 否则 `unlocked: false, via: null`。

**2. 下单**

```
POST /api/v1/match-id/order
{ "match_id": 12345 }
```

响应 200：`{ "order_no": "...", "wx_pay_params": { timeStamp, nonceStr, package, signType, paySign } }`

（字段名有意用 `wx_pay_params`，与 reflux DTO 的 `params` 不同；前后端在本设计内保持一致，实现时不要"改回" `params`。）

下单前校验（按序）：

- 未登录 → 401
- 比赛不存在 → 404
- V6+ 会员 → 400 "V6 及以上会员可直接查看，无需购买"（正常前端流程不会到达，防御）
- 本场已解锁 → 409 "本场比赛 ID 已解锁，无需重复购买"
- 未绑定微信（`get_user_open_id` 为空）→ 403 "请先绑定微信后再支付"（校验点与 reflux 下单一致；注意 reflux 把该错误映射为 400，本端点定为 403，实现时不要对齐 reflux）

通过后：`OrderRepository.create_order(NewPaymentOrder { amount_cents: 500, product_type: "match_id_unlock:{match_id}", ... })` → `WechatPayPort.unified_order(order_no, "解锁比赛ID", 500, openid)`，返回订单号与支付参数（与 reflux 订阅下单用例同构）。每次点击生成新订单；旧 pending 单自然遗留、不主动处置（与 reflux 实际行为一致，页面已有轮询 + entitlement 刷新兜底）。

### 支付结算

- `payment/domain/order.rs` 新增 `parse_match_id_unlock_product_type(&str) -> Option<i64>`（仿 reflux 解析器，解析 `match_id_unlock:{match_id}` 中的场次 id）。
- `handle_wechat_notify.rs` 在现有 reflux 分支旁增加 match_id_unlock 分支。
- `PaymentSettlementPort` trait 新增 `settle_match_id_unlock_order(order_no, transaction_id, user_id, match_id)`（与现有 settle 方法的参数风格一致），Postgres 实现在事务内：订单置 paid（复用现有置 paid SQL 模式）+ `INSERT INTO f_i_user_match_id_unlocks ... ON CONFLICT (user_id, match_id) DO NOTHING`（重复回调幂等）。

### 缓存

带 `Authorization` 头的 GET 本就不进 `http_cache`（`should_cache_request` 排除）。防御性地把 `/api/v1/match-id/` 前缀加入 `matches_excluded_prefix`，并补一条回归测试。

## 前端设计（football_insight_mini）

### API 层

新增 `src/api/matchIdUnlock.ts`：

```ts
export interface MatchIdEntitlement {
  unlocked: boolean
  via: 'membership' | 'purchase' | null
  effective_tier: string
}
export function getMatchIdEntitlement(matchId: number): Promise<MatchIdEntitlement>
export function createMatchIdOrder(matchId: number): Promise<{ order_no: string; wx_pay_params: WxPayParams }>
```

订单状态轮询复用 `src/api/payment.ts` 现有 `getOrderStatus(orderNo)`。

### 组件

新增 `pages/ticket-watch/components/TicketMatchIdSheet.vue`（该页首个局部组件，遵循项目组件规范：props 进、emits 出、不调业务 API）：

- props：`visible`、`matchId: number | null`、`matchLabel: string`（轮次 + 对阵 + 日期摘要）、`state: 'loading' | 'locked' | 'unlocked' | 'paying'`、`via`、`effectiveTier`
- emits：`close`、`pay`、`copy`、`upgrade`（升级按钮由页面处理跳转，组件不跳页面）
- 已解锁态：比赛摘要 + 大字号 match_id + "复制"按钮（组件内 `uni.setClipboardData`，平台能力不属于业务 API）+ 来源说明（"会员权益" / "单场解锁"）
- 锁定态：说明文案"V6 及以上会员可免费查看，或支付 ¥5 解锁本场比赛 ID"；主按钮"¥5 解锁本场"（emits pay）、次按钮"升级到 V6"（emits upgrade）
- paying 态：主按钮置 loading 并禁用
- 视觉沿用页面现有 subscription-dialog / recent-reflux-lock 的卡片与按钮语言

### 页面编排（index.vue，约 100 行新增）

ticket-watch 整体重构在 backlog 中，本次不动存量结构；页面新增代码保持薄编排（状态 + API 调用），展示逻辑一律下沉到组件与 helpers。

按钮放在 `watch-monitor-actions` 区，与"停止监控 / 订阅提醒"并列，复用 `watch-monitor-actions__button` 样式。`openMatchIdSheet()` 流程：

1. 无 `currentMatch` 直接返回；未登录按页面现有 401/未登录惯例引导。
2. 打开 sheet（state=loading）→ `getMatchIdEntitlement(match_id)` → unlocked / locked。
3. pay 事件：先检查微信绑定（复用页面 `has_wechat_binding` 状态，先例 `goToMembershipPurchase` 引导去"我的"页绑定）；state=paying → `createMatchIdOrder` → `uni.requestPayment`（仅 MP-WEIXIN；H5 弹"请在微信小程序内完成支付"并回到锁定态）→ 复用页面现有 `waitForPaidOrder` 轮询 → paid 后刷新 entitlement → unlocked（via=purchase）。
4. copy 事件由组件内部完成并 toast。

极端窗口说明：若订单 A 的支付回调延迟、用户又下新单 B 并支付，可能出现双付款；该窗口继承自 reflux 先例，由"下单前 409 已解锁校验 + 支付后 entitlement 刷新"缓解，本期不做自动退款。

### helpers 纯函数（+ 单测）

`pages/ticket-watch/helpers.ts` 仿 `resolveRecentRefluxPanelMode` 先例新增：

- `resolveMatchIdSheetState(entitlement): 'unlocked' | 'locked'`
- `buildMatchIdSourceLabel(via): string`（"会员权益" / "单场解锁"）

## 错误处理

| 场景 | 行为 |
| --- | --- |
| 未登录点按钮 | 按页面现有未登录/401 惯例引导登录 |
| 未绑定微信点付费 | 引导去"我的"页绑定（仿 `goToMembershipPurchase` 先例）；后端下单也返回 403 兜底 |
| entitlement 404（比赛不存在） | toast 并关闭 sheet |
| 下单 409（已解锁） | 刷新 entitlement 后展示解锁态 |
| 支付取消/失败 | toast"支付未完成"，回到锁定态，不落权益 |
| 轮询超时未确认 | toast"支付结果确认中，请稍后再试"；下次打开 sheet 时 entitlement 会反映真实结算结果 |
| 重复点击 | paying 态禁用按钮防抖 |

## 向后兼容声明（软门槛）

- 现有端点与响应**零改动**：看板接口继续明文返回 `match_id`，老客户端与回流订阅下单完全不受影响。
- 新增内容全部为增量：新表、新路由、notify 新分支、http_cache 新排除前缀。
- 软门槛的含义：按钮与 entitlement 接口约束正常用户；抓包高手理论上可在看板响应里看到 id。**硬门槛**（看板接口对未解锁用户脱敏 `match_id`）需连带把回流订阅下单改为服务端解析场次，回归风险大，作为后续独立需求，届时单独决策。
- 按用户约定：凡向后兼容与最优方案冲突时，须提交用户决策，不得自行取舍。

## 测试

- 后端 `cargo test`：
  - entitlement 用例：V6 解锁 / 低于 V6 锁定 / 已购解锁 / 会员过期回退 V3 后锁定 / 比赛不存在 404
  - 下单用例：V6 拒绝 / 已解锁 409 / 未绑定微信 403 / 比赛不存在 404 / 正常下单（金额 500、product_type 编码、调用 unified_order）
  - 结算幂等：重复 notify 只落一行 unlock
  - product_type 解析往返
  - http_cache 排除前缀回归
- 前端 `bun test`：helpers 纯函数单测；`bun run build:mp-weixin`（含边界检查与组件注册检查）。
- 手动：dev:h5 验证锁定/解锁两种 UI 态；小程序预览验证支付全链路（真实扣款，由用户择机执行）。

## 不做的事（Out of Scope）

- 硬门槛脱敏（后续独立需求，需用户决策）
- H5 端支付
- 价格可配置化 / 次数包 / 余额体系
- ticket-watch 页面整体组件化重构（本设计只新增一个局部组件，大重构另行安排）
