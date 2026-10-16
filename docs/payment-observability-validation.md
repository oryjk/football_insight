# 支付漏斗可观测化交付与验证

日期：2026-10-16。设计：`docs/superpowers/specs/2026-10-16-payment-observability-design.md`。
基线：main `38ea2c0`。后端提交 `9c81a3e`；小程序为紧随其后的独立提交，均带 Co-Authored-By。

## 实现结果与发布边界

三类商品均关联 attempt；新增客户端上报与四个 Admin GET 查询。只加观测，不修改价格、购买界面、现有 API 契约、支付取消规则、订单状态机或 prepay_id。

- **本地后端已重启**：最后执行 `./deploy/local-test/up.sh`，北京时间 11:44:28 返回 active 与 health status=ok；端口 18092。
- **生产后端未部署、未重启**。没有上传小程序。
- **迁移未执行**：仅新增 `20261016120000_add_payment_attempt_events.sql`，等待用户指示在 peiqian 执行；没有自动迁移路径。
- 发布仍须按：用户指示迁移 → 部署后端 → 验证事件入口 → 上传小程序。事件保留策略 180 天，清理任务不在本期。

## 后续用户 Review 补充（2026-10-16）

用户逐项审查第 11 节后确认无阻塞问题，并反馈已在生产事务内执行事件迁移、5 条模拟事件及 5 条后台查询，结果符合预期；无条件回滚后表不存在、无残留。以下原交付记录保留当时证据，本补充为用户提供的审查结果，不冒充原实现 agent 的执行结果。stuck-orders 时间下限/索引及北京时间统计建议留待上线后另做；本次 P1 不修改它们，也不执行可观测化上线。

## 真实验证结论

| 命令 | 实际结果 |
|---|---|
| `./deploy/local-test/up.sh` | exit 0；编译成功，用户级服务 active，健康检查 ok；既有 compose orphan / no services to build 提示，不清理其他容器 |
| 后端 `cargo test` | exit 0；27 个 suite，**445 passed，0 failed，64 ignored** |
| 后端 `cargo check` | exit 0；Finished dev profile，最终 2.81 秒 |
| 小程序 `bun run type-check` | exit 0；vue-tsc --noEmit 无错误 |
| 小程序 `bun test` | exit 0；**672 pass，0 fail，58 files**，主 agent 最终复验 12.77 秒 |
| 小程序 `bun run check:boundaries` | exit 0；依赖边界检查通过 |
| 小程序 `bun run verify:mp-components` | exit 0；All mini-program custom components are registered. |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | exit 0；Build complete、组件注册与产物 API 校验通过；不登记版本，不上传；既有 uni-app / Browserslist 更新提示 |
| `git diff --check` | exit 0 |
| 改动文件 `rustfmt --check --edition 2024 --config skip_children=true` | exit 0；42 个 Rust 文件 |
| 额外检查 `cargo fmt --check` | exit 1；全仓既有未格式化代码（如未修改的 auth 模块）及本次当时未格式化文件被命中；随后只格式化本次 42 个文件并通过上述定向检查，没有扩展到无关模块 |

后端完整最后一次输出位于本地忽略目录 `.tmp/payment-observability/cargo-test-final.log`；该日志不提交。

## TDD 与独立审查

- 领域、存储失败、上报、HTTP 各组测试先因缺失模块失败，再实现通过。
- 三个下单用例先因缺少观测接入失败；实现后分别验证 order_created、pay_params_issued / pay_params_failed，Provider 原错误仍可 downcast。
- 结算和退款先缺少 builder 导致失败，再用 fake 验证三类商品成功、原错误保留、重复发货跳过、无限等待 recorder 不拖慢结果。
- 小程序可选参数及上报先失败；三类购买流程先有 10 个失败再通过；UTF-8 限制先失败再通过。
- 迟到前置事件、逆序支付批次回归分别先得到错误归因再修正；缺失漏斗阶段测试先实际 5 行 / 期望 29 行失败，再通过。
- 独立 reviewer 检查整个 diff，提出并复查上述乱序、关联回退与漏斗补零；最终无 Critical / Important 阻塞项。

## 设计第 11 节逐项自查

| # | 结果 | 依据 |
|---|---|---|
| 1 埋点失败不影响主流程 | 满足 | `payment_event_recording.rs` 阻塞 / panic recorder；`payment_event_storage_failure.rs` 已关闭 pool 吞错；结算 / 退款 blocked recorder 测试；mini 上报 404/500/断网不抛错且无 Toast/Modal |
| 2 unified_order 失败原错误、状态、文案 | 满足 | OrderAttempt::params 返回传入的同一个 anyhow::Result；三类用例 fake Provider downcast；三个现有 handler 的错误映射和 DTO 保持原样 |
| 3 不传 attempt 行为与取消规则 | 满足 | 原取消表达式 `errCode === -2 || /cancel|取消/i` 保留；原错误文案、reject、十次轮询不变；已有与新增无 attempt 用例及全量 mini 测试通过 |
| 4 上报无越权 | 满足 | JWT 后校验订单 user_id；外用户 / 缺失 / 查询失败整批丢弃；服务端事件不在客户端白名单；HTTP 验证 204 / 401 |
| 5 detail 白名单和隐私 | 满足 | domain sanitize_detail 只保留规定标量键，不接收嵌套对象；1KB、300 Unicode 字符测试；mini 从明确设备字段采集，无 openid/token/手机号/完整请求体 |
| 6 迁移命名、时间、未执行 | 满足 | 时间戳 20261016120000 > 20261014163000；仅 CREATE TABLE、CREATE INDEX；没有调用 run_migrations 或手工 SQL |
| 7 不加 ALLOWLIST、不改版本 | 满足 | 这两个文件和 generatedMiniProgramVersion 均无 diff；边界检查通过；离线构建后版本仍为 1.0.73 |
| 8 三类商品覆盖 | 满足 | 三个下单用例成功 / 失败测试、结算三种 product_type 测试、mini 三个购买入口流程测试 |
| 9 未污染生产库 | 满足 | 没有新增写库测试；存储失败测试关闭 lazy pool 后调用，无连接；全量测试保留 64 个数据库测试 ignored；没有真实支付或手工业务写入 |
| 10 第 7.1 节向后兼容 | 满足 | 下表逐条检查；后台 INSERT / 关联查询全部在有界 spawned task，观测 pool 与支付 pool 分离 |

### 第 7.1 节逐条补充

| 要求 | 结果和依据 |
|---|---|
| 迁移只新增表、索引 | 满足；没有 ALTER、DROP 或既有数据回填 |
| 现有路径、请求体、响应体、状态、文案不变 | 满足；旧 DTO 未改；只新增事件入口和 Admin GET，旧 handler 仅传递请求头观测上下文 |
| 可选 header，缺失或非法不拒绝 | 满足；domain attempt_id 单测覆盖有效、缺失、非法；nil 也回退新 UUID |
| 旧小程序不上报仍记服务端事件 | 满足；服务端创建自己的 attempt，独立于客户端上报 |
| 新小程序对旧后端静默 | 满足；404/500/网络失败吞错测试，无任何提示框 |
| 两个公共函数参数可选、原行为不变 | 满足；原无 attempt 用例和全量测试通过，新增逻辑通过可选参数隔离 |
| 表缺失、超时不影响支付 | 满足；后台任务 250ms deadline、128 任务容量，独立最多 2 连接 / 100ms 获取超时；错误只 warn |
| 发布顺序 | 满足；仅交付代码与迁移文件，尚未执行任何生产动作 |

## 偏离与补充说明

1. **归因采用阶段证据优先级，而非机械读取最后入库事件**：500ms 合批和并发上报可能逆序；否则已经取消 / 失败的 attempt 会被早期“拉起支付”覆盖。确认超时优先于客户端成功回调，服务端 delivered / refunded 最优先。只影响 derived 查询，完全不改支付状态。已加先失败再通过的回归。
2. **异步隔离的具体实现**：record port 保留 async 签名，application 在有界 spawned task 内调用，并捕获 panic / 超时；另开最多两连接的观测 pool，避免观测占用支付连接。这是 7.1 非阻塞要求的具体化。
3. **生命周期关联**：优先该订单最早的服务端 attempt，服务端初始写入丢失时回退已有客户端事件；都没有时用不可变 order UUID。所有查询在后台完成，不改变结算 / 退款结果。
4. **未预期的服务端错误**：error_code 为 Other，不保存可能携带凭据的任意底层错误原文；已知 VirtualPaymentError 保留固定消息，客户端微信原始 errCode / errMsg 保留并按设计截断。
5. **小程序类型与测试位置**：新增 DTO 在 types/paymentTelemetry.ts，避免 api 反向依赖 composable。会员 SFC handler 测试放 scripts，以免应用 TypeScript 环境引入 Node 类型。比赛 ID / 回流下单按实际代码分别修改 api/matchIdUnlock.ts 与 api/ticketWatch.ts，并非把它们迁入 api/payment.ts；实际路径均存在，没有缺失引用。
6. **Admin 查询细节**：按 UTC 自然日汇总并在 JSON 明示 timezone；设计未指定分日时区。from/to 为 RFC3339、区间最大 180 天，避免无界聚合；相邻转化率使用 attempt 交集，补齐零阶段。只读沿用 ViewAuditLogs 权限，不新增审计写入。

## 未做的测试及原因

- 没有新增 `tests/payment_events_postgres.rs`，没有执行现有 ignored 数据库测试。唯一数据库是生产，新增表尚未迁移；生产池 adapter 的独立后台写入无法保证包在测试事务里统一回滚，不能安全满足测试数据自建 / 回滚约束。因此使用 fake、纯函数和关闭连接池测试。**INSERT、索引与聚合 SQL 尚未在真实 PostgreSQL 运行验证**，当前为静态审查；需在用户指示迁移后验收。
- 没有发起真实支付，也没有对运营号或真实用户创建 / 修改 / 删除订单、会员或商品。微信 SDK 真实 errCode、真机上报、发货 / 退款端到端验证需要后续经用户指示进行。
- 没有微信开发者工具 / 真机验收；仅离线微信构建、公共支付函数与三类流程的 fake 测试。没有执行生产迁移、部署或小程序上传。
- 两周成功标准（未知原因低于 5%）必须上线采样后验收，本次不能提前声称达到。

## 改动文件清单

### 后端（44 个）

- `football_insight_service_backend_rs/docs/api.md`
- `football_insight_service_backend_rs/migrations/20261016120000_add_payment_attempt_events.sql`
- `football_insight_service_backend_rs/src/admin/adapters/persistence/mod.rs`
- `football_insight_service_backend_rs/src/admin/adapters/persistence/postgres_admin_payment_repository.rs`
- `football_insight_service_backend_rs/src/admin/adapters/web/mod.rs`
- `football_insight_service_backend_rs/src/admin/adapters/web/payment_handlers.rs`
- `football_insight_service_backend_rs/src/admin/adapters/web/payment_routes.rs`
- `football_insight_service_backend_rs/src/admin/adapters/web/payment_tests.rs`
- `football_insight_service_backend_rs/src/admin/application/admin_payment_service.rs`
- `football_insight_service_backend_rs/src/admin/application/mod.rs`
- `football_insight_service_backend_rs/src/admin/bootstrap.rs`
- `football_insight_service_backend_rs/src/admin/domain/admin_payment.rs`
- `football_insight_service_backend_rs/src/admin/domain/mod.rs`
- `football_insight_service_backend_rs/src/admin/ports/admin_payment_repository.rs`
- `football_insight_service_backend_rs/src/admin/ports/mod.rs`
- `football_insight_service_backend_rs/src/app.rs`
- `football_insight_service_backend_rs/src/match_id_unlock/adapters/web/handlers.rs`
- `football_insight_service_backend_rs/src/match_id_unlock/application/create_match_id_order.rs`
- `football_insight_service_backend_rs/src/match_id_unlock/bootstrap.rs`
- `football_insight_service_backend_rs/src/payment/adapters/persistence/mod.rs`
- `football_insight_service_backend_rs/src/payment/adapters/persistence/postgres_payment_event_recorder.rs`
- `football_insight_service_backend_rs/src/payment/adapters/web/event_handlers.rs`
- `football_insight_service_backend_rs/src/payment/adapters/web/handlers.rs`
- `football_insight_service_backend_rs/src/payment/adapters/web/mod.rs`
- `football_insight_service_backend_rs/src/payment/application/create_membership_order.rs`
- `football_insight_service_backend_rs/src/payment/application/handle_wechat_notify.rs`
- `football_insight_service_backend_rs/src/payment/application/mod.rs`
- `football_insight_service_backend_rs/src/payment/application/payment_events.rs`
- `football_insight_service_backend_rs/src/payment/application/report_payment_events.rs`
- `football_insight_service_backend_rs/src/payment/application/virtual_payment.rs`
- `football_insight_service_backend_rs/src/payment/application/virtual_payment_tests.rs`
- `football_insight_service_backend_rs/src/payment/bootstrap.rs`
- `football_insight_service_backend_rs/src/payment/domain/mod.rs`
- `football_insight_service_backend_rs/src/payment/domain/payment_event.rs`
- `football_insight_service_backend_rs/src/payment/ports/mod.rs`
- `football_insight_service_backend_rs/src/payment/ports/payment_event_recorder.rs`
- `football_insight_service_backend_rs/src/reflux_subscription/adapters/web/handlers.rs`
- `football_insight_service_backend_rs/src/reflux_subscription/application/create_reflux_subscription_order.rs`
- `football_insight_service_backend_rs/src/reflux_subscription/bootstrap.rs`
- `football_insight_service_backend_rs/tests/payment_event_domain.rs`
- `football_insight_service_backend_rs/tests/payment_event_ingestion.rs`
- `football_insight_service_backend_rs/tests/payment_event_recording.rs`
- `football_insight_service_backend_rs/tests/payment_event_storage_failure.rs`
- `football_insight_service_backend_rs/tests/payment_events_http.rs`

### 小程序（17 个）

- `football_insight_mini/scripts/payment-membership-telemetry.test.ts`
- `football_insight_mini/src/api/matchIdUnlock.ts`
- `football_insight_mini/src/api/payment.ts`
- `football_insight_mini/src/api/paymentTelemetry.test.ts`
- `football_insight_mini/src/api/ticketWatch.ts`
- `football_insight_mini/src/composables/confirmPaidOrder.test.ts`
- `football_insight_mini/src/composables/confirmPaidOrder.ts`
- `football_insight_mini/src/composables/paymentTelemetry.test.ts`
- `football_insight_mini/src/composables/paymentTelemetry.ts`
- `football_insight_mini/src/pages/membership-purchase/index.vue`
- `football_insight_mini/src/pages/ticket-watch/paymentActions.test.ts`
- `football_insight_mini/src/pages/ticket-watch/purchaseTelemetry.test.ts`
- `football_insight_mini/src/pages/ticket-watch/useTicketWatchMatchId.ts`
- `football_insight_mini/src/pages/ticket-watch/useTicketWatchSubscription.ts`
- `football_insight_mini/src/types/paymentTelemetry.ts`
- `football_insight_mini/src/utils/request.ts`
- `football_insight_mini/src/utils/virtualPayment.ts`

### 共享文档

- `FEATURES.md`
- `CHANGELOG.md`
- `docs/superpowers/plans/2026-10-16-payment-observability.md`
- `docs/payment-observability-validation.md`
