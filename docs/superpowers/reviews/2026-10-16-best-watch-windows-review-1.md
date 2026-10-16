# 最佳盯盘时段：第 1 轮评审意见

日期：2026-10-16
评审对象：monitor 仓库（`/home/betalpha/projects/ticket/ticket-monitor-axum`，未提交的工作区）和 football 后端（`football_insight_service_backend_rs`，未提交的工作区）
对照文档：`docs/superpowers/specs/2026-10-16-best-watch-windows-design.md`
结论：**未通过，需修改后再评审。** 有 1 个会让功能整体不可用的阻塞问题，2 个健壮性问题，以及交付缺口。

给修复 agent 的规则不变：不部署、不迁移、不写生产库；只提交自己改的具体文件；monitor 提交 master，football 提交 main，subject 英文，带 Co-Authored-By；不要碰别的 agent 的文件。

---

## 阻塞问题

### B1. 离线批处理会在第一场大比赛上失败并中止整轮，导致没有任何快照

`src/bin/rebuild-restock-profiles.rs` 设置了 `MAX_EXECUTION_TIME=5000`（5 秒），候选比赛按 `id` 升序处理，且循环里 `rebuild.execute(...).await?` 一出错就退出。

我用你们同一条 SQL（`RESTOCK_PROFILE_SQL`）在只读连接上实测：

| schedule_id | 记录数 | 结果 |
|---|---|---|
| 551 | 267,279 | **5.03 秒超时**（`3024 Query execution was interrupted`） |
| 556 | 80,245 | 1.51 秒 |
| 573 | 60,670 | 0.98 秒 |

验证报告里的 0.07 到 0.19 秒只覆盖了 577、580、581、582、583 这五场中等规模的比赛，没有测最大的 551。551 是已结束的成都蓉城主场，会进入候选，而且排在最前面。结果是：每次批处理都在 551 上超时、`?` 退出，556 到 583 以及之后的 584 永远不会生成快照，football 一侧面板永远 `unavailable`。

必须修：

1. 批处理里**单场失败不能中止整轮**：捕获每场错误，`tracing`/`eprintln` 记录 match_id 和原因，继续处理下一场，全部处理完后若有失败则以非零码退出。
2. **按开赛时间从新到旧处理**（最近的比赛最重要，football 只需要最近 8 场），而不是 id 升序。
3. 离线批处理的单条 SELECT 时间预算调到 **60 秒**（仍是单连接、只读、有界；离线作业，不影响抢票）。把数值写成命名常量并在文档中说明。
4. 加测试：一场失败时后面的比赛仍被处理、最终返回失败；候选顺序为新到旧。
5. 对 551、556、573 以及最近一场已结束的比赛各做一次真实只读对账，贴出耗时和总事件数。验证报告里要列出**最大比赛的耗时**。

---

## 健壮性问题

### R1. football 聚合：任何一场快照未就绪，整个面板变成 unavailable

`get_best_watch_windows.rs` 的 `build_aggregate` 对 8 个候选逐个 `fetch_restock_profile(...).await?`。比赛刚结束、离线作业还没跑时，最近一场的快照是 404，`?` 会让整次聚合失败。结果是每次新比赛结束到离线作业跑完之间，面板对所有用户消失。

修改：区分"未就绪（404）"和"其他错误"。未就绪的比赛 `tracing::warn` 后**跳过**，用剩下的场次聚合；剩余有效场次少于 3 场才返回 `insufficient`。非 404 的错误（超时、500、快照校验失败）仍视为失败。注意第 5.1 节的顺序：先取最近 8 场，再丢弃事件数小于 20 的，**跳过未就绪的场次不应该往后再补取更老的比赛**（保持"最近 8 场"的语义）。补测试：8 场中 1 场 404、3 场样本、全部 404 三种情况。

### R2. football 刷新失败会覆盖上一次可用的聚合结果

`refresh_at` 出错时写入 `unavailable` 并更新时间戳，于是一次 monitor 的短暂故障会让面板在接下来至少 10 分钟内全部消失，且之前算好的结果被丢弃。`GetBestWatchWindowsUseCase` 的注释说"existing complete aggregate stays readable while refreshing"，实现与注释不一致。

修改：刷新失败时**保留上一次成功的聚合**，只记录告警；没有任何成功结果时才保持 `unavailable`。失败后不要等满 10 分钟，可以缩短重试间隔（例如 1 分钟）；成功后恢复 10 分钟。补测试：先成功、再失败，前台仍能拿到旧结果。

---

## 交付缺口

1. **小程序部分没有任何改动**（没有 `TicketBestWindowsPanel.vue`、composable、helpers、api、类型、付费位埋点）。设计第 6 节整节未实现。
2. **football 的验证报告不存在**：monitor 的报告里写"football 完整交付报告在其 docs/best-watch-windows-validation.md"，但该文件不存在。
3. **两个仓库都没有提交**（monitor 和 football 的改动都在工作区）。按约定修完并验证后提交推送。
4. 第 9 节评审清单要逐条给出证据，目前只有 monitor 一半。

---

## 风格问题（必须改）

football 新增的四个文件是挤成一行的写法，不符合仓库的 rustfmt 风格：

- `src/ticket_watch/application/get_best_watch_windows.rs`（4 处差异）
- `src/ticket_watch/domain/best_watch_windows.rs`（4 处）
- `src/ticket_watch/adapters/web/best_windows_dto.rs`（1 处）
- `src/ticket_watch/adapters/integration/restock_profile_dto.rs`（1 处）

对这四个文件分别运行 `rustfmt --edition 2021 <文件>`，并检查没有波及别的文件（仓库里其他文件的格式差异是存量，不要碰）。另外有三处测试代码里 `ticket_sale_ended: false,` 缩进错位（`get_current_ticket_watch_board.rs`、`http_ticket_monitor_port.rs` 两处），一并整理。monitor 一侧的新文件格式正常。

---

## 已核对、没有问题的部分

- 事件判定：6 秒为命名常量，微秒比较，严格大于 6 秒才算 appear；首条不计；`since` 只过滤当前记录、`prev` 用整场；单测覆盖边界。
- SQL：`ORDER BY time, id`、整场 `LAG` 后再过滤、`HOUR(time)` 不换算时区；只读事务并回滚。
- 快照：同目录临时文件加 `rename` 原子替换；读端校验版本、match_id、24 桶总数、appear+up=total；HTTP 缺少或无效 `since` 返回 400，快照缺失或起点不一致返回 404；在线路径无 SQL 计算端口。
- football：`since` 由后端算好随请求发送并在响应里核对；选场 6 条规则有测试（含 8 场上限、`include_in_reflux_stats`、精确主场名、缺少开售时间跳过）；`full` 与 `preview` 分级在后端、按有效会员等级（`get_user_membership_tier` 内部已处理过期回落）、`preview` 不含 `hours` 和第 2、3 个时段；缓存里存的是完整结果，每次响应现裁剪，不会把 full 给 preview；身份查询失败按 preview；`ticket_watch_best_window_lock` 已加入白名单和测试；路由在缓存未就绪时返回 HTTP 200 + `unavailable`。
- 我本地运行 `cargo test --lib ticket_watch`：34 个全部通过。

## 关于对账差异（接受，不需要再改）

验证报告里 577、580、582 的 `up` 与我给的参考值略有差异（7 对 9、168 对 169、238 对 251），原因是我的参考脚本在同一时间戳下按未定义顺序处理，而实现按 `time, id` 排序；报告也如实说明了，没有为凑数改算法。分布峰值 12、11、14 与我之前的 12、14、11 只是 11 和 14 两点的次序互换，我自己的数据里这两点是 2.22 对 2.18，几乎相同，不影响结论。

## 给后续小程序实现的两个提醒

- 取关公众号后"会员权益暂停"目前只在小程序端判断（`membership_benefits_enabled` / `resolveMembershipBenefitsLocked`），后端接口按等级判定。小程序面板在权益暂停时应按**未解锁**展示，与回流看板其余会员内容一致。
- 面板文案用"座位"（如"102 座位"），不要写"区块"或"区域"，见设计文档第 00 节。
