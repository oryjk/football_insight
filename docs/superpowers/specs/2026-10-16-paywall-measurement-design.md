# 付费引导曝光与点击度量设计

日期：2026-10-16
状态：待用户确认后交付实现
范围：`football_insight_service_backend_rs`（activity 模块、admin 模块）、`football_insight_mini`（洞察页、余票监控页、我的页）
与进行中任务的关系：**不改动 `payment` 模块、不改动 `membership-purchase` 页面**，可与"商品目录数据驱动化"并行实施，不会产生文件冲突
不涉及：任何文案、价格、交互改版；入口转化的改版设计（等本期数据和日卡定位确定后另做）

## 1. 背景与目标

已掌握的漏斗数据：

- 会员购买页访问用户 367 人，创建过订单的用户 49 人（约 13%），付费用户 30 人。
- 付费集中在主场比赛前 1–3 天，且多发生在注册当天。

**缺口**：购买页之前的环节没有数据。现有付费引导点（均已核实）：

| 引导点 | 位置 | 触发条件 |
|---|---|---|
| 最近回流速览锁定面板 | `pages/ticket-watch/components/TicketRecentRefluxLock.vue`，按钮「升级查看」 | 用户等级 < V6，且存在当前比赛 |
| 比赛 ID 解锁弹层的「升级」 | `pages/ticket-watch/components/TicketMatchIdSheet.vue` | 弹层状态为 `locked` |
| 洞察页回流看板入口 | `pages/insights/components/InsightsTicketWatchEntry.vue` | 进入余票监控的唯一入口 |
| 我的页的会员入口 | `pages/user/index.vue`（跳购买页处） | 用户点击 |

现在**不知道**：有多少用户看到了锁定面板、有多少人点了「升级查看」、各入口各自带来了多少购买页访问。没有这些数据，后续任何入口改版都只能凭感觉。

**目标**：对每个付费引导点，能回答"多少用户看到过、多少用户点击了、点击者当时的会员等级"，并能按天、按入口汇总。

**成功标准**（上线 7 天后验收）：能产出一张表：入口 × 日期 → 曝光用户数、点击用户数、点击率；并能与购买页访问用户数、`f_i_payment_attempt_events` 的下单与发货事件按用户和时间对照。

## 2. 设计原则

- **只加观测，不改行为**：不改任何界面文案、布局、跳转目标。埋点失败绝不能影响页面渲染、跳转和购买流程。
- **与支付模块解耦**：本设计不使用 `f_i_payment_attempt_events`，也不引用 `payment` 模块的任何代码，避免与正在进行的商品目录重构冲突。
- **不记个人敏感信息**：只记用户 ID、入口、动作、会员等级、时间。
- **向后兼容**：新增表和接口；旧版小程序不上报，不受影响。

## 3. 数据模型

新增迁移 `YYYYMMDDHHMMSS_add_paywall_events.sql`，时间戳必须大于目前仓库里最新的迁移（以 `ls migrations | tail -1` 为准，并大于商品目录重构任务的迁移，如已存在）。

```sql
CREATE TABLE f_i_paywall_events (
    id          BIGSERIAL PRIMARY KEY,
    user_id     UUID        NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    surface     TEXT        NOT NULL,   -- 见 4.1，应用层白名单校验
    action      TEXT        NOT NULL CHECK (action IN ('impression','tap')),
    membership_tier TEXT,               -- 事件发生时用户的有效等级，如 'V3'
    detail      JSONB       NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_paywall_events_time ON f_i_paywall_events (occurred_at DESC, surface, action);
CREATE INDEX idx_paywall_events_user ON f_i_paywall_events (user_id, occurred_at DESC);
```

`detail` 允许的键（应用层白名单，其余丢弃）：`mini_version`、`platform`（ios/android/devtools）、`match_id`（有当前比赛时）。单条序列化后不超过 512 字节。保留 180 天，清理脚本不在本期范围。

## 4. 事件定义

### 4.1 入口（surface）白名单

| surface | 对应位置 | impression 触发 | tap 触发 |
|---|---|---|---|
| `ticket_watch_recent_reflux_lock` | 余票监控页最近回流速览锁定面板 | 面板以锁定态进入可见（页面渲染后首次） | 点击「升级查看」 |
| `ticket_watch_match_id_sheet` | 比赛 ID 解锁弹层 | 弹层以 `locked` 状态打开 | 点击弹层内的「升级」 |
| `insights_ticket_watch_entry` | 洞察页回流看板入口 | 入口卡片渲染且用户已登录且等级 < V6 | 点击入口 |
| `user_membership_entry` | 我的页会员入口 | 入口渲染 | 点击入口（跳购买页处） |

### 4.2 去重规则（客户端）

- 同一入口在同一次页面显示（`onShow` 到 `onHide`）内只记 1 次 `impression`；再次 `onShow` 才可再记。
- 同一入口 `tap` 不去重，但同一入口 2 秒内的重复点击合并为 1 次。
- 未登录用户不上报（接口也要求 JWT）。

## 5. 后端设计

遵循 `domain / ports / application / adapters`，放入现有 `activity` 模块（`src/activity/`），业务逻辑放 `application`。

### 5.1 上报接口

`POST /api/v1/activity/paywall-events`（需要 JWT，复用 activity 模块现有的鉴权方式）。

```json
{ "events": [ { "surface": "ticket_watch_recent_reflux_lock", "action": "impression",
                "membership_tier": "V3", "detail": { "mini_version": "1.x.x", "platform": "android" } } ] }
```

- 每次最多 10 条，请求体上限 4 KB；白名单外的 `surface`、`action` 丢弃该条。
- `membership_tier` 只接受 `V1`–`V9` 的格式，非法则存 NULL；**以客户端上报为准，服务端不做二次查询**（这只是度量字段）。
- `detail` 按 3 节白名单过滤。
- 每用户每分钟最多 60 条，超出静默丢弃（进程内计数）。
- **永远返回 204，除非未登录返回 401**，不暴露任何校验细节。

### 5.2 写入不能影响主流程

沿用可观测化的做法：异步 `tokio::spawn` 写入，带超时与并发上限；写入失败只打 warn 日志。使用独立的小连接池，不占业务连接池。可复用可观测化已有的连接池装配模式，**但不要引用 `payment` 模块的类型**，在 activity 内部自行实现。

### 5.3 只读查询接口（Admin）

放 `src/admin` 模块，沿用现有 Admin JWT，仅 `GET`：

- `GET /api/v1/admin/paywall/funnel?from=&to=&surface=`：按 **北京时间自然日**（`Asia/Shanghai`）× `surface` 汇总，返回：曝光次数、曝光用户数、点击次数、点击用户数、用户点击率（点击用户数 / 曝光用户数）。
- `GET /api/v1/admin/paywall/by-tier?from=&to=`：按 `membership_tier` × `surface` 汇总曝光与点击用户数，用于判断是哪个等级的用户在点击。

聚合 SQL 放 `adapters/persistence`，`application` 只做参数校验与组装。时间范围最大 92 天，默认最近 7 天。注意日期口径用北京时间，不要沿用 UTC。

## 6. 小程序设计

遵守 `football_insight_mini/AGENTS.md`：组件层不得 import `api/**`（上报由**页面**发起，组件只通过 emit 通知）；HTTP 只走 `utils/request.ts`；不加 ALLOWLIST 例外；不改 `manifest.json` 的 `versionName`。

### 6.1 新增文件

- `src/api/paywall.ts`：`reportPaywallEvents(payload)`，`POST /activity/paywall-events`，`auth: true`，静默处理网络错误（参照 `src/api/payment.ts` 里 `reportPaymentEvents` 的 `silentNetworkError` 用法）。
- `src/composables/usePaywallTelemetry.ts`：
  - `reportPaywall(surface, action)`：fire-and-forget；内部做 4.2 的去重和 500ms 合并；**所有错误吞掉，不抛出，不弹 Toast**；未登录直接 return。
  - 采集 `membership_tier`（来自 appSession 中的当前用户）、`mini_version`、`platform`，取不到就省略。
  - 导出纯函数 `shouldReportImpression(state, now)` 供单测。
- `src/composables/usePaywallTelemetry.test.ts`：覆盖去重、合并、吞错、未登录跳过。

### 6.2 接入点（只加调用，不改渲染和跳转）

- `pages/ticket-watch/index.vue`：
  - 监听 `currentRecentRefluxPanelMode` 变为 `'locked'` 时记 `ticket_watch_recent_reflux_lock` impression；
  - 在已有的 `@upgrade="goToMembershipPurchase"` 处理中，按触发来源记对应 surface 的 tap（锁定面板 vs 比赛 ID 弹层），**再**调用原有跳转，原有逻辑一行不改；
  - 监听 `matchIdSheetState` 变为 `'locked'` 且弹层可见时记 `ticket_watch_match_id_sheet` impression。
- `pages/insights/index.vue`：入口卡片渲染时（已登录、等级 < V6）记 `insights_ticket_watch_entry` impression；`openTicketWatch` 被调用时记 tap，**原有早退逻辑（未登录、权益暂停）保持在前，只有真正跳转的路径才记 tap**。
- `pages/user/index.vue`：跳购买页的位置（约 327 行）前记 `user_membership_entry` tap；入口渲染记 impression。
- 不修改 `membership-purchase` 页面（正由其他任务修改）。

### 6.3 测试

- `bun test`：`usePaywallTelemetry.test.ts` 新增；现有 `pages/ticket-watch/effects.test.ts`、`pages/insights/sessionEffects.test.ts`、`pages/user/loginFlow.test.ts` 保持通过，证明页面行为不变。
- `bun run type-check`、`bun run check:boundaries`、`bun run verify:mp-components` 全部通过。

## 7. 测试与验证（后端）

- 单元测试：白名单外 `surface`/`action` 被丢弃；超长、非法 `membership_tier` 被处理；`detail` 过滤；限流；未登录 401；任何校验失败仍返回 204。
- 写入失败（连接池耗尽、超时）不影响响应。
- 聚合 SQL：北京时间日界（`23:59:59` 与 `00:00:00` 附近）分组正确；点击率分母为 0 时不报错。
- **只有生产这一个数据库**：写库测试必须在事务内回滚；SQL 必须用"生产库开事务 → 执行迁移与全部新增 SQL → 无条件 ROLLBACK → 确认回滚后表不存在"的方式真实验证一遍（参考可观测化任务遗留的做法：先 `SET LOCAL lock_timeout='2s'`，用一个已存在的用户 ID 写入模拟数据，脚本放会话 scratchpad 目录，不输出连接串）。不得在生产库留任何数据。
- 迁移不随部署自动执行，实现者不要自行对生产执行。

## 7.1 向后兼容

- 迁移只新增一张表和索引。
- 不修改任何现有接口的路径、请求体、响应体、状态码、文案。
- 旧版小程序不上报，服务端无影响；新版小程序遇到旧后端（上报接口 404/5xx）静默忽略。
- 小程序不改渲染、文案、跳转目标；埋点代码出错不得影响页面。

## 8. 发布顺序

1. 合并后端；用户确认后在 `peiqian` 执行迁移，再部署后端。
2. 发布小程序新版本。**与商品目录重构的小程序改动合并为同一次发版**，减少审核轮次。
3. 上线满 7 天后，用 Admin 接口出第一份入口转化报告。

## 9. 范围外

- 任何入口的文案、价格展示、布局改版（等本期数据与日卡定位确定后另做）；
- 给购买页传入口来源参数以做精确归因（需要改 `membership-purchase`，与进行中任务冲突，后续做；本期靠用户 ID 与时间邻近对照）；
- Web 后台页面；
- 告警。

## 10. 审查清单（Review 时逐条核对）

1. 没有引用 `payment` 模块任何代码，没有改动 `membership-purchase`。
2. 埋点失败不影响主流程：后端异步写入有超时与并发上限；客户端吞错测试存在。
3. 原有跳转与早退逻辑一行未改，`tap` 只在真正跳转的路径上记录。
4. 去重规则生效：一次页面显示内同一入口只记一次曝光。
5. 上报接口无越权风险：只能为当前 JWT 用户写入；白名单生效；永远 204。
6. 日期口径是北京时间；点击率分母为 0 时安全。
7. 组件层没有 import `api/**`；无 ALLOWLIST 新增；无手改 `versionName`。
8. 迁移只新增表和索引，时间戳足够大，未被自行对生产执行；SQL 已在回滚事务中真实验证；生产库无残留数据。
9. 四个入口都覆盖，且 surface 名称与第 4.1 节完全一致。
