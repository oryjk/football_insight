# 当前比赛换座撮合设计

## 背景

去现场看球的成都蓉城球迷可能对自己抢到的座位不满意，希望和其他现场球迷换位置。换座是比赛日现场工具，不应埋在洞察内容里；小程序第一版将新增独立换座页面，并加入底部 tabbar，作为和首页、榜单、洞察、赛程、我的并列的一等入口。

本功能只服务成都蓉城当前比赛，不做跨比赛、历史比赛或未来比赛撮合。当前比赛来源复用现有余票看板接口对应的后端能力，分区选项复用 `/api/v1/ticket-watch/regions` 的成都蓉城分区数据。

## 目标

- 用户可以发布自己的当前座位和希望换到的位置。
- 系统根据双方当前分区和目标分区做双向撮合。
- 双向满足后，双方可以看到联系方式并线下沟通。
- 双方线下确认后，在小程序里各自确认同一个对象，形成正式匹配。
- 正式匹配成功后，双方本场请求关闭，不再参与其他候选。
- 正式匹配成功后仍可撤销，但必须提交说明和双方达成一致的截图，供未来信用体系使用。

## 非目标

- 不对接票务系统，不承诺真实票权转移。
- 不做跨比赛或非当前比赛撮合。
- 不做泛场馆座位模型，第一版只面向成都蓉城。
- 不公开展示会员等级，会员等级与换座撮合无关。
- 不在第一版展示撤销截图给普通小程序用户。
- 不把换座入口放在洞察页内部；换座作为独立页面和 tabbar 入口。

## 用户与权限

- 未登录用户可以看到入口说明和脱敏的公共换座意向。
- 发布、更新、撤销、查看匹配联系方式、确认换座都必须登录。
- 同一用户同一场比赛只保留一条有效换座请求；再次发布视为更新。
- 联系方式至少填写一种：微信号或手机号。手机号如果填写，需要校验 11 位大陆手机号。

## 座位输入

用户发布请求时填写：

- 当前分区：必选，来自 `/api/v1/ticket-watch/regions`。
- 当前排：必填。
- 当前号：必填。
- 目标位置列表：至少一项。
- 每个目标位置包含：
  - 目标分区：必选，来自 `/api/v1/ticket-watch/regions`。
  - 目标排：可选。
  - 目标号：可选。

匹配只按分区判断，目标排/号只作为偏好展示，不参与双向匹配计算。

## 匹配状态

换座请求状态：

- `active`：发布中，参与匹配。
- `matched`：双方已确认，正式匹配成功。
- `cancelled`：用户撤销。
- `expired`：请求所属比赛不再是当前比赛。第一版可在查询时过滤，不需要批量写库。

候选展示状态：

- `communicable`：双方分区双向匹配，可见联系方式，可线下沟通。
- `waiting_peer_confirmation`：我已确认对方，对方还未确认我。
- `peer_confirmed_me`：对方已确认我，我可确认后完成正式匹配。
- `matched`：双方确认完成。
- `display_only`：不是双向匹配，只展示脱敏信息，不展示联系方式。

确认规则：

- 双向匹配后即可看到对方联系方式。
- 用户可以确认一个双向候选作为当前选择。
- 未双方确认前，用户可以改选其他候选；改选会覆盖之前的确认对象。
- 如果 A 确认 B，且 B 也确认 A，则 A/B 都进入 `matched`。
- 进入 `matched` 后，双方本场请求不再参与其他匹配。

撤销规则：

- 未正式匹配的 active 请求可以直接撤销。
- 正式匹配成功后也可以撤销，但必须提交撤销说明和双方达成一致的截图。
- 成功匹配后的撤销会写入撤销记录，供未来信用体系扣分或审核使用。
- 撤销后当前撮合列表不再返回双方联系方式。

## 后端架构

Rust 后端新增独立 `seat_swap` 模块，遵循现有六边形架构：

```text
src/seat_swap/
  domain/
  ports/
  application/
  adapters/
    persistence/
    web/
```

模块职责：

- `domain`：换座请求、目标座位、候选状态、联系方式可见性、确认状态。
- `ports`：换座 repository、当前比赛/分区读取端口、撤销证据存储端口。
- `application`：读取当前换座池、发布/更新我的请求、撤销、确认、成功后撤销。
- `adapters/persistence`：SQLx 实现。
- `adapters/web`：Axum DTO、handler、routes 和鉴权。

当前比赛判断复用 ticket watch 当前比赛能力。不是当前成都蓉城比赛时，发布、确认、撤销都拒绝。

## 数据库设计

新增表：

### `f_i_seat_swap_requests`

保存用户在某场比赛的换座请求。

核心字段：

- `id uuid primary key`
- `match_id bigint not null`
- `user_id uuid not null`
- `current_region_key text not null`
- `current_region_name text not null`
- `current_row text not null`
- `current_seat_no text not null`
- `wechat_id text null`
- `phone_number text null`
- `status text not null`
- `matched_request_id uuid null`
- `created_at timestamptz not null`
- `updated_at timestamptz not null`

约束：

- 同一 `match_id + user_id` 只允许一条未取消的当前请求；实现上可用查询更新或部分唯一索引约束 active/matched 状态。
- `wechat_id` 和 `phone_number` 至少一个非空。

### `f_i_seat_swap_desired_seats`

保存目标位置列表。

核心字段：

- `id uuid primary key`
- `request_id uuid not null`
- `region_key text not null`
- `region_name text not null`
- `desired_row text null`
- `desired_seat_no text null`
- `sort_order int not null`

### `f_i_seat_swap_confirmations`

保存确认对象。

核心字段：

- `id uuid primary key`
- `match_id bigint not null`
- `request_id uuid not null`
- `target_request_id uuid not null`
- `confirmed_by_user_id uuid not null`
- `created_at timestamptz not null`

约束：

- 同一 `match_id + confirmed_by_user_id` 只能有一个当前确认对象。
- 不能确认非双向匹配对象。

### `f_i_seat_swap_cancellations`

保存正式匹配成功后的撤销证据。

核心字段：

- `id uuid primary key`
- `match_id bigint not null`
- `request_id uuid not null`
- `target_request_id uuid not null`
- `cancelled_by_user_id uuid not null`
- `reason text not null`
- `evidence_object_key text not null`
- `evidence_url text not null`
- `created_at timestamptz not null`

## MinIO 证据存储

Rust 后端复用现有 MinIO 环境变量命名：

- `FI_MINIO_ENDPOINT`
- `FI_MINIO_ACCESS_KEY`
- `FI_MINIO_SECRET_KEY`
- `FI_MINIO_BUCKET`
- `FI_MINIO_REGION`
- `FI_MINIO_PREFIX`
- `FI_MINIO_PUBLIC_BASE_URL`

撤销截图使用单独 object prefix，例如：

```text
seat-swap/cancel-evidence/{match_id}/{request_id}/{uuid}.{ext}
```

DB 只保存 object key 和 public URL。第一版不在小程序公开展示撤销截图。

## API 设计

统一前缀：`/api/v1/seat-swap`

### `GET /api/v1/seat-swap/current`

返回当前比赛、我的请求、候选列表、是否可发布。

- 未登录可调用。
- 未登录不返回联系方式和我的确认状态。
- 没有当前成都蓉城比赛时，返回不可发布状态。

### `PUT /api/v1/seat-swap/my-request`

登录必需。创建或更新我的本场请求。

入参包含：

- 当前分区 key/name
- 当前排、号
- 目标位置列表
- 微信号和手机号

校验：

- 当前比赛存在且是当前成都蓉城比赛。
- 分区必须来自当前成都蓉城分区列表。
- 当前排/号必填。
- 目标分区至少一个。
- 微信号/手机号至少一个。

### `DELETE /api/v1/seat-swap/my-request`

登录必需。撤销未正式匹配的 active 请求。

### `POST /api/v1/seat-swap/matches/{target_request_id}/confirm`

登录必需。确认某个双向候选。

- 若之前确认过别人，则覆盖。
- 若对方也确认我，则双方请求进入 `matched`。

### `POST /api/v1/seat-swap/matches/{target_request_id}/cancel`

登录必需。正式匹配成功后的撤销。

入参：

- 撤销说明
- 截图文件

截图上传到 MinIO，撤销记录保存 object key 和 URL。

## 小程序设计

新增独立换座页面，并加入底部 tabbar。tabbar 从 5 个入口扩展为 6 个入口：

- 首页
- 榜单
- 洞察
- 换座
- 赛程
- 我的

建议页面路径：

```text
pages/seat-swap/index
```

换座页保持现有小程序视觉语言，但信息密度比洞察页更偏工具型：顶部展示当前比赛状态，中部处理我的换座请求，下部展示候选撮合列表。

### 当前比赛状态卡

- 有当前成都蓉城比赛：展示对阵、开赛时间、发布状态。
- 无当前比赛：展示“当前暂无可换座比赛”，隐藏发布按钮。
- 未登录：展示说明和公共列表，操作按钮引导登录。

### 我的换座请求

- 选择当前分区。
- 填当前排、号。
- 添加多个目标位置。
- 填微信号/手机号。
- 已发布后可编辑更新。
- 未正式匹配可直接撤销。
- 正式匹配后可发起撤销，必须填说明并上传截图。

### 候选列表

展示字段：

- 昵称。
- 当前座位：分区、排、号。
- 想换分区列表和可选排/号偏好。
- 发布时间。
- 候选状态。

联系方式展示：

- `communicable`、`waiting_peer_confirmation`、`peer_confirmed_me`、`matched` 状态下，对当前登录用户返回并展示。
- `display_only` 和未登录状态不展示。

## 测试与验证

后端：

- 应用层测试双向匹配、确认覆盖、双方确认成功、联系方式可见性、撤销规则。
- Web/route 测试登录权限、当前比赛不可用时拒绝发布、非双向候选不能确认。
- MinIO 存储端口使用 fake port 测试撤销记录写入，不依赖真实 MinIO。

前端：

- API URL 和 DTO 测试。
- 表单校验测试：联系方式至少一个、当前排号必填、目标分区必填。
- 页面手工验证小程序/H5 构建。

完成后验证命令：

```bash
cd football_insight_service_backend_rs
cargo test

cd ../football_insight_mini
bun run type-check
bun run build:mp-weixin
```

如果修改后端并本地接口验证，必须重启本地后端进程后再验证新行为。
