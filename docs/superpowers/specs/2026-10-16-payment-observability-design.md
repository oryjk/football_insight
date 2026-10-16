# 支付漏斗可观测化设计

日期：2026-10-16
状态：已确认，交付实现
范围：`football_insight_service_backend_rs`（payment 模块）、`football_insight_mini`
不涉及：定价、购买页改版、通知召回、安卓管理端、Web 后台页面

## 1. 背景与目标

生产数据（只读查询，2026-10-16）显示，会员付费漏斗存在两个无法解释的缺口：

- 购买页访问用户 367 人，创建过会员订单的用户 49 人。
- 10 月新建的 23 笔会员订单（≥¥10）无一支付成功。其中 2 个是运营测试号，其余 6 个真实用户在 10 月 8–10 日反复下单（最多 5 单/6 分钟），最终没有付款。

当前**无法回答**：这些订单为什么没付成功。原因：

1. 订单在 `handlePurchase` 的 `createMembershipOrder` 这一步就已写入 `pending`，之后拉起支付、用户取消、微信报错、网络失败，前后端都没有记录。
2. `requestDigitalPayment` 把微信返回的 `errCode/errMsg` 压成三句固定文案，原始错误丢失。
3. `create_membership_order` 先 `create_order` 再 `unified_order`；若 `unified_order` 失败，订单永久停在 `pending`，没有原因，`f_i_payment_orders.status='failed'` 在库里一条都没有被用过。
4. 10 月 8–10 日的 15 笔订单在 `f_i_virtual_payment_orders` 中没有对应行，说明没走到"签发支付参数"，但库里没有任何字段能确认失败发生在哪一步。
5. `prepay_id` 从不落库（含已支付订单），无法交叉验证。

**目标**：对每一次购买尝试，事后都能回答三个问题——走到了哪一步、在哪一步结束、结束原因是什么（取消/失败/超时/成功，带原始错误码）。

**成功标准**（上线 2 周后验收）：

- 任一 `order_no` 可查到完整事件时间线。
- 能按天、按商品类型统计各阶段人数及流失率。
- 能列出 Top N 错误码及其占比。
- 所有 pending 订单都能归因到一个明确的终止阶段，"原因不明"的比例低于 5%。

## 2. 设计原则

- **只加观测，不改行为**：不改下单逻辑、不改订单状态机、不改价格与购买页交互。观测失败绝不能影响支付主流程。
- **覆盖全部三类商品**：会员（`membership:*`）、比赛 ID 解锁（`match_id_unlock:*`）、回流订阅（`reflux_subscription:*`）。三类都经 `requestDigitalPayment`，在公共层埋点一次即可。
- **客户端事件与服务端事件分开记**：只有客户端知道用户有没有取消，只有服务端知道有没有发货。两边各写各的，用 `order_no` 关联。
- **不记个人敏感信息**：不存 openid、手机号、token、完整请求体。

## 3. 数据模型

新增迁移 `football_insight_service_backend_rs/migrations/YYYYMMDDHHMMSS_add_payment_attempt_events.sql`。时间戳必须大于现有最新 `20261014163000`。

```sql
CREATE TABLE f_i_payment_attempt_events (
    id          BIGSERIAL PRIMARY KEY,
    attempt_id  UUID        NOT NULL,           -- 一次点击「确认开通」产生一个，客户端生成，贯穿该次尝试
    order_no    TEXT,                           -- 订单创建前的失败为 NULL
    user_id     UUID        NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    scenario    TEXT        NOT NULL CHECK (scenario IN ('membership','match_id_unlock','reflux_subscription')),
    product_ref TEXT,                           -- 如 'V7'、'583'，仅用于分组
    event       TEXT        NOT NULL,           -- 见 4.1，应用层白名单校验
    source      TEXT        NOT NULL CHECK (source IN ('client','server')),
    error_code  TEXT,                           -- 微信 errCode 或服务端错误枚举名
    error_msg   TEXT,                           -- 截断到 300 字符
    detail      JSONB       NOT NULL DEFAULT '{}'::jsonb,   -- 见 3.1
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_payment_events_order   ON f_i_payment_attempt_events (order_no, occurred_at);
CREATE INDEX idx_payment_events_attempt ON f_i_payment_attempt_events (attempt_id, occurred_at);
CREATE INDEX idx_payment_events_time    ON f_i_payment_attempt_events (occurred_at DESC, event);
```

### 3.1 `detail` 允许的键（应用层白名单，其余丢弃）

`mini_version`、`wx_lib_version`、`platform`（ios/android/devtools）、`system`、`brand`、`model`、`network_type`、`elapsed_ms`（距本次 attempt 起点）、`http_status`。单条 `detail` 序列化后不超过 1 KB。

### 3.2 保留策略

保留 180 天。清理脚本不在本期范围，在文档里写明即可。

## 4. 事件定义

### 4.1 事件清单

| event | source | 触发点 | 说明 |
|---|---|---|---|
| `purchase_tapped` | client | 点「确认开通」且已勾选声明 | attempt 起点 |
| `agreement_prompted` | client | 点「确认开通」但未勾选声明，弹出声明弹层 | 量化声明环节的流失；注意同意后不会自动发起支付，需要再点一次 |
| `session_prepare_failed` | client | `prepareVirtualPayment()` 抛错 | 带原因（不支持/服务未开放/换号/网络） |
| `order_create_requested` | client | 调 `createMembershipOrder` 等之前 | |
| `order_create_failed` | client | 下单接口返回非 2xx 或网络失败 | 带 `http_status` 与服务端返回的消息 |
| `order_created` | server | `create_order` 成功 | |
| `pay_params_failed` | server | `unified_order` 返回错误 | `error_code` 为 `VirtualPaymentError` 的变体名（`Unavailable`、`ProductUnavailable`、`Provider` 等） |
| `pay_params_issued` | server | `unified_order` 成功 | |
| `pay_sheet_invoked` | client | 调 `requestVirtualPayment` 之前 | |
| `pay_cancelled` | client | `fail` 回调且 `errCode === -2` 或消息含取消 | 保留原始 `errCode/errMsg` |
| `pay_failed` | client | 其他 `fail` 回调 | 保留原始 `errCode/errMsg`（`-15007` 单独可见） |
| `pay_success_client` | client | `success` 回调 | |
| `confirm_timeout` | client | `waitForPaidOrder` 10 次轮询仍非 `paid` | |
| `delivered` | server | 结算成功（`settle_verified_order` 返回） | 与客户端侧结果交叉验证 |
| `refunded` | server | 退款回调处理成功 | |

### 4.2 attempt_id

客户端在 `purchase_tapped`（或 `agreement_prompted`）时生成 UUID v4，该次尝试的所有客户端事件携带它。服务端事件需要 attempt_id：客户端下单请求通过请求头 `X-Payment-Attempt-Id` 传递，服务端读取后写入事件。**缺失或格式非法时服务端自行生成**，不拒绝请求（兼容旧客户端）。

## 5. 后端设计

遵循现有 `domain / ports / application / adapters` 四层，逻辑放 `application`，不要写进 web handler 或 SQL 层。

### 5.1 ports

`src/payment/ports/payment_event_recorder.rs`：

```rust
#[async_trait]
pub trait PaymentEventRecorder: Send + Sync {
    /// 记录失败只打 warn 日志，绝不向调用方返回错误影响支付主流程。
    async fn record(&self, event: NewPaymentEvent);
}
```

`NewPaymentEvent` 放 `domain/payment_event.rs`，包含 4.1 的字段，并提供 `PaymentEventKind` 枚举与 `from_client_str()`，客户端只允许提交 `source=client` 的那些事件（见 5.4）。

### 5.2 adapters

- `adapters/persistence/postgres_payment_event_recorder.rs`：单条 INSERT；写入失败吞掉并 `tracing::warn!`。
- 单元测试用内存 fake recorder。

### 5.3 在现有用例中埋点（服务端事件）

- `application/create_membership_order.rs`：`create_order` 成功后记 `order_created`；`unified_order` 成功后记 `pay_params_issued`；失败记 `pay_params_failed`（error 取 `VirtualPaymentError` 变体名，非该类型记 `Other`）。**记录之后原错误照常返回，不改变现有返回行为。**
- 比赛 ID 解锁与回流订阅的下单用例（`match_id_unlock/application/create_match_id_order.rs`、`reflux_subscription/application/create_reflux_subscription_order.rs`）：同样三处。三个用例共用同一个小的 helper（放 `payment/application/` 下），避免复制粘贴。
- `payment/application/virtual_payment.rs::deliver` 与 `payment/application/handle_wechat_notify.rs` 的结算成功路径：记 `delivered`；`refund` 记 `refunded`。
- recorder 通过 `bootstrap.rs` 装配，沿用现有注入方式；依赖这些用例的其他模块（`match_id_unlock`、`reflux_subscription`）通过构造参数获得 `Arc<dyn PaymentEventRecorder>`，不要跨模块直接引用 payment 的 adapter。

### 5.4 客户端事件上报接口

`POST /api/v1/payment/events`（需要 JWT，复用 `authenticate_user`）。

请求体：

```json
{
  "attempt_id": "uuid",
  "order_no": "string|null",
  "scenario": "membership|match_id_unlock|reflux_subscription",
  "product_ref": "V7",
  "events": [
    {"event": "pay_cancelled", "error_code": "-2", "error_msg": "requestVirtualPayment:fail cancel",
     "detail": {"mini_version": "1.x.x", "platform": "android", "elapsed_ms": 4210}}
  ]
}
```

校验与限制：

- `events` 每次最多 10 条；整个请求体上限 8 KB。
- `event` 必须在客户端事件白名单内（4.1 中 source=client 的），其余直接丢弃，整体仍返回 204。
- `order_no` 非空时必须属于当前用户（用 `OrderRepository::find_order_by_no` 校验 `user_id`），不属于则丢弃该批并返回 204；不暴露订单是否存在。
- `error_msg` 截断到 300 字符，`detail` 按 3.1 白名单过滤。
- 限流：同一用户每分钟最多 30 条事件，超出静默丢弃。可用进程内计数，不要求 Redis。
- **永远返回 204，除非未登录返回 401**。上报接口不能让客户端因为上报失败而改变行为。

### 5.5 只读查询接口（Admin）

放 `src/admin` 模块，沿用现有 Admin JWT 与审计约定，仅 `GET`：

- `GET /api/v1/admin/payment/funnel?from=&to=&scenario=`：按 `scenario × product_ref × event` 汇总 `attempt` 数与 `user` 数，并给出相邻阶段转化率。
- `GET /api/v1/admin/payment/errors?from=&to=`：按 `event + error_code` 汇总 Top 20，附最近一条 `error_msg` 与出现次数。
- `GET /api/v1/admin/payment/orders/{order_no}/events`：该订单的完整事件时间线，按 `occurred_at` 升序，客户端与服务端事件合并。
- `GET /api/v1/admin/payment/stuck-orders?older_than_minutes=30`：`pending` 且超过阈值的订单，附其最后一个事件和阶段（见 5.6）。

聚合查询在 `adapters/persistence` 用 SQL 实现，`application` 只做参数校验与结果组装。Web 后台页面不在本期范围，接口返回 JSON 即可。

### 5.6 终止阶段归因（derived，不改库状态）

`stuck-orders` 与漏斗统计按下表把每个 attempt 归到一个终止阶段，规则在 `domain/payment_event.rs` 写成纯函数并单测：

| 该 attempt 的最后一个事件 | 终止阶段 |
|---|---|
| `agreement_prompted` | 未同意声明 |
| `session_prepare_failed` | 会话准备失败 |
| `order_create_failed` / `pay_params_failed` | 下单失败 |
| `pay_params_issued`，无后续 | 拿到参数但未拉起支付（客户端中断/无上报） |
| `pay_sheet_invoked`，无后续 | 拉起支付后无回调（杀进程/无上报） |
| `pay_cancelled` | 用户取消 |
| `pay_failed` | 支付失败 |
| `pay_success_client` 且无 `delivered` | 已付款待确认（需人工关注） |
| `confirm_timeout` 且无 `delivered` | 确认超时（需人工关注） |
| `delivered` | 成功 |

**本期不把长期 pending 订单改成 `failed`**：`deliver()` 只接受 `Pending | Paid`，提前改状态会让迟到的支付回调无法发货。需要关单时另做设计。

### 5.7 对现有行为的影响（需要实现者确认并写入 PR 描述）

- 下单接口新增读取请求头 `X-Payment-Attempt-Id`；缺失不报错。
- 三个下单用例和结算路径各多 1–2 次 INSERT，均在主流程之后、失败被吞掉。
- 不改任何现有响应结构、状态码、订单状态。

## 6. 小程序设计

遵守 `football_insight_mini/AGENTS.md` 的分层与 `scripts/check-import-boundaries.mjs`：不新增 ALLOWLIST 例外；HTTP 只走 `utils/request.ts`；颜色等样式无改动。

### 6.1 新增文件

- `src/api/payment.ts`：追加 `reportPaymentEvents(payload)`，`POST /payment/events`，`auth: true`。
- `src/composables/paymentTelemetry.ts`：
  - `startPaymentAttempt(scenario, productRef): PaymentAttempt`，生成 `attempt_id`，记录起点时间，并采集一次 `detail` 公共字段（`mini_version` 取 `MINI_PROGRAM_VERSION`；其余来自 `uni.getSystemInfoSync()` 与 `uni.getNetworkType`，取不到就省略）。
  - `attempt.report(event, extra?)`：fire-and-forget；内部小队列合并 500ms 内的事件一次发出；**所有错误吞掉，不抛出，不弹 Toast**；未登录直接 return（沿用 `reportPageActivity` 的做法）。
  - `attempt.bindOrder(orderNo)`：下单成功后绑定，后续事件带 `order_no`。
  - 导出纯函数 `classifyPaymentFailure(failure)`：输入微信 `fail` 回调的 `{errCode, errMsg}`，输出 `{event: 'pay_cancelled'|'pay_failed', error_code, error_msg}`。**判定规则必须与 `requestDigitalPayment` 现有的取消判定完全一致**（`errCode === -2` 或消息匹配 `/cancel|取消/i`），并有单测覆盖。
- `src/composables/paymentTelemetry.test.ts`：覆盖队列合并、吞错、未登录跳过、`classifyPaymentFailure` 各分支。

### 6.2 修改现有文件

- `src/utils/virtualPayment.ts::requestDigitalPayment(params, attempt?)`：增加可选第二参数。传入时在调用前报 `pay_sheet_invoked`，`success` 报 `pay_success_client`，`fail` 按 `classifyPaymentFailure` 报取消或失败（**保留原始 errCode/errMsg**）。**对用户展示的错误文案与 reject 行为保持不变**；不传 `attempt` 时行为与现在完全一致。
- `src/composables/confirmPaidOrder.ts::waitForPaidOrder(orderNo, isCurrent, attempt?)`：轮询耗尽返回 `false` 时报 `confirm_timeout`。
- `src/pages/membership-purchase/index.vue::handlePurchase`：
  - 勾选声明后进入主流程时 `startPaymentAttempt('membership', targetTier)` 并报 `purchase_tapped`；
  - 未勾选声明走 `promptAgreement()` 时先 `startPaymentAttempt` 并报 `agreement_prompted`（该 attempt 随后被丢弃，不复用）；
  - `prepareVirtualPayment()` 抛错：报 `session_prepare_failed`（`error_msg` 取异常消息）；
  - `createMembershipOrder` 前后分别报 `order_create_requested` / `order_create_failed`（带 HTTP 状态，用 `ApiRequestError.statusCode`，**不得按文案判断**）；
  - 下单请求头带 `X-Payment-Attempt-Id`（在 `api/payment.ts` 的 `createMembershipOrder` 增加可选参数）；成功后 `attempt.bindOrder(order.order_no)`。
- `src/pages/ticket-watch/useTicketWatchMatchId.ts` 与 `useTicketWatchSubscription.ts`：同样的接入（scenario 分别为 `match_id_unlock`、`reflux_subscription`，`product_ref` 取比赛 ID / 套餐码）。两处现有的 `requestTicketWatchPayment` 与 `waitForPaidOrder` 调用传入 `attempt` 即可。
- 页面里不新增业务逻辑，只做"开始 attempt、传递 attempt、在已有 catch 分支里补报事件"。

### 6.3 测试

- `bun test`：`paymentTelemetry.test.ts` 新增；现有 `prepareVirtualPayment.test.ts`、`pages/ticket-watch/paymentActions.test.ts`、`effects.test.ts` 必须保持通过，说明 `requestDigitalPayment`/`waitForPaidOrder` 不传 `attempt` 时无行为变化。
- `bun run type-check`、`bun run check:boundaries`、`bun run verify:mp-components` 全部通过。

## 7. 测试与验证（后端）

- 单元测试（fake recorder）：每个埋点位置至少一条——成功路径写了对应事件；`unified_order` 失败时写了 `pay_params_failed` **且原错误原样返回**；recorder 返回/内部失败时主流程不受影响。
- 终止阶段归因函数 5.6：每一行规则一条用例。
- 事件上报接口：白名单外事件被丢弃；`order_no` 不属于当前用户时被丢弃且返回 204；超长字段被截断；超限被静默丢弃；未登录 401。
- Postgres 集成测试沿用 `tests/*_postgres.rs` 风格，新增 `tests/payment_events_postgres.rs`。
- **只有生产这一个数据库**（用户确认）。本地后端连接的就是生产 `football_data`，因此：
  - 自动化测试以不连库的单元测试为主（fake recorder、纯函数）。
  - `tests/payment_events_postgres.rs` 若要写库，必须把所有写入放在**事务内并在结束时回滚**，且只操作本测试自己新建的数据；不得读取、修改、删除任何真实用户、订单、会员商品数据，也不得发起真实支付。做不到就不写这份集成测试，在 PR 里说明。
  - 手工验证只用运营测试号，不得对真实用户的数据做创建或删除。
- 迁移不随部署自动执行。迁移文件提交后，由用户指示在 `peiqian` 上用 `cargo run --bin run_migrations` 执行，实现者不要自行对生产执行。

## 7.1 向后兼容（硬性要求，小程序已上线）

线上有各个版本的小程序在运行，本次改动对任何旧版本都必须无感：

- 迁移**只新增一张表和索引**，不 `ALTER`、不 `DROP`、不修改任何现有表、列、约束或触发器。
- 不修改任何现有接口的路径、请求体、响应体、状态码和错误文案；`POST /api/v1/payment/events` 与四个 Admin 接口均为新增。
- `X-Payment-Attempt-Id` 请求头可选，缺失或非法时服务端自行生成，不得拒绝请求。
- 旧版小程序不上报事件，服务端事件仍正常写入；两端互不依赖。
- 新版小程序遇到旧后端（上报接口 404/5xx）必须静默忽略，不影响购买流程。
- `requestDigitalPayment`、`waitForPaidOrder` 新增参数均为可选，不传时行为与现在逐字节一致。
- 事件表写入失败、表不存在、超时，一律只打 warn 日志，不得让下单、支付、结算、退款路径返回错误或变慢（recorder 调用不得阻塞主流程的关键路径，可 `tokio::spawn` 异步写入）。
- 发布顺序：先执行迁移，再部署后端，最后发小程序。

## 8. 发布顺序

1. 合并后端迁移与代码；用户确认后在 `peiqian` 执行迁移，再部署后端（本地后端已重启 / 生产未部署 / 生产已部署并重启，三种状态需分开汇报）。
2. 确认 `POST /api/v1/payment/events` 在线上可用后，再发小程序新版本（`bun run mp:release`）。旧版本小程序不上报，服务端事件仍会写入，两边互不依赖。
3. 小程序新版本上线后，观察 7 天，用第 5.5 的接口做第一次漏斗分析。

## 9. 范围外（明确不做）

- 修改任何价格、档位、购买页布局或文案；
- 把 `pending` 订单批量关闭为 `failed`；
- 给未成交用户发通知召回；
- Web 后台的漏斗页面；
- 告警通道（企业微信、飞书、短信）。本期只提供 `stuck-orders` 接口，是否接告警及接到哪里，等看过第一批数据再定；
- 修改 `prepay_id` 落库逻辑。

## 10. 开放问题（实现前需要用户确认）

1. 事件保留期 180 天：用户已确认。
2. Admin 只读接口默认不记审计日志。

## 11. 审查清单（Review 时逐条核对）

1. 埋点失败不影响主流程：recorder 与客户端上报都有"吞错"测试。
2. `unified_order` 失败时，原错误类型与返回给客户端的状态码、文案与改动前一致。
3. `requestDigitalPayment` 不传 `attempt` 时行为与改动前完全一致，且取消判定规则没有被改写。
4. 客户端上报接口没有越权：不能给别人的 `order_no` 写事件，不能写服务端事件。
5. `detail` 白名单生效，没有 openid、token、手机号进入表。
6. 迁移时间戳大于 `20261014163000`，文件名格式正确，没有被自动对生产执行。
7. 小程序没有往 `check-import-boundaries.mjs` 的 ALLOWLIST 添加例外；没有手改 `manifest.json` 的 `versionName`。
8. 三类商品（会员、比赛 ID 解锁、回流订阅）都有覆盖，而不是只做了会员。
9. 测试没有污染生产库：写库测试已回滚，或没有写库测试。
10. 7.1 向后兼容清单逐条满足，尤其是迁移只新增表、recorder 不阻塞主流程。
