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
