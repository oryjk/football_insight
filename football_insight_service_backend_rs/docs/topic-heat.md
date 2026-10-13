# 话题热度

首页、历史列表和详情响应的每个话题新增 `heat` 非负整数；原有正文、来源、音频和按时间排序契约不变。旧前端可忽略字段，旧批次无需补写 JSON。来源仅在小程序详情展示。

## 规则与存储

- 显示热度 = `base_heat`（后台手动设置的基础值）+ `view_count`（有效详情访问），最大为 JavaScript 安全整数 9007199254740991。
- 普通 GET 不计数；小程序成功打开详情后提交 `POST /api/v1/daily-topics/{id}/visits?team_id=77680`，JSON `{"visitor_id":"匿名设备 UUID"}`，返回 `{"heat":1}`。未发布话题/错误球队返回 404，无效 UUID 返回 400。
- 同一设备同一球队话题距上次有效访问满 30 分钟才再次累计；并发请求在 PostgreSQL 中原子去重，不延长重复请求的窗口。统计失败不影响正文和播报；热度读库失败单独降级为 0 并记录日志，正文存储异常仍返回错误。
- 设备 UUID 只保存在本机；服务端保存 SHA-256 hash，不记录 IP、手机号或用户账号。它是访问统计标识，不承担身份认证。
- `f_i_daily_topic_metrics` 以 `(team_id, topic_id)` 为主键，独立于 Python 批次；`f_i_daily_topic_visitors` 保存每个标识的最后计数时间。重新发布原话题或更新音频不会重置热度。
- 历史访问无记录，从上线后累计，初始基础值和访问值均为 0。

## 后台直接调整

在 `football_data` 中按真实话题 ID 修改基础热度即可，原文和音频 JSON 不需编辑。例如将指定话题基础热度设为 1000：

```sql
INSERT INTO f_i_daily_topic_metrics (team_id, topic_id, base_heat)
VALUES (77680, '<真实话题ID>', 1000)
ON CONFLICT (team_id, topic_id) DO UPDATE
SET base_heat = EXCLUDED.base_heat, updated_at = clock_timestamp();
```

```sql
SELECT topic_id, base_heat, view_count,
       LEAST(base_heat + view_count, 9007199254740991) AS heat
FROM f_i_daily_topic_metrics WHERE team_id = 77680;
```

热度随有效访问继续增加；如需精确设为某个总值，基础值取目标总值减当前 `view_count`（不得为负）。手动调整不会改变排序。

## 迁移与验证

新增迁移 `20261013000200_add_daily_topic_metrics.sql` 只创建独立新表，不修改已有表或批次。先在 disposable PostgreSQL 验证，再按现有生产约定在 peiqian 通过 `cargo run --bin run_migrations` 执行；部署脚本不会自动迁移。迁移先于新后端发布，旧后端兼容新表。

HTTP 回归覆盖字段兼容及非法访客/话题；数据库验证覆盖并发去重、30 分钟窗口、不同设备、手动基础值、球队隔离与原文重新发布。

### 本次代码验证

- 全量 `cargo test`：361 通过，7 项需独立 PostgreSQL 的测试默认忽略；显式 disposable 数据库运行这 7 项全部通过。
- `cargo check`、小程序类型检查、组件注册、依赖边界、双端构建通过；热度转换/显示及原有音频回归 17 项通过。
- H5 浏览器验证：列表无来源、详情保留来源，首页/列表/详情均显示热度；模拟 700ms 上报延迟，先返回列表仍更新累计值。
- 全仓 `cargo clippy -- -D warnings` 有 15 处已有问题（admin/ai/auth/insight/mini_review/payment/system_config/team_board/ticket_watch），本次话题模块未命中。架构扫描的既有 Web 间 DTO 导入、错误文案分支仍存在于其他模块，本次未扩大改动范围。

### 2026-10-13 发布验证

- 本地后端已重建并重启：`football-insight-local-test-backend:topic-heat`，18092 健康，全部原环境配置保留。
- peiqian 已使用 `cargo run --bin run_migrations` 应用新迁移，`_sqlx_migrations` 中 `20261013000200` 成功；生产后端已部署并重启为 `football-insight-service-backend-rs:4bf1ab8`，运行配置文件中的全部值保留。
- 线上历史六篇、首页五篇；访问上报成功，同设备重复上报不累加，首页与详情热度一致，来源和 ready 音频保留，余票及公共配置 HTTP 200。
- 小程序 1.0.66（robot 2）已从干净、已推送的 `4bf1ab8` 构建上传，包大小 1,825,371 字节；未生成预览二维码。
