# Football Insight Admin API

C端免费赛季总结新增接口见[赛季总结 API](season-recap-api.md)，与以下 Admin API 分离，不改变已有接口。

认证请求新增的可选来源元数据另见[用户来源追踪](../../docs/acquisition-source-tracking.md)；这不是Admin API，不改变已有认证响应或鉴权规则。

公开 ticket-watch 新接口另见 [最佳盯盘时段 API](best-watch-windows-api.md)，其可选用户鉴权与以下 Admin API 无关。

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

## 公众号「下一场」被动回复

`/football/wechat/webhook` 的路径、签名及明文/加密封装不变。文字命令去掉全部空白并转小写后，`下一场`、`下一场id`、数字 `2`/`3` 共用门控；`1` 仍为邀请码。

顺序为现有开关 → current-standard 比赛 ID → 公众号身份判定。关闭、无比赛或获取失败继续原缺省回复且不查身份；身份或获取记录写入出错 fail closed。有效等级达到 `MATCH_ID_UNLOCK_MINIMUM_TIER` 或已解锁本场可继续查询，过期等级沿用现有回落规则。没有这些权益、且公众号身份及关联账号都从未成功获取过 ID 的用户，可免费获取一次，无需先绑定小程序；用过后同一场或换场都不再免费。ID 解析失败仍可按等级授权，不能按解锁或首次获取授权。

身份端口按 active 账号公众号 openid 读取等级、到期时间及本场解锁；None 比赛 ID 使用 CASE 跳过解锁子查询。所有成功获取（包含会员和单场解锁）都通过事务写入终身获取记录，并标记该公众号身份关联的全部账号。唯一公众号键和账号行锁确保并发最多一次首次获取，取关重关或自助换绑不重置记录；无比赛、获取失败、身份失败或写入失败不发放免费 ID。每次命令记录 `next_match_id_reply decision=...`，新增 `first_acquisition` 判定，不记录 ID/openid 或原始上游错误。

迁移 `20261020160000_add_official_match_id_acquisitions.sql` 将全部存量账号补记为已获取，并将已有关注者（含取关）、已绑定公众号身份及历史邀请码身份写入 `f_i_official_match_id_acquisitions` 的 historical 记录。新账号默认未获取；这不是按比赛发放的单场解锁，也不授予小程序权益。生产应先应用迁移再部署后端；迁移含存量回填，不应作为日常启动 SQL 重复执行。

## C 端公众号自助绑定

### `POST /api/v1/auth/wechat-official-bind-code`

需要 C 端 `Authorization: Bearer <access_token>`，仅 active 用户可用，无请求体。已绑定用户也允许获取，用于换微信号。

成功 `200 OK`：

```json
{"code":"ABC234","expires_in_seconds":600}
```

所有响应（包括错误）均带 `Cache-Control: no-store`。错误为纯文本：未登录、无效 JWT 或非 active 用户返回 `401`（`not logged in`）；同账号一小时第六次发码返回 `429`（`获取过于频繁，请稍后再试`）；存储或安全随机源不可用返回 `503`（`暂时无法获取绑定码，请稍后再试`）。

绑定码为六位无易混字符的大写字母/数字，只存 SHA-256 哈希，十分钟有效且一次性；重新获取使该账号所有仍有效的旧码失效。

### `GET /api/v1/auth/me` 新增可选字段

现有响应字段保持原样，新增 `official_wechat_bound?: boolean`，含义为公众号 openid 是否非 NULL。它与现有的小程序微信绑定 `has_wechat_binding` 无关；状态读取异常时省略该字段，原接口成功/错误映射不变。其他登录响应不增加此字段。

`official_wechat_binding_required?: boolean` 表示当前账号是否需要先绑定公众号：未绑定，且当前会员不是 `membership_origin=purchase` 的有效购买权益时为 true（V1–V9 一视同仁）；已绑定、未到期购买会员或购买来源的长期会员为 false。推荐升级即使有到期时间也不属于购买豁免，付费到期后需绑定。小程序据此弹出绑定窗口，并区分“未绑定”与原有“已取关暂停”；原有功能等级限制继续生效。此字段仅在 `/auth/me` 补充，读取异常省略，登录响应仍需通过 `/auth/me` 确认。

### 公众号绑定指令

仅在现有 webhook 签名/加密校验成功后处理文本：去空白并忽略大小写，`绑定ABC234`、`绑定 abc234` 均可。缺码给格式提示；错误、过期、已用或非 active 一律回复同一失败文案。按公众号身份限制十分钟内五次失败（包含在途预约），第六次停止校验。码消耗与账号绑定在同一事务完成；相同 openid 仍消耗码并回复已绑定，换微信号可覆盖，同一 openid 可关联多个账号。

每条绑定指令仅记录 `wechat_bind decision=success|already_bound|invalid|rate_limited|missing_code|error`，不记录码、openid、user_id。所有定稿回复及下一场未识别提示见设计第 4.6–4.7 节，保留产品介绍。绑定不授予会员或单场解锁；下一场仍使用既有 fail-closed 门控。绑定后关注状态参与会员权益计算，取关会暂停权益；无主动消息推送。
