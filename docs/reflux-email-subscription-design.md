# 回流邮件订阅设计

## 背景

微信小程序当前只能使用一次性订阅消息，不适合“持续、及时”的回流票提醒。第一版改为邮件提醒：用户购买回流提醒订阅后，`football_insight_service_backend_rs` 每分钟读取 `ticket-monitor-axum` 已暴露的回流接口，按用户有效权益聚合新增回流，并通过飞书公共邮箱 SMTP 发送邮件。

第二版预留 Android App 推送，不把订阅权益、通知任务和发送通道写死为 email-only。

## 产品规则

- 入口在小程序“余票看板”当前比赛页。
- `开始监控` 和 `订阅提醒` 是两个独立功能。
- 用户可以不购买提醒订阅，只使用已有监控会员能力。
- 用户也可以只购买提醒订阅，不购买监控会员。
- 第一版支持三种套餐：
  - 单场订阅：绑定当前队伍和当前比赛。
  - 赛季订阅：绑定当前队伍和当前赛季，第一版只通知该队伍当前正在监控的比赛。
  - 永久订阅：绑定当前队伍，不设过期；第一版先不上架。
- 套餐价格和启用状态由数据库配置，前端不写死价格。
- 套餐配置支持队伍专属覆盖全局配置：先查 `team_code`，没有则回退 `global`。
- 默认 seed：
  - 单场订阅 5 元，启用。
  - 赛季订阅 50 元，启用，有效期到 `2026-12-31 23:59:59 +08:00`。
  - 永久订阅暂不上架。
- 购买弹层允许填写或修改邮箱。
- 支付成功后开通权益，并发送订阅欢迎邮件。
- 回流邮件本身包含完整回流信息，不需要跳转链接。

## 数据模型

### `f_i_reflux_subscription_plans`

保存后端可配置套餐。

关键字段：

- `id`
- `code`: `single_match` / `season_2026` / `lifetime`
- `scope`: `single_match` / `season` / `lifetime`
- `team_code`: `global` / `chengdu` / `yunnanyukun`
- `season`: 例如 `2026`
- `title`
- `description`
- `price_cents`
- `enabled`
- `sort_order`
- `expires_at`
- `created_at`
- `updated_at`

### `f_i_user_notification_targets`

保存用户通知目标。第一版只写 `email`，第二版扩展 `android_push`。

关键字段：

- `id`
- `user_id`
- `channel`: `email` / `android_push`
- `target`: 邮箱地址或设备 token
- `is_active`
- `created_at`
- `updated_at`

### `f_i_user_reflux_subscriptions`

保存支付成功后的订阅权益。

关键字段：

- `id`
- `user_id`
- `plan_code`
- `scope`
- `team_code`
- `season`
- `match_id`
- `order_no`
- `starts_at`
- `expires_at`
- `status`: `active` / `expired` / `cancelled`
- `created_at`
- `updated_at`

### `f_i_reflux_notification_cursors`

保存每个队伍和比赛的处理游标，避免重启漏发。

关键字段：

- `team_code`
- `match_id`
- `last_processed_at`
- `created_at`
- `updated_at`

### `f_i_reflux_notification_jobs`

保存待发送和已发送的聚合邮件任务。

关键字段：

- `id`
- `user_id`
- `target_id`
- `team_code`
- `match_id`
- `subject`
- `body_html`
- `payload_json`
- `status`: `pending` / `sent` / `failed`
- `attempts`
- `next_attempt_at`
- `last_error`
- `sent_at`
- `created_at`
- `updated_at`

## API

订阅接口放在 `/api/v1/ticket-watch/reflux-subscriptions/...`。

### `GET /plans?team_code=...&match_id=...`

返回当前队伍可购买套餐，以及当前用户订阅状态。需要登录。

### `GET /status?team_code=...&match_id=...`

返回当前用户在该队伍/比赛下是否有有效提醒权益、当前邮箱、权益摘要。需要登录。

### `POST /order`

创建回流提醒订阅订单。需要登录。

请求包含：

- `plan_code`
- `team_code`
- `match_id`
- `email`

后端校验：

- 邮箱格式合法。
- 当前队伍有当前监控比赛。
- 单场订阅只能绑定当前正在监控或即将开售的比赛。
- 套餐启用且价格来自数据库配置。

响应复用微信支付参数结构：

- `order_no`
- `params`

支付状态轮询复用现有 `GET /api/v1/payment/order/{order_no}`。

## 支付结算

现有 `f_i_payment_orders` 继续作为订单主表。

回流订阅订单的 `product_type` 使用：

- `reflux_subscription:{plan_code}:{team_code}:{match_id}`

支付回调统一进入现有 `/api/v1/payment/wx-notify`，根据 `product_type` 分发结算：

- `membership:*` 走现有会员结算。
- `reflux_subscription:*` 走回流订阅结算。

结算成功后：

1. 标记订单 paid。
2. upsert 用户 email 通知目标。
3. 创建或延长订阅权益。
4. 创建欢迎邮件任务。

重复购买规则：

- 允许从单场升级到赛季或永久。
- 允许重复购买赛季并延长有效期。
- 不做差价，按所选套餐原价支付。
- 发送通知时只要存在任一有效权益即可。

## 每分钟轮询和聚合

`football_insight_service_backend_rs` 启动一个后台任务：

- 每 1 分钟执行一次。
- 读取当前成都蓉城比赛和当前云南玉昆比赛。
- 如果无当前比赛或缺少 `sale_start_at`，跳过该队伍。
- 起算时间必须使用 `sale_start_at + 10 minutes`。
- 查询起点为 `max(sale_start_at + 10 minutes, cursor.last_processed_at)`。
- 调用 ticket-monitor 已暴露接口读取新增回流。
- 按 `team_code + match_id + user_id` 聚合成一封邮件任务。
- 邮件任务创建成功后推进游标。
- 同一分钟内同一用户同一比赛最多一封邮件。

发送失败处理：

- pending 任务每分钟尝试发送。
- 最多重试 3 次。
- 每次失败后 `next_attempt_at = now + 1 minute`。
- 仍失败标记 `failed` 并记录错误。

## 邮件

通过飞书公共邮箱 SMTP 发送，环境变量：

- `FI_SMTP_HOST`
- `FI_SMTP_PORT`
- `FI_SMTP_USERNAME`
- `FI_SMTP_PASSWORD`
- `FI_SMTP_FROM`
- `FI_SMTP_FROM_NAME`

欢迎邮件标题示例：

- `[回流提醒] 订阅已开通`

回流提醒标题示例：

- `[回流提醒] 成都蓉城 vs 上海海港 出现余票回流`

回流邮件内容包含：

- 队伍和比赛。
- 开赛时间。
- 本轮聚合时间。
- 区域、票价、票数、最新发现时间。
- 订阅类型。
- “数据来自实时监控，余票可能随时变化”的说明。

## 前端

余票看板当前比赛卡片新增两个并排按钮：

- `开始监控`
- `订阅提醒`

订阅弹层展示：

- 当前队伍/当前比赛。
- 邮箱输入框，已有邮箱时可修改。
- 后端返回的套餐列表。
- 购买按钮。
- 支付后轮询订单状态，直到后端回调完成并刷新订阅状态。

“我的”页第一版支持展示/编辑邮箱，不发验证邮件。

## 第二版 App 推送预留

第二版增加：

- `android_push` 通知目标。
- App 登录后上报设备 token。
- 通知任务按用户偏好或多通道发送。

第一版的权益表、通知目标表、通知任务表都保留 `channel` 扩展点，避免后续重写订阅和支付流程。
