# 最佳盯盘时段设计

日期：2026-10-16
状态：修订中；目标已明确为具体座位级统计，座位标识与数据源映射仍需核实，尚不能按原区域算法交付
范围：`ticket-monitor-axum`（新增一个只读统计接口）、`football_insight_service_backend_rs`（ticket_watch 模块聚合接口、paywall 白名单）、`football_insight_mini`（余票监控页新增面板）
不涉及：热度预测、智能提醒推送、任何支付和商品改动。具体座位级回流数据属于本任务的目标，不能以区域余票总数替代。

---

## 0. 给实现 Agent 的提示词（先读这一节）

你要实现本设计文档。通读全文后按第 8 节顺序做。硬性规则：

1. 先读各子项目的约定：`/home/betalpha/projects/football_insight/CLAUDE.md`、`football_insight_service_backend_rs/AGENTS.md`、`football_insight_mini/AGENTS.md`，以及 `/home/betalpha/projects/ticket/ticket-monitor-axum/AGENTS.md`、`CLAUDE.md`、`docs/agents/architecture.md`。
2. 以第2节的具体座位级口径为准。先核实采集记录中的座位唯一标识，再定稿查询与接口字段；不得把区域库存统计换个名称交付。第12节保留的旧区域算法/数字仅用于溯源，不是座位级验收基准。
3. 有三处改动，分属两个仓库。monitor 仓库在 `/home/betalpha/projects/ticket/ticket-monitor-axum`，在那里用它自己的 git 提交，沿用它的提交风格，**不要部署、不要重启生产服务**。
4. 不要运行迁移，也不要部署。本设计**没有数据库迁移**。验收用的数据库查询一律只读；不要对生产库写入，也不要在 monitor 的 MySQL 上做任何 DDL 或写操作。
5. 工作区里可能有其他 agent 的未提交改动（尤其是 `football_insight_mini/src/pages/ticket-watch/` 下的历史比赛筛选相关文件）。**不要 stash、不要 revert、不要提交别人的文件**；用 `git add <具体文件>` 只提交自己改的，并且只提交整个文件都是你改的内容；如果同一文件里混有别人的改动，停下来并在汇报里说明，不要硬提交。
6. 小程序遵守：无渐变、无散落十六进制色值，只用 `src/styles/fi-tokens.css` 里的设计变量（需要新变量时按现有两层命名新增），不往 `scripts/check-import-boundaries.mjs` 的 ALLOWLIST 加例外，组件层不 import `api/**`。
7. 每个仓库改完都跑各自的验证命令并贴出结果。后端 `cargo test`；小程序 `bun run type-check`、`bun test`、`bun run check:boundaries`、`bun run verify:mp-components`；monitor `cargo test`。已知 `bun test src/pages/ticket-watch` 里 4 个"历史筛选"用例属于另一个 agent 的进行中工作，若仍失败请在汇报中单列，不要去修。
8. monitor 提交并推送 master；football_insight 提交并推送 main。subject 用英文，结尾带 `Co-Authored-By` 行。后端改完要重启本地后端再验证（见根目录 CLAUDE.md），汇报时分开写：本地后端已重启 / 生产未部署。
9. 汇报时逐条对照第 9 节的评审清单，每条给出证据（命令输出、测试名、文件行号）。做不到的写明原因，不要含糊带过。

---

## 1. 背景与目标

回流看板已经展示：每个区块从开售后 10 分钟起累计的回流张数，并按占比着色（`resolveInventoryHeatLevel`）。它回答的是"这场到现在为止哪里出过票"，**没有回答"什么时候最值得盯"**。

现有监控库 `rs_block_info` 保存比赛余票采样。旧方案按编号库存变化推测补票时段；本次修订要求先确认具体座位身份，再统计其回流时间。用户需要的是可追溯的座位级依据，用来安排有限的盯盘时间。

**目标**：基于具体座位的回流记录，在余票监控页展示成都蓉城一线队主场一天中的补票高峰，让用户安排盯盘时间。统计应能追溯到同一个具体座位的记录；仅有“某区域剩几张票”的数据不能当作同一个座位的回流记录。

用户已明确：由 ticket monitor 提供接口，当前项目通过接口读取；比赛结束后离线计算，在线只读已经计算好的结果，不影响抢票任务。

**成功标准**：
- 数据源能够区分具体座位，能验证同一座位跨采样时刻的身份；禁止把区域名、采样记录主键或区域库存数量当作座位身份。
- 余票监控页的当前比赛 tab 出现"最佳盯盘时段"面板，展示 24 小时分布和前三个高峰时段。
- V6 及以上会员看到完整面板；其他用户看到最高的一个时段（预览）和升级入口。
- 新增付费位的曝光、点击进入现有付费度量，上线后能和其他付费位对比转化。

## 2. 数据口径（必须严格遵守）

### 2.0 数据源核查结果与术语

已直接核对 peiqian 正在运行的 ticket-monitor-axum 容器配置、MySQL 实例及源码写入链路：

- 当前使用的库为 `ticket_system`，成都监控写入 `rs_block_info`。
- 链路为 `BlockInfoMonitor::check_and_save_block_info → TicketApiClient::get_ticket_regions → save_block_info → MySqlBlockInfoRepository::create → INSERT INTO rs_block_info`。
- `block_name` 来源于票务响应 `item.name`，`inventory` 来源于 `item.usable_count`，同时写入比赛、票价和时间。表中的 `id` 是采样记录主键，不能直接作为稳定座位ID。
- 实际记录包括 `530 → 余票4张`、`105 → 余票3张`。现有表未保存独立的排号/座号字段。**这些编号是否能够对应用户所说的“具体座位”，必须沿实际采集与业务标识核实，不能仅凭 block/seat 字样判断，也不能由实现者擅自认定二者等价。**
- 此次核查使用只读事务并回滚；当前项目运行时不会直连该 MySQL。不得让用户自行回忆表名代替源码核查。

本设计中的“具体座位”指可在多次采样间稳定识别的同一个座位。“区域”指可能包含多个座位、记录一个余票总数的集合。二者必须在数据和接口中区分；不强制要求文本排号/座号，但必须有能证明唯一性的业务座位标识。

### 2.1 座位级事件与数据前置条件

1. 先定位真实的座位唯一标识及其采集来源，证明同一标识在同场比赛的多条记录间代表同一个座位；排号/座号或业务 seat_id 均需有依据。不得用采样自增id造一个“座位ID”。
2. 每条参与统计的记录必须能够映射为 `(schedule_id, seat_key, observed_at, availability)`。其中 seat_key 的真实字段映射需经过核实；这是当前尚未完成的前置条件，不是假定 rs_block_info 已经具备这些字段。
3. 在身份核实后，按 `(schedule_id, seat_key)` 的时间序列识别事件，再汇总到单场24小时桶。输出计数单位为“具体座位的补票事件次数”，不能直接累加连续采样，也不能把区域余票总数当作可识别座位清单。
4. 原设计的6秒监控间隔、首条记录排除与起算点之前记录参与prev等规则仅在能作用于同一座位序列时才可沿用；`up` 原先代表区域库存增加，不能不经验证就移植成单个座位的事件。座位状态与原appear/up的对应关系必须在数据标识确认后补齐，禁止在此之前照搬第12节旧SQL。
5. 如果实际保存的只有区域库存总数，必须在报告中说明座位数据缺口与受影响历史范围；不能从总数反推出哪些具体座位出现过，也不能将区域结果作为座位级成功结果。数据源未就绪时不发布该面板，接口降级unavailable。

当前可以继续完成数据源追踪、现有端口/缓存隔离检查与不依赖未定口径的工作；具体座位事件SQL、对账基准及统计成功结果必须等上述映射核实后再定稿。若需要新增采集或持久化，应先把方案、兼容影响和不影响抢票的依据补入设计，不得悄悄修改现有监控写入链路。

### 2.2 统计起算时间（项目统计口径）

只统计 `time >= sale_start_at + 10 分钟` 的事件。比赛缺少 `sale_start_at`（monitor 里是 `rs_match.begin_time`）时，**报错或跳过该场，不要默默改成全量统计**。注意：计算 `prev` 时要用整场数据，过滤只作用在事件的 `cur.time` 上，否则起算点附近的第一条会被误丢。

起算固定10分钟，是根目录AGENTS.md既有统计口径，不做成配置项。monitor离线作业根据同一开售元数据计算起点；football后端仍计算since并随读取快照请求发送，monitor核对一致性。计算prev需保留整场同一座位的数据，过滤仅作用于当前事件时间。

### 2.3 时间与分桶

有效分桶字段为核实后的具体座位记录 `observed_at`，其真实物理字段及时间语义须在数据映射中注明。已核实当前区域来源 `rs_block_info.time` 是北京时间本地DATETIME，但这不能代替座位来源的核实。

对最终确认的北京时间本地时间，直接按 `HOUR(observed_at)` 分24桶（0到23），不重复增加8小时。若真实座位源使用其他时间语义，先在映射设计中明确，不得套用未验证假设。开售所在小时因+10min起算天然少10分钟，不做补偿。

### 2.4 座位级验收基准

原五场up/appear数字来自按block_name分组的库存序列，且存在同时间戳排序差异；它们不能证明具体座位级统计正确。旧数字及SQL仅在第12节保留溯源。

新的基准必须提供：实际座位标识映射、同一座位多次采样/回流的可追踪样例、同时间多座位的独立计数样例、开售+10min边界样例，并用同一份只读明细对账逐座位事件和24桶合计。不预设12/14/11为必须凑出的峰值，结果必须由核实后的座位数据计算。数据映射尚未完成时，不得宣称此项验收通过。

## 3. 总体架构

```
peiqian MySQL / monitor实际采集记录
   │ 先核实具体座位身份，比赛结束后独立离线计算
   ▼
单场具体座位补票24小时快照（原子发布、带统计粒度/版本标识）
   │ 只读快照，HTTP请求不触发SQL重算
   ▼
ticket-monitor-axum: GET /api/match/restock-profile/{match_id}
   │ HTTP（TicketMonitorPort）
   ▼
football_insight后台取快照 → 选场/聚合缓存
   │ 前台请求仅按有效会员分级裁剪缓存
   ▼
GET /api/v1/ticket-watch/best-windows
   │
   ▼
小程序当前比赛tab：最佳盯盘时段
```

football_insight不直连monitor的MySQL，也不在前台请求时触发上游计算。monitor缺少合格座位快照时返回未就绪，football返回HTTP200 + unavailable，前端隐藏。旧区域快照必须通过统计粒度与版本标识隔离，不能被当成座位级快照复用。

## 4. monitor 仓库：单场分桶接口

仓库 `/home/betalpha/projects/ticket/ticket-monitor-axum`，沿用六边形分层，放在 `region` 模块下（现有 `/api/match/block-info/:match_id` 所在模块），Handler 只解析协议。

**路由**：`GET /api/match/restock-profile/:match_id`，查询参数 `since`（必填，ISO 8601 带时区偏移，语义同现有 `block-info` 的 `since`，由调用方传 `sale_start_at + 10 分钟`）。缺少 `since` 返回 400，不要默默全量。

**座位级响应草案**（字段在座位来源核实后定稿，不代表现有block记录已经是座位明细）：

```json
{
  "success": true,
  "data": {
    "match_id": 582,
    "since": "2026-08-27T14:10:00+08:00",
    "stat_unit": "seat_restock_event",
    "schema_version": 2,
    "total_events": 3,
    "seats_with_events": 2,
    "hourly": [
      { "hour": 0, "events": 3 },
      { "hour": 1, "events": 0 },
      { "hour": 2, "events": 0 },
      { "hour": 3, "events": 0 },
      { "hour": 4, "events": 0 },
      { "hour": 5, "events": 0 },
      { "hour": 6, "events": 0 },
      { "hour": 7, "events": 0 },
      { "hour": 8, "events": 0 },
      { "hour": 9, "events": 0 },
      { "hour": 10, "events": 0 },
      { "hour": 11, "events": 0 },
      { "hour": 12, "events": 0 },
      { "hour": 13, "events": 0 },
      { "hour": 14, "events": 0 },
      { "hour": 15, "events": 0 },
      { "hour": 16, "events": 0 },
      { "hour": 17, "events": 0 },
      { "hour": 18, "events": 0 },
      { "hour": 19, "events": 0 },
      { "hour": 20, "events": 0 },
      { "hour": 21, "events": 0 },
      { "hour": 22, "events": 0 },
      { "hour": 23, "events": 0 }
    ]
  }
}
```

- `hourly` 恒为 24 项，按 `hour` 升序，没有事件的小时补 0。
- `match_id` 的取法和现有 `block-info` 一致（同样要处理"业务 ID 与外部 ID 的回退"，参考 `resolve_inventory_lookup_match_ids` 的做法在该接口里保持同样的语义；如果 monitor 一侧的既有接口已经有等价的解析，复用它）。

**离线查询与快照约束**：

- 真实seat_key字段确认后才编写座位序列SQL；不得在文档中虚构现有表列，也不得把旧 `PARTITION BY block_name` 查询换个字段名就声称完成。
- MySQL窗口查询只在独立离线作业执行，聚合返回24桶，不把整场明细拉到HTTP请求里循环计算；单连接、只读、超时有界。
- 生成范围覆盖全部符合结束条件且开售时间有效的成都蓉城精确主场，由football独自执行include/最近8场取样，避免遗漏较老有效样本。
- 默认跳过相同口径版本的已完成结果；重算原子替换，失败保留旧完整文件，不发布半份数据。不得复用旧区域口径缓存。
- 离线作业在售票中或即将开售时跳过，每场前重新核查；不挂接现有抢票循环或修改其角色调度。
- 在线用例只注入快照读取端口，不注入SQL计算端口；缺失、起点或统计粒度不符均不在线补算。
- 核验须包含座位身份正确、同座位重复采样不重复累计、不同座位独立累计、起算边界、24桶补零、原子更新、在线缺失不计算、旧区域快照拒绝，以及只读EXPLAIN/对账记录。

## 5. football_insight 后端：聚合接口

模块 `src/ticket_watch`，按现有四层写：

- `ports/ticket_monitor_port.rs`：新增 `fetch_restock_profile(match_id, fallback_match_id, since) -> RestockProfile`。
- `adapters/integration/http_ticket_monitor_port.rs`：实现它，路径构造沿用 `build_inventory_history_path` 的风格（`since` 做百分号编码）。
- `application/get_best_watch_windows.rs`：选场、取数、聚合、缓存、分级。业务逻辑只在这里。
- `adapters/web`：路由 `GET /api/v1/ticket-watch/best-windows`，Handler 薄。

### 5.1 选场规则

从 `fetch_all_matches` 的结果里选：

1. 已结束的比赛（开赛时间已过，或 `ticket_sale_ended` 为真）；
2. `include_in_reflux_stats` 为真；
3. 成都蓉城一线队的主场比赛（沿用小程序 `selectRefluxStatsMatches` 的"第一队精确匹配"语义，去掉预备队、青年队等；后端已有对应字段和规则就复用，没有就在应用层写成纯函数并测试）；
4. 有 `sale_start_at`（缺失的跳过并 `tracing::warn`，不要默默改全量）；
5. 取开赛时间最近的 **8 场**；
6. 该场起算后事件数小于 **20** 的丢弃（售罄过快或几乎没有补票的比赛，噪声大）。

最终样本少于 **3 场**时，返回 `status: "insufficient"`，不返回分布。

### 5.2 聚合

对每个小时 `h`：

- `avg_events`：各样本场在该小时的事件数的平均值，保留 1 位小数；
- `hit_matches`：该小时事件数大于 0 的样本场数；
- `share`：`avg_events / Σ avg_events`，保留 3 位小数（总和为 0 时全部为 0）。

`top_windows`：按 `avg_events` 降序取前 **3** 个小时，并列时取更晚的小时靠前不重要，取小时升序即可，需稳定可测。每项 `{ "start_hour": 12, "end_hour": 13, "avg_events": 38.4, "hit_matches": 7 }`，表示 12:00 到 13:00。**不要合并相邻小时**。

### 5.3 响应与会员分级

```json
{
  "status": "ok",
  "access": "full",
  "sample_size": 8,
  "generated_at": "2026-10-16T09:00:00+08:00",
  "top_windows": [ ... 最多 3 项 ... ],
  "hours": [ { "hour": 0, "avg_events": 0.4, "share": 0.002, "hit_matches": 2 } ]
}
```

- `access` 取值 `full` 或 `preview`，**由后端按会员等级判定，不信任客户端**。V6 及以上为 `full`（与现有"最近回流速览"的 `isMembershipTierAtLeast(tier, 'V6')` 保持一致，用后端已有的会员等级比较，不要新写一套）；其余为 `preview`。
- `preview` 时：`top_windows` **只含第 1 项**，`hours` 为**空数组**，`sample_size` 照常返回。预览不能泄露完整分布。
- 未登录或取不到会员等级时按 `preview` 处理。接口本身沿用同模块其他 ticket-watch 接口的鉴权方式（看 `current-board` 接口怎么取用户），不要自创。
- `status` 取值：`ok`、`insufficient`（样本不足）、`unavailable`（monitor 不可达或出错，HTTP 仍返回 200，前端据此隐藏面板；同时 `tracing::warn` 记录原因）。

### 5.4 缓存

- 单场座位快照：键包含比赛ID和统计口径版本，不设过期；进程重启重新读取monitor离线快照，不能触发窗口SQL，也不能混用旧区域快照。
- 聚合结果：后台每10分钟读取已完成快照并替换缓存；前台只读聚合缓存，后台替换期间旧完整结果仍可读。
- 后台首次读取同场快照合并并发；前台缓存未就绪返回unavailable，不触发上游拉取或重算。
- 缓存的是不含 `access` 分级的完整结果，分级在每次响应时按当前用户裁剪，**不能把 full 结果缓存给 preview 用户**。

### 5.5 付费度量白名单

`src/activity/domain/paywall.rs` 的 `PAYWALL_SURFACES` 新增 `ticket_watch_best_window_lock`，并更新对应测试和 admin 漏斗里如果有硬编码入口列表的地方（先 grep `ticket_watch_recent_reflux_lock` 找出所有位置，逐个处理）。不需要迁移，`surface` 是文本列。

### 5.6 后端测试

`cargo test`，至少覆盖：选场的 6 条规则各一个用例；聚合平均、命中场次、占比、前三、并列稳定；样本不足；preview 不泄露 `hours` 和第 2、3 个时段；monitor 失败返回 `unavailable`；缓存不串级（先 full 后 preview 的请求序列）；`ticket_watch_best_window_lock` 被接受、未知入口仍被丢弃。

## 6. 小程序

### 6.1 位置与结构

余票监控页（`pages/ticket-watch`）**当前比赛 tab**，放在"今日监控状态/速览"之后、区块库存列表之前。新增：

- `components/TicketBestWindowsPanel.vue`：只负责展示，**不 import `api/**`**，数据和事件走 props/emit。
- `useTicketWatchBestWindows.ts`：请求、状态、过期保护（沿用本目录其他 composable 的会话 revision / owner 防护写法）。
- `bestWindowsHelpers.ts`（+ 测试）：时段文案、柱高计算、是否当前小时等纯函数。
- `src/api/ticketWatch.ts` 新增 `getTicketWatchBestWindows()`，类型放 `src/types/ticketWatch.ts`。

另一个 agent 正在改这个目录下的"历史比赛筛选"文件。**不要动 `ticketWatchHistory*`、`effects.test.ts` 等**；`index.vue` 里只加你自己的几处，用 `git add -p` 式的最小提交（见第 0 节规则 5）。

### 6.2 面板（`access === 'full'`）

- 标题"最佳盯盘时段"，副文案一行："近 N 场主场的补票分布"（N 取 `sample_size`）。
- 24 个竖条（用 `view` 画，高度按 `avg_events` 相对最大值的百分比，最小可见高度保证非零桶看得见）。竖条之间留间隙；横轴只标 0、6、12、18 四个刻度，避免拥挤。
- 前三个高峰小时的竖条用主色（墨色）强调，其余用淡色；**当前小时**的竖条下方放一个小标记"现在"（页面每分钟刷新一次当前小时即可，不要每秒）。
- 面板下方一行结论："补票最集中在 12:00–13:00、14:00–15:00、11:00–12:00"（取 `top_windows`，不够 3 个就按实际数量）。
- 一行小字说明："按历史补票次数统计，仅供参考，不代表之后一定出票。"
- 点任意竖条可展开一行"X 点：近 N 场里有 M 场出现补票"（用 `hit_matches`）。同一时间只展开一个。

### 6.3 面板（`access === 'preview'`）

- 只展示"最容易出票的时段：12:00–13:00"一行（取 `top_windows[0]`）和样本说明。
- 其下是锁定提示，样式参考 `TicketRecentRefluxLock.vue`：文案"开通 V6 查看全天分布和前三个高峰时段"，按钮"升级查看"，跳转目标与该组件的升级按钮完全一致（购买页）。
- 不渲染任何柱状图，不要把真实分布放进去再用模糊遮住（预览数据后端已裁剪，前端也没有可画的数据）。

### 6.4 状态与降级

- 加载中：用现有的 `fi-skeleton-*` 全局骨架样式占位，高度与完整面板接近，避免页面跳动。
- `status` 为 `insufficient` 或 `unavailable`、请求失败、或审核期间 `systemConfigUnderReview`：**整块不渲染**，不弹 toast、不显示错误卡。
- 切换账号（会话 revision 变化）时重置并重新请求，旧请求的迟到结果不得覆盖。
- 比赛切换、轮询不触发重新请求；面板数据按会话缓存，页面 `onShow` 时若距上次成功超过 10 分钟再刷新一次。

### 6.5 付费度量

沿用现有 `usePaywallTelemetry`（见 `useTicketWatchState`/`index.vue` 对 `ticket_watch_recent_reflux_lock` 的处理方式）：

- 新增 `PaywallSurface` 成员 `ticket_watch_best_window_lock`（`src/types/paywall.ts`）。
- 锁定面板（preview）**可见时**上报一次曝光（同一页面访问内去重，沿用现有机制）；点"升级查看"时上报点击，`detail` 带 `matchId`（若有）。
- 不要改动现有两个付费位的埋点代码，只新增。

### 6.6 样式

只用设计变量；新增变量放 `src/styles/fi-tokens.css`，按"primitive 层 + 语义层"现有命名。不使用渐变、不写十六进制色值。竖条宽度和间隙用现有 spacing 变量。必须检查在手机窄屏下 24 根竖条不溢出，不出现横向滚动。

### 6.7 小程序测试

`bun test`：`bestWindowsHelpers`（柱高、最小可见高度、前三高亮、当前小时、文案拼接，含 0 值、全 0、不足 3 个高峰）；`useTicketWatchBestWindows`（成功、`insufficient`、失败静默、会话切换时旧响应不覆盖、10 分钟内不重复请求）；面板的 preview 与 full 两种形态（沿用仓库里组件测试的方式；若仓库没有组件渲染测试，就把分支判断抽成纯函数测试）。

## 7. 与现有功能的关系

- 现有"区块累计回流热力"不变，也不改它的颜色规则。
- 现有"最近回流速览"（V6 锁定）不变。本面板是新增的第三个会员权益位，门槛同为 V6。
- 本设计**不预测**未来某一分钟的概率，只呈现历史时段分布。

## 8. 实施顺序

0. 从ticket monitor实际监控写入链路定位表及编号含义，完成第2.1节的座位唯一标识映射；若存在区域聚合与具体座位的缺口，先补齐设计与数据来源，不能伪装成已完成。
1. monitor：核实后的座位级离线计算、版本隔离快照、只读接口与真实只读对账；提交推送master。
2. football：HTTP读取接口、后台聚合、有效会员裁剪、缓存与白名单；本地重启后验证full/preview。运行时不直连MySQL。
3. 小程序：独立helpers/composable/面板与新付费位，不改现有轮询、历史筛选和旧埋点。
4. 按第9节审查并汇报。原区域实现仅保留为未交付的工作，不算座位级完成。代码推送与生产发布分开报告，发版范围和时机按用户指令执行。

## 9. 评审清单（逐条给出证据）

1. 数据身份：真实数据源及seat_key映射已核实，能追溯同一个具体座位；区域总数、采样主键不冒充座位ID。
2. 事件与对账：座位状态/重复采样/重新回流规则已定稿并有表驱动测试；逐座位和24桶合计与只读明细一致。第12节旧区域数字不作为此项通过依据。
3. 起点与分桶：只统计sale_start_at+10min后事件，缺起点跳过并warn；prev保留起点前同座位记录，按核实后的北京时间observed_at分桶，不重复换算。
4. 选场：结束、include、一线队精确主场、有效开售时间、最近8场、事件20门槛均有测试；最终不足3场返回insufficient。
5. 离线隔离：比赛结束后由monitor独立计算；线上monitor只读版本化快照，缺失/错粒度不补算；football前台只读聚合缓存，不直连MySQL。
6. 降级与分级：monitor快照缺失/起点或粒度不匹配返回未就绪错误（404），不补算；football接口在monitor错误/座位源未就绪时返回HTTP200+unavailable。preview hours=[]且仅第1时段，full缓存不串给preview。
7. 权益：后端复用有效会员与V6等级比较，身份失败只给preview。
8. 付费度量：新surface加入白名单；旧两个入口的埋点代码不变，新曝光只在可见preview时发生。
9. 小程序：无新增渐变、hex或ALLOWLIST；预览不画分布，失败/不足/审核整块隐藏，无toast。
10. 展示：窄屏24柱不溢出，“现在”只标当前小时，组件分钟级时钟隐藏/卸载清理。
11. 交付：未混入他人改动，文档与实际口径一致，验证输出真实；不把草稿/数据缺口当作完成，不对生产库写入或执行DDL，推送/迁移/部署/上传分别汇报。

## 10. 上线后观察

上线（由用户指令触发）后用付费度量对比 `ticket_watch_best_window_lock` 与 `ticket_watch_recent_reflux_lock` 的点击率，并在 10 月 22 日开售窗口之后看这个入口带来的购买页访问和下单。若 preview 的点击率明显高于现有锁定面板，说明"预览一个时段"是有效的钩子，再考虑把同样的做法用到其他锁定位。

## 11. 2026-10-16 用户沟通后的定稿边界与当前状态

- 目标：具体座位级回流统计，不接受仅按区域余票总数冒充具体座位结果。
- 架构：ticket monitor拥有MySQL数据并提供接口；当前项目通过HTTP读取，不直连该库。
- 执行：比赛结束后独立离线统计，在线仅读计算结果，不影响正在运行的抢票程序。
- 分支：monitor直接提交master，football提交main；不新建分支，提交内容可单独回滚。
- 已核实数据：peiqian当前monitor使用ticket_system.rs_block_info；表和写入链路已找到，不要求用户回忆名称。具体座位标识与block_name的业务对应尚未证实，不能凭列名下结论。
- 当前实现状态：旧区域算法已有未提交代码与测试，尚非本修订目标的交付。不能继续将其作为座位级功能提交/发布；应先完成第2.1节数据身份核验。
- 本次修订只更新设计与实施计划，不宣称功能已实现、部署或上传。生产数据库未写入、未迁移。

## 12. 原区域方案归档（仅供溯源，不得作为本修订实施规格）

以下保留旧事件定义、参考数字与SQL，解释已有分析/未提交代码的来历。它们按block_name库存序列计算，不证明具体座位身份。任何旧测试通过或基准吻合，都不能代替第9节的座位级验收。

### 12.1 旧区域事件定义

数据源是 monitor 的 `rs_block_info(schedule_id, block_name, inventory, price, time)`。监控只在页面为"立即购买"且该区块有票时才写记录，有票期间约每秒一条；没票时没有记录。因此"补票事件"从同一场比赛、同一区块的相邻两条记录推出：

对每个 `(schedule_id, block_name)`，按 `time` 排序，设相邻记录为 `prev` 和 `cur`：

- **appear（重新出现）**：`cur.time - prev.time` 超过 **6 秒**。含义：该区块曾经没票或没被监控到，现在又有票了。
- **up（库存增加）**：间隔不超过 6 秒，且 `cur.inventory > prev.inventory`。
- 其余（库存不变或减少）不是事件。
- 一个区块在该场比赛里的**第一条记录**没有 `prev`，**不计为事件**。

`restock_events = appear + up`。统计的单位是**事件次数**，不是张数。

6 秒是常量，写成命名常量并加注释，不要散落魔法数字。已知局限：监控中断也会产生一次 appear，所以事件数略有高估；文案上不要把它说成"精确"。

### 12.2 旧区域参考数字（非座位验收基准）

我用独立脚本对生产监控库做了只读分析。这是旧区域方案当时记录的参考值，不能要求新的座位级接口复现这些总数：

| schedule_id | 对手 | 起算后事件总数 | up | appear |
|---|---|---|---|---|
| 577 | 山东泰山 | 523 | 9 | 512 |
| 580 | 武汉三镇 | 1371 | 169 | 1198 |
| 581 | 上海申花 | 253 | 4 | 247 |
| 582 | 辽宁铁人 | 1282 | 251 | 1029 |
| 583 | 天津津门虎 | 222 | 24 | 196 |

旧文档的总数列与up+appear存在不一致，当时要求按旧区域规则排除首条（见12.1），以类别列为参考。该说明仅记录旧方案的历史，不能给当前座位级方案提供计数基准。

分布形态（跨 562 号以后的主场比赛，按每场归一化后叠加）：峰值在 **12 点**，其次 **14 点**，再次 **11 点**；凌晨 2 点到 7 点极少。此形态仅是旧区域分析的描述，不作为座位级结果的验收约束。

### 12.3 旧区域SQL与测试（仅供溯源）

用 MySQL 8 窗口函数一次算完，不要把全部行拉到应用里循环。思路：

```sql
SELECT HOUR(time) AS hour,
       SUM(CASE WHEN gap > 6 THEN 1 ELSE 0 END) AS appear_events,
       SUM(CASE WHEN gap IS NOT NULL AND gap <= 6 AND inventory > prev_inventory THEN 1 ELSE 0 END) AS up_events
FROM (
  SELECT block_name, time, inventory,
         LAG(inventory) OVER w AS prev_inventory,
         TIMESTAMPDIFF(MICROSECOND, LAG(time) OVER w, time) / 1000000.0 AS gap
  FROM rs_block_info
  WHERE schedule_id = ?
  WINDOW w AS (PARTITION BY block_name ORDER BY time)
) t
WHERE time >= ? AND gap IS NOT NULL
GROUP BY HOUR(time)
```

`blocks_with_events` 另用同一子查询按 `block_name` 去重计数。注意 `time` 带微秒（`datetime(6)` 或秒级，两种数据都存在），间隔用微秒差。该查询对大比赛（约 27 万行）也必须在可接受时间内完成；先在只读副本或低峰用 `EXPLAIN` 看一下，**不要对生产库加索引**，如果需要索引，在汇报里提出，由用户决定。

**测试**（monitor 仓库 `cargo test`）：把"事件判定"抽成纯函数或纯 SQL 之外可测的逻辑做单元测试，覆盖：间隔恰好 6 秒（不是 appear）、6.001 秒（appear）、库存不变、库存减少、首条记录不计、`since` 之前的记录不计事件但参与 `prev`、24 桶补零。仓库已有 MySQL 测试方式的话沿用，否则用上面的纯逻辑测试，并另外手工对12.2节的旧区域样本做只读核查（不替代座位级对账），把结果贴进汇报。
