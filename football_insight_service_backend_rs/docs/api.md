# Football Insight Admin API

管理端 API 与 C 端 API 完全分离，统一位于 `/api/v1/admin/**`。除登录外，请求必须携带：

```http
Authorization: Bearer <admin_access_token>
```

C 端 JWT、已过期 Admin JWT、已撤销 Admin 会话均返回 `401 Unauthorized`。

## 认证

### `POST /api/v1/admin/auth/login`

请求：

```json
{"username":"owner","password":"your-password"}
```

响应包含 `access_token`、`expires_at` 和独立管理员身份 `admin`。

### `GET /api/v1/admin/auth/me`

返回当前管理员的 `id`、`username`、`display_name` 和 `role`。

### `POST /api/v1/admin/auth/logout`

撤销当前服务端会话，成功返回 `204 No Content`。

## 用户

### `GET /api/v1/admin/users`

查询参数：

- `query`：账号或昵称模糊搜索
- `status`：`active` 或 `disabled`
- `membership_tier`：`V1` 至 `V9`
- `page`、`page_size`：分页，单页最多 100

### `POST /api/v1/admin/users`

创建 C 端用户。请求包含 `account_identifier`、`display_name`、`password`、`membership_tier`，可选 `avatar_url` 与 `membership_expires_at`。

### `GET /api/v1/admin/users/{id}`

返回用户账号、昵称、邀请关系、状态、微信绑定和会员信息，并包含：

- `referrals`：该用户邀请的下级用户
- `activity`：最近登录、活跃时间与页面
- `orders`：最近 50 笔支付订单
- `subscriptions`：最近 50 条回流订阅
- `devices`：推送设备；令牌只返回首尾掩码

### `PATCH /api/v1/admin/users/{id}`

更新账号、昵称或头像等基本资料。

### `POST /api/v1/admin/users/{id}/disable`

请求 `{"reason":"账号风险"}`。软禁用用户并撤销其 C 端会话，同时写入审计日志。

### `POST /api/v1/admin/users/{id}/restore`

请求 `{"reason":"人工复核通过"}`。恢复用户并写入审计日志。

### `POST /api/v1/admin/users/{id}/membership`

请求示例：

```json
{
  "membership_tier": "V8",
  "expiration_mode": "never",
  "membership_expires_at": null,
  "reason": "线下年度会员"
}
```

`expiration_mode` 支持 `preserve`、`never`、`specific`；`specific` 必须提供 RFC3339 格式的 `membership_expires_at`。调整与审计记录在同一事务中提交。

## 审计

### `GET /api/v1/admin/audit-logs`

支持 `page`、`page_size`，按时间倒序返回管理员、动作、目标、原因和创建时间。


## 支付漏斗只读查询（Admin，2026-10-16）

四个新增接口仅支持 GET，使用现有 Admin JWT（`Authorization: Bearer <token>`）及有效、未撤销的后台会话，并检查当前数据库角色的 `ViewAuditLogs` 权限（owner/admin）。不写审计日志，所有响应含 `Cache-Control: no-store`。未登录/失效 JWT 返回 401，无权限返回 403，查询参数错误返回 400，查询存储不可用返回 500（仅影响这些只读查询）。不改订单状态。

### `GET /api/v1/admin/payment/funnel?from=&to=&scenario=`

`from`、`to` 必填 RFC3339 时间戳，采用 `[from,to)` 区间；区间必须为正且不超过 180 天。`scenario` 可选，仅允许 `membership`、`match_id_unlock`、`reflux_subscription`。

响应：`{ "timezone": "UTC", "stages": [...], "terminal_stages": [...] }`。

- `stages` 按 UTC 自然日 `day`、`scenario`、`product_ref`、`event` 分组，返回 `attempt_count`（去重 attempt）、`user_count`（去重用户）、`previous_event`、`previous_attempt_count`、`converted_attempt_count`、`conversion_rate`。
- 相邻主链为 `purchase_tapped → order_create_requested → order_created → pay_params_issued → pay_sheet_invoked → pay_success_client → delivered`。转化率是同日、同场景和同商品内，同时经过前后阶段的 attempt 数除以前阶段 attempt 数；前阶段不存在/为零或事件不属于主链时为 null。旧客户端缺少客户端事件时不推算其转化率。每个已观测到的 UTC 日期、场景、商品组合均补齐七个主链阶段；缺失阶段的 attempt 数与人数为 0，有前阶段但未转入下一阶段时转化率明确为 0。前阶段为零时转化率为 null；不同日期、场景或商品间不混算。分支事件仍返回原始人数和 attempt 数。
- `terminal_stages` 按区间内每个 attempt 的事件序列派生终止阶段，按 attempt 在区间内首个事件的 UTC 日期、场景、商品和阶段汇总；同组用户去重。阶段规则与 stuck-orders 共用领域纯函数；成功发货优先于迟到的客户端失败，退款优先于发货，不写库状态。

示例阶段：`{"day":"2026-10-16","scenario":"membership","product_ref":"V7","event":"order_created","attempt_count":2,"user_count":1,"previous_event":"order_create_requested","previous_attempt_count":4,"converted_attempt_count":2,"conversion_rate":0.5}`。

### `GET /api/v1/admin/payment/errors?from=&to=`

时间区间校验同上（也接受可选 `scenario`）。响应 `{"items":[...]}`，按 `event + error_code` 聚合，出现次数降序取 Top 20。每项返回 `event`、`error_code`、最近一次的 `error_msg`、`count`、`total_error_count`、`share`、`last_occurred_at`。分母包含区间内所有错误组而非仅 Top 20。错误包括有错误码的事件，以及会话/下单/签发参数/微信支付失败、取消、确认超时事件；缺失错误码保留 null。

### `GET /api/v1/admin/payment/orders/{order_no}/events`

响应 `{"order_no":"...","events":[...]}`；事件含 `id`、`attempt_id`、`order_no`、`user_id`、`scenario`、`product_ref`、`event`、`source`、`error_code`、`error_msg`、`detail`、`occurred_at`。包含直接关联订单的事件及同一用户、同一 attempt 的下单前无 order_no 事件，客户端/服务端合并，按 `occurred_at,id` 升序。不存在事件返回空数组，订单号最多 128 字节。

### `GET /api/v1/admin/payment/stuck-orders?older_than_minutes=30`

阈值默认 30 分钟，允许 1–259200。响应 `{"items":[...]}`，列出创建时间超过阈值且仍 pending 的订单，按创建时间和订单号升序，返回 `order_no`、`user_id`、`product_type`、`created_at`、`last_event`、`last_event_at`、`stage`。关联 attempt 的下单前事件同样纳入归因。

`stage` 取值：`agreement_not_accepted`、`session_prepare_failed`、`order_failed`、`params_without_pay_sheet`、`pay_sheet_without_callback`、`cancelled`、`payment_failed`、`paid_awaiting_confirmation`、`confirmation_timeout`、`success`、`refunded`、`unknown`。上线前无埋点的老订单明确返回 `unknown`，最后事件字段为 null。

事件保留期为 180 天；本期仅记录保留约定，清理脚本与定时清理不在本期范围。新增迁移由用户指示在 peiqian 执行，未迁移时查询会返回 500，不影响购买/支付路径。
## Payment attempt ingestion (additive)

`POST /api/v1/payment/events` requires the existing user Bearer JWT. Invalid or missing authentication returns 401; all authenticated requests return an empty 204, including invalid JSON, oversized payloads, rejected ownership, rate limits and failed persistence.

```json
{"attempt_id":"a UUID","order_no":null,"scenario":"membership","product_ref":"V7","events":[{"event":"purchase_tapped","detail":{"mini_version":"1.0.73","elapsed_ms":0}}]}
```

The three scenarios are `membership`, `match_id_unlock`, `reflux_subscription`. The client whitelist is `purchase_tapped`, `agreement_prompted`, `session_prepare_failed`, `order_create_requested`, `order_create_failed`, `pay_sheet_invoked`, `pay_cancelled`, `pay_failed`, `pay_success_client`, `confirm_timeout`. Server events cannot be submitted by clients. A nonempty order number must belong to the authenticated user; otherwise the entire batch is discarded without disclosing existence.

Limits: 8 KiB body, 10 events per batch, 30 accepted client events per user per minute in this process. Messages are limited to 300 Unicode characters; detail is at most 1 KiB and accepts scalar `mini_version`, `wx_lib_version`, `platform`, `system`, `brand`, `model`, `network_type`, numeric `elapsed_ms`, `http_status` only. Unknown and nested detail keys are removed. No openid, phone, tokens or request bodies are collected as metadata.

All three existing order endpoints optionally read `X-Payment-Attempt-Id`; missing, nil or invalid values generate a UUID. Existing request/response bodies and errors are unchanged. Background recorder tasks have bounded capacity, a 250 ms deadline and an independent two-connection pool (100 ms acquisition timeout); errors and overload only warn. Settlement resolves the attempt off the critical path from the earliest server event for that order (or a surviving client event when the initial server writes were lost); historical orders without events use their immutable order UUID.

Event timestamps reflect server receipt/creation. `elapsed_ms` preserves client-relative timing; queued client precursors cannot regress a later payment stage, and delivery/refund evidence takes priority over late client results. Retention policy is 180 days; no cleanup job is introduced. Apply migration explicitly, deploy backend, then release the mini program. None of these production actions are automatic.
## Membership catalog P1: optional pass metadata

Existing membership GET/POST paths, request bodies and response fields remain unchanged. Annual options still come from legacy `membership_product` system configuration; P1 does not introduce table mode or change `catalog_source`.

New clients send `X-Client-Capabilities: catalog-v2` on `GET /api/v1/payment/membership-product` and `POST /api/v1/payment/membership/order`. Comma/space-separated capabilities are matched exactly. The optional `X-Payment-Attempt-Id` remains independent and all server events retain the existing nonblocking recorder.

Eligible, enabled catalog offers can append optional per-option fields: `expiry_rule`, `duration_label`, `group_key`, `is_pass`. When absent they are omitted, so legacy annual JSON fields and their serialization are unchanged. Passes return `membership_duration_months: null`, `expiry_rule: "end_of_next_day"`, `duration_label: "购买次日 23:59（北京时间）"`, `group_key: "pass"`, `is_pass: true`. Prices remain integer cents. A table offer using fixed pricing does not inherit legacy annual upgrade credit.

Offers with `requires_capability` are hidden unless declared. An attempt to buy the gated offer without its capability returns 400 `请更新小程序后购买`. Passes require effective current tier at most V3; listing and purchase revalidate eligibility, and parameter issuance repeats validation. Ineligible new offers return 400 `当前会员等级不可购买该商品`. Existing annual errors are retained. Selection stays keyed by target_tier; duplicate eligible offers are sorted by sort_order, price, key, and the selected immutable identity/key/price is rechecked before checkout.

Membership virtual orders now store `grant_snapshot` with kind, grant_spec, catalog_product_id. Fulfillment reads that snapshot without looking up the mutable catalog; NULL snapshots retain the historical settlement path. A previously earned permanent V4+ entitlement acquired while a pass order is pending is preserved rather than converted into an expiring entitlement. Expired membership still follows the existing V3 fallback. Unknown qualified membership tiers now error; historical unqualified `membership` remains the documented V9 legacy product.

The migration only adds catalog/snapshot fields, backfills new catalog metadata and widens the key CHECK to V3–V9; the V3 account restriction is retained. `kind` defaults to membership for existing Admin/SQL writers, which still insert the original columns. Legacy rows reconstruct their monthly grant from the current legacy tier/duration columns at checkout, so existing Admin edits remain effective in P1. Explicit new offer metadata uses grant_spec. No Web Admin form, generic catalog endpoint, annual table switch, shadow comparison or nonmembership pricing change is included in P1.
