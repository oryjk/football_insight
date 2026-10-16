# 最佳盯盘时段验证报告

日期：2026-10-16。设计：`superpowers/specs/2026-10-16-best-watch-windows-design.md`；评审：`superpowers/reviews/2026-10-16-best-watch-windows-review-1.md`。

状态：实现和本地验证完成；monitor提交master、football后端和小程序分开提交main。monitor推送前同步了远端5个提交，仅补齐本任务3个测试夹具的新DTO字段；没有执行远端迁移。**本地后端已重启；生产后端未部署/未重启；未迁移、未写生产库、未上传小程序。** 推送代码不代表生产发布。monitor详细报告在其仓库`docs/best-watch-windows-validation.md`；前端细节见`../football_insight_mini/docs/best-watch-windows-validation.md`。

## 口径及评审修复

设计00已确定“座位”=`(schedule_id,block_name)`中的block_name编号，“区域”=price价位组。不需要更细的排号/座号。序列按time,id排序，整场LAG之后只统计cur>=开售+10分钟；>6秒appear，<=6秒且库存增加up，首条排除；按北京时间原值HOUR(time)分桶。未改变采集器或抢票行为。

|意见|落实及回归证据|
|---|---|
|B1|monitor命名60秒预算、单连接只读；候选按开赛时间倒序；逐场失败warn后继续、最终非零退出；失败继续/最终失败/新到旧测试先观察3失败再转绿；551最大场真实对账见下表|
|R1|football以RestockProfileNotReady区分404；固定最近8场之后跳过未就绪，不补老场；其他错误失败；`best_windows_skips_not_ready_within_fixed_latest_eight_without_backfill`覆盖8中1缺、剩3/2及全缺；HTTP测试区分404与500|
|R2|刷新失败保留旧aggregate和generated_at，60秒重试，成功600秒；`best_windows_failed_refresh_keeps_previous_result_and_retries_after_one_minute`验证成功→失败→659秒不刷新→660恢复→成功后600秒|
|交付缺口|小程序面板、composable/helpers/API/types/付费位完整；两仓报告及本报告已补；第9节逐条证据见下；分仓库/前后端分别提交|
|格式|四个新Rust模块及本地联调测试逐文件rustfmt检查通过；三处ticket_sale_ended fixture缩进已纠正，仅整理本任务新增hunk，未全仓格式化|
|复审P1|小程序真实logout/invalidate先变revision后清凭据，原同步请求捕获旧Bearer；实际session+API/request测试先RED，改为同步清缓存、等待微任务再读取凭据，发起前和发布前均校验owner；退出后只得preview|

R1/R2回归在原实现观察2失败/10通过后修复；v2/单位不匹配用例也先复现旧版本错误接收，再实现校验。初期domain/选场、缓存/裁剪、HTTP回退、路由和surface白名单均按TDD推进。最终复审未发现新阻塞问题。

## 验证命令与真实输出

命令均以rtk执行；全量测试没有使用统一`--ignored`，避免真实数据库写入测试。

|范围/命令|实际结论|
|---|---|
|根目录`./deploy/local-test/up.sh`|exit0；重新编译并重启用户systemd服务，active，健康接口status=ok，固定18092；Redis正常。已有orphan container提示未擅自清理|
|后端`cargo test`|509 passed / 0 failed / 68 ignored，34 suites；默认忽略包含隔离数据库测试和本地快照联调|
|后端`cargo check`|exit0，Finished dev，4.03s|
|后端`cargo build`|exit0，Finished dev，0.24s|
|后端`cargo clippy`|exit0，0 errors，16项既有lib warnings；无新best-windows模块告警|
|后端四个新模块及联调测试`rustfmt --edition 2021 --config skip_children=true --check`|exit0|
|后端`BEST_WINDOWS_LOCAL_MONITOR=http://127.0.0.1:4001 cargo test --test best_windows_local_monitor -- --ignored --nocapture`|1 passed / 0 failed；实际本地monitor HTTP快照及football路由，四个身份序列见下|
|小程序`bun test`|852 pass / 0 fail，5736 expect calls，73 files，14.34s|
|小程序`bun run type-check`|exit0，vue-tsc --noEmit|
|小程序`bun run check:boundaries`|依赖边界检查通过|
|小程序`bun run verify:mp-components`|All mini-program custom components are registered|
|小程序H5/离线微信构建|均Build complete；MINI_REVIEW_SKIP=1，版本保持1.0.85，无版本申请/上传|
|monitor `cargo test`|同步远端master后732 passed / 0 failed / 45 ignored，16 suites（同步前706/44）|
|monitor `cargo check --all-targets`|exit0；已有successful_orders_api_test两个unused import警告|
|monitor `cargo build --bins`|exit0，29.22s（同步前23.81s）|
|monitor `cargo clippy --all-targets`|exit0，0 errors，185项存量warnings；严格-D warnings此前被存量问题阻塞，未扩大范围修复|
|monitor本次16个Rust文件格式 / 两仓git diff --check|通过|
|monitor全仓cargo fmt --all -- --check|未通过，仅3个存量文件：group_ticket_order/domain/entities/group_ticket_order.rs、ticket_grabbing/application/strategy.rs、ticket_grabbing/domain/sharding.rs，均未修改|

原始本地日志位于会话scratch `.tmp/best-watch-windows/`，不提交。mini构建与浏览器脚本在其报告列明。

## 本地只读联调

独立monitor `restock-profile-server`监听127.0.0.1:4001，仅文件读取，无数据库、抢票和通知初始化；未停止已有4000监听。夹具源于此前只读对账的581/582/583快照，仅在scratch转换v2文件合同，无数据库写入。实际HTTP583返回schema_version=2、stat_unit=seat_restock_event、since=2026-10-08T14:10:00+08:00，total220=appear196+up24，seats_with_events54，24对象桶。

football集成用真实HttpTicketMonitorPort、DTO/用例/路由，赛程元数据和身份为隔离fixture，不伪称真实生产会员登录；无数据库访问，四次实际输出：

```text
HTTP200 status=ok access=full sample_size=3 hours=24 top_windows=3
HTTP200 status=ok access=preview sample_size=3 hours=0 top_windows=1
HTTP200 status=ok access=preview sample_size=3 hours=0 top_windows=1
HTTP200 status=ok access=preview sample_size=3 hours=0 top_windows=1
1 passed; 0 failed
```

依次为V6、匿名、有效等级已回落V3、无效token。真实18092新路由匿名读取也得到HTTP200、insufficient、sample_size0、hours[]；生产monitor尚无新接口，404按未就绪正常降级，**不是生产已可用**。前台请求无上游拉取由fake端口调用计数断言证明。

## 生产MySQL真实只读对账

monitor专用只读连接，从.env在内存中读取RS_DATABASE_URL，未输出连接串。每轮`START TRANSACTION READ ONLY`，finally无条件ROLLBACK并关闭；无写入/DDL/索引修改。脚本放会话scratch，通过uv/PyMySQL执行。

|schedule_id|行数|耗时秒|appear|up|total|有事件座位|
|---|---:|---:|---:|---:|---:|---:|
|551（最大）|267279|6.890|3500|87705|91205|59|
|556|80245|1.287|4633|31835|36468|61|
|573|60670|0.658|6657|3146|9803|60|
|583（最新结束）|5093|0.074|196|24|220|54|

551 EXPLAIN使用原rs_block_info_schedule_id_index；超过旧5秒、在新60秒预算内。每场24桶合计等于appear+up，完整24桶在monitor报告中。

五场参考复核：577实际appear512/up7/total519；580为1198/168/1366；581为247/4/251；582为1029/238/1267；583为196/24/220。同time原脚本顺序未定义，time,id稳定定序后up有差异；原total列本身与两类别也不相等。第1轮review已明确接受，未为凑数改阈值/去重/首条规则。20场主场按单场总数归一化叠加峰值12、11、14，相比原12、14、11的近似并列次序亦已接受。

## 设计第9节逐条自查

|序号|结果|依据|
|---|---|---|
|1 术语粒度|满足|monitor SQL PARTITION BY block_name、schedule_id过滤；seats_with_events按block_name去重，不按price合并；设计00已确认|
|2 事件/对账|满足|monitor domain表驱动6/6.001秒、首条、库存平降、多座位序列；SQL time,id；五场对账及最大551实测如上，24桶合计校验|
|3 起点/分桶|满足|monitor restock_candidates缺begin_time warn跳过；football select_best_windows_matches缺/非法sale_start_at跳过；+10min、整场LAG再过滤、HOUR(time)原值|
|4 选场|满足|best_windows_selection_enforces_ended_included_exact_home_and_sale_start、takes_latest_eight_after_metadata_filters；domain样本20边界、<3不足；R1固定8内跳过|
|5 离线隔离|满足|monitor rebuild命令独立，HTTP只依赖Reader；football foreground_never_fetches_and_background_is_single_flight断言调用数；运行时无MySQL直连|
|6 降级/分级|满足|monitor缺失/旧版/起点不符404，无补算；HTTP404/500类型回归；football默认200不可用，R1不足insufficient，R2旧结果保留；full→preview本地四身份序列hours清空|
|7 权益|满足|复用UserMembershipPort.get_user_membership_tier（仓库已按期限解析有效等级）与membership_tier_rank(V6)；身份失败测试preview；前端暂停权益隐藏/裁剪|
|8 付费度量|满足|PAYWALL_SURFACES新增且未知surface仍拒绝；mini仅新增handler/watch；H5fixture新surface曝光去重1次/点击1次、match_id584、原购买路径；旧两个handler/watch及beginPaywallVisit未改|
|9 小程序边界|满足|仅用现有token，未改ALLOWLIST/manifest；组件无api导入，request收口；真实SFC preview无柱，insufficient/unavailable/503/审核隐藏且静默；边界检查通过|
|10 窄屏/时钟|满足自动化，真机待验收|实际H5 320px文档宽320、24柱在283.875图宽内，3高峰；北京时间分钟时钟仅可见full启动，隐藏/卸载清理的SFC断言；详mini报告|
|11 交付隔离|满足|仅本任务具体文件分仓库提交；无生产写入/DDL/迁移/部署/上传；未读Android或暂存env备份；报告明确已测/未测边界|

## 文件清单

### football后端及文档

新增：
- `football_insight_service_backend_rs/docs/best-watch-windows-api.md`
- `football_insight_service_backend_rs/src/ticket_watch/adapters/integration/restock_profile_dto.rs`
- `football_insight_service_backend_rs/src/ticket_watch/adapters/web/best_windows_dto.rs`
- `football_insight_service_backend_rs/src/ticket_watch/application/get_best_watch_windows.rs`
- `football_insight_service_backend_rs/src/ticket_watch/domain/best_watch_windows.rs`
- `football_insight_service_backend_rs/tests/best_windows_local_monitor.rs`

修改：
- `football_insight_service_backend_rs/docs/api.md`
- `football_insight_service_backend_rs/src/activity/domain/paywall.rs`
- `football_insight_service_backend_rs/src/app.rs`
- `football_insight_service_backend_rs/src/ticket_watch/adapters/integration/http_ticket_monitor_port.rs`
- `football_insight_service_backend_rs/src/ticket_watch/adapters/integration/mod.rs`
- `football_insight_service_backend_rs/src/ticket_watch/adapters/web/handlers.rs`
- `football_insight_service_backend_rs/src/ticket_watch/adapters/web/mod.rs`
- `football_insight_service_backend_rs/src/ticket_watch/adapters/web/routes.rs`
- `football_insight_service_backend_rs/src/ticket_watch/application/get_current_ticket_watch_board.rs`
- `football_insight_service_backend_rs/src/ticket_watch/application/mod.rs`
- `football_insight_service_backend_rs/src/ticket_watch/bootstrap.rs`
- `football_insight_service_backend_rs/src/ticket_watch/domain/mod.rs`
- `football_insight_service_backend_rs/src/ticket_watch/domain/ticket_watch.rs`
- `football_insight_service_backend_rs/src/ticket_watch/ports/ticket_monitor_port.rs`
- `FEATURES.md`、`CHANGELOG.md`、`docs/superpowers/specs/2026-10-16-best-watch-windows-design.md`、`docs/superpowers/plans/2026-10-16-best-watch-windows.md`。

另外新增：`docs/best-watch-windows-validation.md`（本报告）。

### 小程序（独立提交）

新增：
- `football_insight_mini/docs/best-watch-windows-validation.md`
- `football_insight_mini/scripts/ticket-best-windows-panel.test.ts`
- `football_insight_mini/src/pages/ticket-watch/bestWindowsHelpers.test.ts`
- `football_insight_mini/src/pages/ticket-watch/bestWindowsHelpers.ts`
- `football_insight_mini/src/pages/ticket-watch/components/TicketBestWindowsPanel.vue`
- `football_insight_mini/src/pages/ticket-watch/useTicketWatchBestWindows.test.ts`
- `football_insight_mini/src/pages/ticket-watch/useTicketWatchBestWindows.ts`

修改：
- `football_insight_mini/src/api/ticketWatch.ts`
- `football_insight_mini/src/pages/ticket-watch/index.vue`
- `football_insight_mini/src/types/paywall.ts`
- `football_insight_mini/src/types/ticketWatch.ts`

删除：无。monitor的具体文件与边界在其仓库报告记录，保留所有既有.env.bak文件。

## 偏离设计及未做验证

- 按用户澄清和review修订设计：座位=block_name，取消更细身份门槛；time,id稳定排序的对账差异已被review接受；新增v2/stat_unit明确隔离。没有新增业务范围。
- 设计第6节位置称“今日监控状态/速览”，实际代码名为TicketRecentRefluxPanel；新面板放其后、库存前。当前数据用例实际为get_current_ticket_watch_board.rs，按现有路径接入。API文档原docs/api.md仅Admin接口，因此新增best-watch-windows-api.md并在api.md链接，避免混用Admin鉴权。
- 默认不运行68项backend/45项monitor ignored测试：部分需要可丢弃数据库/外部服务或写入，只有生产库，不满足隔离条件。本次仅单独显式执行无数据库的best_windows_local_monitor测试；没有真实支付、公众号消息或真实会员/订单创建。
- 未启动完整monitor main，避免初始化现有抢票/通知；未运行生产离线命令或安装timer，实测统计用只读回滚脚本。
- 无微信开发者工具/真机环境，不能证明真实MP视觉；已经验证真实H5构建/320窄屏与离线MP组件注册。`@tap`沿用uni编译，面板无新增scroll-view，24柱flex收缩；时钟使用组件生命周期清理，CSS变量用既有token，两端实际设备效果待用户验收。
- 未部署生产、未上传小程序；生产未生成新快照，因此线上暂时不会出现面板。

## 后续上线顺序（仅清单，未执行）

1. 用户审查后部署monitor读取端与离线二进制；设置TK_RESTOCK_PROFILE_DIR为持久目录，并让在线容器与离线作业共享该目录。不要把批处理挂进抢票循环。
2. 避开售票窗口，用户执行rebuild-restock-profiles（首次生成，重算用--force）；确认最近场次v2快照与24桶/起点匹配，再按需安装启用仓库systemd service/timer。失败逐场继续、最终退出失败，保留已有文件。
3. 部署football后端，确认后台聚合及真实V6/普通用户的full/preview，404不足与故障保留符合约定。**无需任何数据库迁移或INSERT。**
4. 从干净同步main按既有发版流程上传小程序并真机验收，再按用户指令提审/发布；本任务未申请或手改版本。
