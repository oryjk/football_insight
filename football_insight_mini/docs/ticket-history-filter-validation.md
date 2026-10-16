# 历史双 tab 统一筛选与无票价区域过滤验证

2026-10-16，用户要求历史统计排除的比赛也从历史比赛隐藏，两个 tab 必须复用同一套规则；补充隐藏没有票价的区域，并明确保留自己的水印改动、允许上传。只修改小程序，无后端/API/数据库写入，未部署后端。

## 实现与边界

- ticketWatchLoadHelpers.ts 的 selectHistoryMatchesWithData 是展示筛选的唯一入口：复用 selectRefluxStatsMatches 的一线队主客队精确匹配和 include_in_reflux_stats != false，再要求回流总量为有限正数。保留云南分支参数，不硬编码另一套球队判断。
- visibleHistoryMatches 使用该函数；历史比赛选择栏和 displayedHistoryMatch 读取它，历史统计也使用这一 computed。独立调用的 buildHistoryRefluxTrend 同样复用该函数，没有复制过滤表达式。原始 historyMatches/historyStatsMatches 仅保留为加载候选，不会因为未加载项被隐藏而阻断数据拉取。
- 无可见项时显示加载或无数据提示。syncVisibleHistorySelection 仅在页面可见、历史比赛 tab、历史列表/趋势读取完成、已加载且非审核/权益锁定时，才将失效选择切到首场可见比赛；保留有效选择，调用原 handleHistoryMatchSelect，沿用请求 owner/revision 防护。
- 两球队的初选共用 syncInitialHistorySelection，先排除球队/统计标记不合格的比赛，再沿用原列表与用户选择共享 owner。全部无候选时清空并失效旧选择，避免被排除首场缺开售时间导致有效历史加载失败。不放宽 sale_start_at + 10 分钟约束。
- 保留最近 12 场自动预取与每批 3 场的既有范围，更早未加载的场次在两个 tab 都不显示，本次没有扩大读取范围或新增接口。
- groupInventoryByPrice 在唯一分组入口跳过空、非有限数值、零和负票价；当前/历史及两球队都复用它，区域卡片与其派生价位/区域汇总不会包含这些区域。只读核实生产 regions 的 VIP2 price 为空，VIP1/3 为 1288.00，后者保留；不按 VIP 名称拉黑，不改价格配置、原始库存或已保存关注记录。历史趋势仍沿用原始回流总量计算，本次只过滤无票价区域展示。
- 用户的 35469bf 原生查询 loading 与 a3e60cf 昵称水印保留；经用户授权，将水印绑定放回 TicketMatchIdSheet 属性内。只同步旧 SFC 测试以适应新的加载交互，未改用户组件或 token 实现；断言昵称水印不污染复制的业务 ID。

## TDD、原用例与复审

- 共享筛选/自动选择 RED：10 pass/4 fail，明确原始列表包含 B 队、无关队、排除标记及零值，且未自动选有效场次。被排除首场缺 sale_start_at 的两个用例另行观察到失败，再修正初选。
- 无票价 RED：1 pass/8 fail；涵盖空串、空白、零、零小数、负数、非数字、NaN 和 Infinity。修正后 9 项通过，有价 VIP 和正常小数票价保留。
- 定向 helpers/effects/historySelection/真实 SFC 共 179 pass/0 fail。原 6 个选择竞态用例不删除、不缩减断言；此前用 statsOff 禁预取的 fixture 改为合法比赛及已缓存趋势，以继续隔离后台读取，并符合新增可见规则。
- 首次全量为 824 pass/1 fail：共享 main 的用户原生 loading 改动使旧测试仍期望弹层加载文案。按新交互更新宿主 uni.showLoading/hideLoading 与断言后全量通过；没有还原用户改动。
- 独立只读复审通过，包括共享口径、候选与展示区分、初选、旧请求边界、无票价区域及用户水印接线。没有重复运行数据库或真实购买测试。

## 最终真实输出

所有命令退出码 0，原始输出在会话 scratch `.tmp/history-tabs-shared-filter/`。

| 命令 | 实际输出 |
|---|---|
| bun test | 834 pass / 0 fail / 5655 expect() calls，70 files，13.15s |
| bun run type-check | vue-tsc --noEmit，退出 0 |
| bun run check:boundaries | 依赖边界检查通过（组件↛api/页面、页面不互引、api 依赖受限、uni.request 收口） |
| bun run build:h5 | DONE Build complete. |
| MINI_REVIEW_SKIP=1 bun run build:mp-weixin | DONE Build complete.，组件注册与生产 API 校验通过，无版本登记/上传 |
| bun run verify:mp-components | All mini-program custom components are registered. |
| git diff --check | 无输出，退出 0 |

构建仍有现存 uni-app 更新及 Browserslist 旧数据提示。ticket-watch 下 gradient/hex 搜索无结果，本次未修改 token、ALLOWLIST 或手动版本号。真实微信/H5 页面视觉未人工验证，后续按用户授权上传测试版供复测。

## 文件与跨端说明

- 新增：src/pages/ticket-watch/historySelection.test.ts、ticketWatchHistorySelection.ts，当前 docs/ticket-history-filter-validation.md。
- 修改：src/pages/ticket-watch/index.vue、useTicketWatchState.ts、refluxHelpers.ts、ticketWatchLoadHelpers.ts、ticketWatchHistoryBoard.ts、inventoryHelpers.ts、effects.test.ts、helpers.test.ts，scripts/ticket-match-id-sheet.test.ts，根 CHANGELOG.md、FEATURES.md。
- 无删除文件，后端不变。原 @tap/@click、历史横向 scroll-view、居中滚动与 hover 行为保留；新增 watch 由页面生命周期释放，无新计时器或样式。用户水印 CSS 变量保留，组件层不调用业务 API。

## 用户授权后的实际上传

用户确认水印是自己所加并授权上传。功能提交 006c9f4 先推送 main；统一 mp:release 登记库分配 1.0.85，首次按 Git 门禁在版本文件生成后暂停（退出 1，无构建/上传），版本文件单独提交为 060de1f 并推送后重跑。

真实命令 `bun run mp:release -- --robot 2 --desc '历史比赛与统计统一筛选，隐藏无票价区域，保留昵称水印'` 最终退出 0。源为干净且同步的 main 060de1f，保留用户原生 loading/水印提交。Git 门禁、依赖边界、生产构建、组件注册与生产 API 校验通过；微信 SDK 确认版本 1.0.85 robot=2 上传完成，完整包 2,090,729 字节。登记库回读 version_code=10085、is_reviewing=false；只是测试版本上传，不代表正式微信审核/发布。

原始输出位于会话 scratch `.tmp/mini-upload-history-filter/allocate.log`、`release.log`。本次没有后端部署/重启、迁移或其他生产数据操作；真机效果由用户体验版验收。前文“待上传”为实现阶段状态，以本节后续上传结果为准。
