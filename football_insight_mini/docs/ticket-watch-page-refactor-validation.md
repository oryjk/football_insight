# 回流看板页重构交付与验证

日期：2026-10-16。设计：`docs/superpowers/specs/2026-10-16-ticket-watch-page-refactor-design.md`。

## 范围与状态

任务 A 已先独立提交并推送（后端 `232d9d4`、小程序 `74a2504`，北京时间 16:21），本报告仅记录随后执行的任务 B。任务 B 不改后端或接口，没有数据库操作、迁移、生产部署或小程序上传，没有版本或边界 ALLOWLIST 改动。

本地后端在任务 A 已重启并健康；任务 B 没有后端改动、没有再次重启。生产未部署、未重启。代码按两个可独立回退的提交组织：首屏与刷新状态（连同其纯函数/状态），其余卡片纯样式。没有把任务 A 的改动混入任务 B。

## 设计第 10 节自查

| 项 | 结果 | 依据 |
|---|---|---|
| 1. 删除 hero/球队切换/会员胶囊，共享三段图标切换 | 满足 | Header 只收 activeTab、emit tab，使用 FiSegmented；当前卡移除 membershipLabel/crown；六张 24×24 Lucide 灰/白 SVG 几何与官方固定版本逐项一致 |
| 2. 四种状态、定稿文案和纯函数边界 | 满足 | idle → refreshing → failing → live 优先级；33 项表驱动纯函数用例，真实 SFC 测试检查文字；无数据只显示尚未更新，不叠加“更新” |
| 3. 成功时间、静默失败与重置 | 满足 | 仅现有成功分支写 Date.now；失败只在 loaded poll 为真；保留对象/时间/原 message，不写错误卡；initial 原错误行为保留；新增 9 项 effects 覆盖恢复和会话 reset/迟到结果 |
| 4. 组件内部时钟与清理 | 满足 | TicketRefreshStatus 本地 now 与 timer；监控 1000ms、未监控 15000ms；ticking=false 清理，恢复即校准，卸载清理；真实 SFC timer 测试通过；freshnessNowMs 与轮询文件未改 |
| 5. 行为与度量保持 | 满足 | 三个付费函数、两个 watch 整段逐字保留，beginPaywallVisit/两处 upgrade 绑定原样；轮询、订阅、比赛 ID、sessionSync 文件逐字未改；九个既有卡片的 template/script 逐字未改。关注区块仅按设计移动位置，折叠 key/事件不变 |
| 6. 成都界面、保留云南逻辑 | 满足 | 删除页面 switchTeam/绑定；selectedTeam 初值仍成都且页面无写入口；TicketWatchTeam、Yukun 两套加载分支及其原测试保留。没有原测试通过页面 switchTeam 驱动，无需改写旧用例 |
| 7. 目录无渐变/hex，控制 token 数量 | 满足 | 原 39 处渐变改为纯色，交付前目录全文扫描零匹配；UI hex 扫描零匹配；fi-tokens.css 完全未改，新增 token 为零；SVG 颜色按设计明确例外 |
| 8. 骨架结构与共享类 | 满足，导航占位有说明 | 使用全局 fi-skeleton-*；标题/状态/双指标/整行主按钮/双次按钮。骨架支持三段占位，但页面保留真实可操作导航，传 showTabs=false 避免重复，见下方偏离说明 |
| 9. 按钮层级与事件 | 满足 | 监控按钮整行，停止监控保留次要语义；订阅与比赛 ID 两个描边按钮并排，已开通保留状态样式；monitor/subscribe/match-id 事件不变 |
| 10. 工程边界与验证 | 满足 | 无页面互引、无组件 API import、无 ALLOWLIST/manifest 修改；四项规定验证及双端离线构建结果见下表 |

## TDD 与回归证据

- 新格式化/状态纯函数测试先因缺少导出失败，随后 33 项通过：间隔 1/15/59/60/90/120/600、无效值、小数秒；null/未来/5 秒/60 秒/1 小时边界；全部八个布尔组合。
- 新展示状态 9 项 effects 先全部因缺少 Ref 失败，随后通过；原 59 项 effects 用例保留，仅补 Date.now 的 afterEach 恢复。断言 poll 失败仍是同一旧 match/sections 对象、旧 loadedAt、空 currentErrorMessage，并验证恢复、initial 错误、账号替换后的迟到成功/失败。
- 刷新状态 SFC 测试先有 4 项失败；真实编译、Vue renderer 与受控时钟验证 1 秒→15 秒切换、隐藏/显示、卸载、重试优先级、文案。补无效时间断言曾复现“尚未更新 更新”，修复后通过。
- 骨架 SFC 测试先复现 showTabs=false 仍有三段占位（Expected 0 / Received 3）；新增可选 prop 后两项通过，主比赛卡骨架仍保留。
- 纯样式未新增与 CSS 实现逐字对应的机械测试；使用既有功能回归、CSS 解析/token 检查、九卡 template/script 字节比较与双端编译验证，真实视觉仍待人工验收。

## 逐组件结构、token 与参考出处

| 组件 | 结构及改动 | token 与参照 |
|---|---|---|
| TicketWatchHeader | 一个真实 view 纵向 stretch 包含 FiSegmented；三段/六图标，仅 activeTab/tab | FiSegmented 自身的 color-segmented-*/space-6、10、20/font-26/radius-round/icon-sm/weight-bold；直接复用共享组件，不 import 页面 |
| TicketCurrentMatchCard | FiPanel 外壳；球+队名与右侧轮次日期；状态条；两个指标；一主两次；原 message | component-panel-*；color-page/text-strong/text-muted/border-chip/surface；space-10/12/16/18/20/78、font-20/22/26/32/44、leading、weight、radius-md、icon-sm/md、shadow-soft。参考 FiPanel、FiSegmented 的克制纯色与原按钮状态语义 |
| TicketRefreshStatus | 左圆点+状态、右拉取年龄，允许小屏换行；每实例自持 timer，无 emit | color-page/text-secondary/text-muted/primary/success-accent；space-8/10/12/14/16、font-20、weight-medium、leading-normal、radius-md、opacity-pressed；沿用组件壳 token；prefers-reduced-motion 关闭脉冲 |
| TicketBoardSkeleton | 三种 mode；共享形状、纯色 block，禁用 sweep；当前为标题/状态/双指标/一主两次 | 全局 fi-skeleton-*，color-skeleton-end/page、space/icon/radius tokens；参考 styles/fi-skeleton.css 与 InsightsSkeleton.vue，不跨页面 import。showTabs 可选，默认 true |
| TicketRecentRefluxPanel | 结构/分桶/权限/事件不改；各档位改成独立纯色 | 复用 ticket-recent-reflux-bucket-within*、price-tone、space/font/radius/weight；参考原档位强调颜色取主色，不改分桶语义 |
| TicketRecentRefluxLock | 结构与原锁定预览不变，遮罩和预览纯色 | color-page/text-primary、原 preview/bar/decoration、leading-none；参考原锁定态，未增加升级入口 |
| TicketFocusPanel | 热区与折叠不变，只改底色/图标 | 原 focus/price-tone、icon-sm、space/font/radius；参考原热区等级颜色与共享标题 |
| TicketTrackedInterests | 等待/命中结构与折叠不变，页面有内容时放在最近回流之前，否则保持热区之后 | 原 tracked-interest 色与共享字体/间距，icon-sm；父页面负责布局，子组件模板未改 |
| TicketInventoryPanel | 原价位/库存/关注事件/折叠完全保留，热度改纯色 | 原 price-tone RGB 及原透明度，border/shadow/space/font；参考原价位热度级别，无新业务色 |
| TicketSupplyInsights | 原供给指标/决策与两种折叠保留 | 原 insight 色、space/font/leading；面板与标题沿用共享契约 |
| TicketHistoryTrend | 原图表/选择/文案不动，强调条取原深色端 | 原 bar 深色端/selected-row、space/font/weight；参考原图表，不更改数值/宽度计算 |
| TicketHistorySelector | 原滚动选择结构不动，尺寸用现有刻度组合 | 原 chip 色/字体、space、radius；原 scroll-view 与选择事件保留 |
| TicketHistoryMatchCard | 原已 token 化，文件未改，继承共享外观 | 原字体/边框/色彩，公共 panel/title token |
| ticket-watch-shared.css / 页面 | 统一通用面板、标题，页面遮罩改单色半透明；不改页面导航/分享/进入遮罩语义 | component-panel-* 来自 FiPanel.vue，component-kicker/title-* 来自 FiSectionHeading.vue；保留紧凑 font-40 标题，page-bg-fade 用现有半透明 token |

公共 CSS 也被 TicketMembershipLock 引入，因此会员暂停提示卡继承新的卡片外观/标题样式；锁定条件、文案、按钮/事件完全不动。订阅与兴趣确认弹层虽然 import 公共 CSS，模板不匹配本次改动选择器，实际不受这些样式变化影响；比赛 ID 弹层不 import 它。

## 图标核对

采用 Lucide 官方仓库固定版本 0.468.0 的 [activity](https://raw.githubusercontent.com/lucide-icons/lucide/0.468.0/icons/activity.svg)、[history](https://raw.githubusercontent.com/lucide-icons/lucide/0.468.0/icons/history.svg)、[chart-column](https://raw.githubusercontent.com/lucide-icons/lucide/0.468.0/icons/chart-column.svg)。下载后用 XML 逐项对比所有几何节点/属性：六个变体均与官方一致，24×24、stroke-width=2、round 端点/连接，idle 为 #7f8490，active 为 #ffffff，保留 Lucide 版权注释。未引入图标库依赖。

## 偏离与原因

1. 保留真实 FiSegmented 在加载/会员锁定时可操作。骨架提供 showTabs 占位，页面传 false：如果为避免重复占位而隐藏真实 Header，currentLoading 在权益锁定路径可能一直为 true，用户将无法切换历史；这违反行为不变的硬约束。审查发现后已修复并有骨架 SFC 回归；报告不把不可点击占位称作真实导航。
2. 纯函数对非有限时间防御返回尚未更新，小数间隔向下取整，避免 NaN 文案；所有设计给定整数与时间边界完全一致。
3. 真实 SFC runtime 测试放 scripts/，沿用既有 membership-catalog-render.test.ts：应用 tsconfig 的最小 bun:test 声明不含 spyOn/node 类型，测试仍由 bun test 自动运行，未扩大 tsconfig、依赖或边界例外。
4. 将纯函数/状态与其首屏消费方放在同一个功能提交，其余卡片纯 CSS 单独提交；这样回退任一提交都不会留下缺失导出或 props 的依赖。
5. 设计引用的文件路径均存在。helpers.ts 实际是六个页面纯函数模块的 re-export（只有 6 行），新增函数按设计允许放 presentation.ts。没有测试调用已删除的页面 switchTeam，旧 Yukun 测试无须修改。

## 未做测试与原因

- 未做 H5 浏览器/微信开发者工具或真机视觉验收，无法仅凭终端确认卡片换行、rpx 缩放、图标、滚动手势和 reduced-motion 的屏幕效果；单测与构建不等同视觉通过。
- 未连接生产接口加载真实账户，不真实支付、不开订阅、不发送公众号消息；不执行后端验证或 SQL（任务 B 无后端/数据改动）。
- 未新增纯 CSS 快照或实现镜像测试；样式审查依据结构不变、token 检查、官方图标比对和双端产物，仍需用户视觉测试。

## 验证命令实际输出

均在 football_insight_mini 下真实执行，退出码均为 0：

| 命令 | 实际输出 |
|---|---|
| `bun test` | `781 pass` / `0 fail` / `5489 expect() calls` / `Ran 781 tests across 67 files. [13.02s]` |
| `bun run type-check` | `$ vue-tsc --noEmit`，退出 0 |
| `bun run check:boundaries` | `依赖边界检查通过（组件↛api/页面、页面不互引、api 依赖受限、uni.request 收口）` |
| `bun run verify:mp-components` | `All mini-program custom components are registered.` |
| `bun run build:h5` | `DONE Build complete.` |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | `DONE Build complete.`，组件全部注册、编译包 API 校验通过；manifest 保持 1.0.80，无版本登记、无上传 |

另在会话 scratch 内从 74a2504 的 git archive 构造“仅首屏功能”和“仅其余卡片 CSS”两个快照，分别叠加各自文件、不使用 stash/还原工作区；两份均 `bun run type-check` 退出 0，确认两个提交可独立回退而不缺导出/props。

两端构建保留现有 uni-app 新版本与 Browserslist 六个月旧数据提示。全量测试保留原 matches 失败 fixture 的 warn 日志，不是失败用例。原始 stdout/stderr 在会话忽略目录 `.tmp/ticket-watch-refactor/`：bun-test.log、type-check.log、boundaries.log、components.log、build-h5.log、build-mp.log，不提交。

## H5 与微信跨端说明

- FiSegmented/按钮使用 @tap，不双绑 @click；共享 FiSegmented 的 hover-class 保留，原其他组件事件处理逐字不变。
- Header 与 FiPanel 外各有真实 view，flex-direction:column + align-items:stretch，避免微信自定义组件宿主缩为内容宽；不依赖父 scoped 样式穿透组件根节点。
- 原历史 selector 的 scroll-view 与滚动事件保持原样；未把可折叠内容换成新的滚动容器。双按钮使用真实 view 的 grid，内容布局不依赖组件宿主 class。
- Status 自己每秒/15 秒刷新本地 now，只重渲染该组件。页面 onShow/onHide/onUnload 同步 isPageVisible，既有 pageVisible 与 freshnessNowMs 的用途/15 秒时钟不动；后台停止组件 timer，回前台按 Date.now 校准。
- CSS 变量在 App.vue 全局导入，全部使用既有 token；不新增 px/rem/vw 固定样式。骨架局部覆盖共享填充为纯色，禁用扫光，不影响其他页面。
- 状态脉冲使用 prefers-reduced-motion 取消动画。H5 与微信实际设置效果仍待人工核验，不声称真机已验证。

## 文件清单


- 新增：`football_insight_mini/docs/ticket-watch-page-refactor-validation.md`
- 新增：`football_insight_mini/scripts/ticket-board-skeleton.test.ts`
- 新增：`football_insight_mini/scripts/ticket-refresh-status.test.ts`
- 新增：`football_insight_mini/src/pages/ticket-watch/components/TicketRefreshStatus.vue`
- 新增：`football_insight_mini/src/pages/ticket-watch/presentation.test.ts`
- 新增：`football_insight_mini/src/static/ticket-watch/activity-active.svg`
- 新增：`football_insight_mini/src/static/ticket-watch/activity-idle.svg`
- 新增：`football_insight_mini/src/static/ticket-watch/chart-column-active.svg`
- 新增：`football_insight_mini/src/static/ticket-watch/chart-column-idle.svg`
- 新增：`football_insight_mini/src/static/ticket-watch/history-active.svg`
- 新增：`football_insight_mini/src/static/ticket-watch/history-idle.svg`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketBoardSkeleton.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketCurrentMatchCard.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketFocusPanel.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketHistorySelector.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketHistoryTrend.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketInventoryPanel.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketRecentRefluxLock.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketRecentRefluxPanel.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketSupplyInsights.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketTrackedInterests.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/components/TicketWatchHeader.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/effects.test.ts`
- 修改：`football_insight_mini/src/pages/ticket-watch/index.vue`
- 修改：`football_insight_mini/src/pages/ticket-watch/presentation.ts`
- 修改：`football_insight_mini/src/pages/ticket-watch/ticket-watch-shared.css`
- 修改：`football_insight_mini/src/pages/ticket-watch/ticketWatchCurrentBoard.ts`
- 修改：`football_insight_mini/src/pages/ticket-watch/useTicketWatchState.ts`
- 修改：`CHANGELOG.md`、`FEATURES.md`（任务 B 自己的条目与原因）。
- 新增：`docs/superpowers/plans/2026-10-16-ticket-watch-page-refactor.md`。
- 无删除文件；fi-tokens.css、manifest、ALLOWLIST 未改。

## 后续修正：历史走势默认高度

仅修改 index.vue 的历史统计布局变量和 TicketHistoryTrend.vue 的高度/flex 样式。页面使用现有导航、间距和 FiSegmented 尺寸 token 计算可用高度；组件使用明确高度，不依赖小程序自定义组件宿主的百分比高度。删除原 620rpx max-height，标题/摘要不收缩，scroll-view 使用 height:0 + flex:1 占满剩余高度。无业务脚本、接口、统计或点击行为改动，新增 token 为零。修正后重新执行：bun test 781 pass/0 fail，type-check、check:boundaries、verify:mp-components、H5 与离线微信构建均退出 0；日志在同一会话目录。

用真实源样式和现有 token 构造 Chromium 布局夹具，对比原版与修改版：375×812 的 18 行样例，卡片底部留白从约 253px 减为原页面预留的 18px；仅一行也保持卡片填满。320×568、430×932 与各自 1/18 行共六种情况均通过，长列表的 scrollHeight 大于 clientHeight，仍在卡内滚动。此验证是浏览器布局夹具，不冒充微信真机或完整 H5 页面验收；脚本位于会话 scratch `.tmp/ticket-history-height/check-layout.cjs`。

## 后续要求：统计只纳入成都蓉城一线队

这是用户在重构完成后新增的统计口径要求，单独提交。后端票务接口是通用赛程，HTTP adapter 还供比赛 ID 存在性查询使用，因此不在适配器中过滤赛程。仅在小程序统计样本入口 selectRefluxStatsMatches 中，将既有 include_in_reflux_stats !== false 与主/客队全名精确相等条件取 AND；默认目标成都蓉城，保留休眠云南分支显式传云南玉昆。

修改 ticketWatchLoadHelpers.ts、useTicketWatchState.ts 与 ticketWatchHistoryBoard.ts，图表样本与趋势预取一致：排除“成都蓉城希拉谷”和无关球队，不使用 includes 模糊匹配。历史比赛列表/详情仍完整；即便用户查看其他队详情产生 totals 缓存，也不会进入趋势点、场均、最高值、场次数。后端/API、原可靠性排除标记、开售十分钟口径、加载/会话/请求 owner 均不变。

helpers.test.ts 新增 10 项表驱动覆盖主客场、B 队、无关球队、可靠性开关及云南分支；effects.test.ts 新增两种球队的真实 boards/state 集成测试，断言只预取合格样本、历史列表保持完整，并核对均值/最高值/场次数。RED：旧逻辑 4 pass/8 fail，明确错误预取 B 队/无关队；GREEN：相关两文件合计 141 pass/0 fail。独立只读复审通过，无生产数据读取或写入。

最终验证：bun test 793 pass/0 fail（67 files，5515 expect calls）；type-check、check:boundaries、verify:mp-components、H5 与离线微信构建全部退出 0。原始日志在会话 scratch `.tmp/chengdu-reflux-stats/`，没有版本登记、上传或部署。

## 后续用户指令：上传测试版

2026-10-16 按用户“上传一下”指令，使用统一 mp:release 流程上传 1.0.82 robot 2。登记库自动分配版本，生成文件单独提交为 baec7a5 并推送；随后从该干净、同步的 main 构建并上传。Git 门禁、依赖边界、组件注册及生产 API 校验通过，微信 SDK 确认完整包 2,081,073 字节；登记库回读 is_reviewing=false，保持测试展示状态。发布链路回归 30 pass/0 fail，真实 mp:release 退出 0。原始输出留在会话 scratch `.tmp/mini-upload-1.0.82/release.log`。

本次只上传小程序，没有后端部署/重启或迁移；前文“未上传”描述为各实现阶段当时状态，此处为后续发布状态。用户可测试历史走势默认高度与一线队统计筛选。

## 后续要求：历史统计隐藏无数据比赛（待上传）

用户要求从历史统计列表移除无数据比赛。buildHistoryRefluxTrend 只保留已加载、有限且大于 0 的回流总量；零值、缺失、负数、NaN 和无穷值均排除，列表、场均、最高值、最近值和场次数共用同一批样本。保留一线队精确匹配与服务端排除规则；历史详情列表与统计候选预取不变，避免先过滤未知总量导致比赛永远无法加载。既有自动预取最近 12 场的上限未扩改，更早比赛需要先查看历史详情加载后才有机会进入走势。

TicketHistoryTrend 在全空时提示“暂无有回流数据的历史比赛。”，拉取中提示“正在加载历史回流数据…”。仅新增 token 样式：space-24、color-text-muted、font-24、weight-medium、leading-normal，保留原撑满高度和 scroll-view。无新增事件、计时器或端差异；实际 SFC SSR 验证空态、加载态与非空列表，H5/微信真机视觉未测试。没有后端/API、数据库、manifest 或 ALLOWLIST 改动，未上传、未部署。

TDD：统计与 boards/state 回归先得到 1 pass/10 fail；真实 SFC 渲染测试先得到 1 pass/2 fail。实现后相关三个文件合计 150 pass/0 fail，新增 SFC 三项单独复跑 3 pass/0 fail。集成用例证明一线队零值比赛仍被请求但不进入图表/计数；独立只读复审未发现筛选错误或拉取死锁。

最终真实输出（各命令退出码均为 0），日志在会话 scratch `.tmp/reflux-stats-with-data/`：

| 命令 | 输出结论 |
|---|---|
| `bun test` | `802 pass` / `0 fail` / `5562 expect() calls` / `Ran 802 tests across 68 files. [13.07s]` |
| `bun run type-check` | `$ vue-tsc --noEmit`，退出 0 |
| `bun run check:boundaries` | `依赖边界检查通过（组件↛api/页面、页面不互引、api 依赖受限、uni.request 收口）` |
| `bun run build:h5` | `DONE Build complete.` |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | `DONE Build complete.`，组件全部注册、生产 API 校验通过；未分配版本或上传 |
| `bun run verify:mp-components` | `All mini-program custom components are registered.` |

双端构建仍有现存 uni-app 更新和 Browserslist 旧数据提示，不影响退出结果。新增 scripts/ticket-history-trend-render.test.ts；修改 refluxHelpers.ts、helpers.test.ts、effects.test.ts、components/TicketHistoryTrend.vue、当前验证报告、根 CHANGELOG.md 和 FEATURES.md，无删除文件。
