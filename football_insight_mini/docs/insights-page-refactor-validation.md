# 洞察页重构交付与验证

日期：2026-10-16。基线：`9018535`，直接在 main 完成。
设计：`docs/superpowers/specs/2026-10-16-insights-page-refactor-design.md`。

## 范围与状态

仅修改小程序展示层、必要 token，以及用户指定的根目录 CHANGELOG.md / FEATURES.md。没有修改后端、接口、其他页面、依赖边界脚本或版本；没有读取、搜索或修改 Android 项目。

本次未重启本地后端；生产后端未部署、未重启；小程序未上传。离线构建通过 MINI_REVIEW_SKIP=1 保留已有版本，没有登记或上传。

## 改动文件清单

以下路径除两份根文档外，均相对 `football_insight_mini/`。

新增：

- `src/pages/insights/components/InsightsTeamPanel.vue`
- `src/pages/insights/components/InsightsContributionSurface.vue`
- `src/pages/insights/components/InsightsSkeleton.vue`
- `docs/insights-page-refactor-validation.md`

修改：

- `src/pages/insights/components/InsightsTicketWatchEntry.vue`
- `src/pages/insights/components/InsightsTeamSelector.vue`
- `src/pages/insights/components/InsightsSummaryPanel.vue`
- `src/pages/insights/components/InsightsLockedPanel.vue`
- `src/pages/insights/helpers.ts`
- `src/pages/insights/helpers.test.ts`
- `src/pages/insights/index.vue`
- `src/styles/fi-tokens.css`
- 根目录 `CHANGELOG.md`
- 根目录 `FEATURES.md`

删除：

- `src/pages/insights/components/InsightsHeroCard.vue`
- `src/pages/insights/components/InsightsTeamBoardEntry.vue`
- `src/pages/insights/components/InsightsContributionPanel.vue`
- `src/static/user/member-card-dots.png`

删除图片前已搜索运行时代码及脚本：除旧 Hero 外无使用；删除后无残留运行时引用。仓库设计文档里对旧文件的历史说明保留。

## 每个组件的结构、token 与参考出处

所有运行时组件从本页 components 或已有 Fi 组件直接导入；没有 import rankings 页面。色值新增到 fi-tokens.css primitive 后建立 semantic 别名，页面没有直接引用 primitive。

| 组件 | 结构与布局 | 使用的主要 token | 榜赛页参考出处 |
|---|---|---|---|
| InsightsTicketWatchEntry | FiPanel flush 内单行：64 图标块、36 ticket SVG、标题与单行说明、登录胶囊或 36 箭头；整行 tap/open，按压不透明度 0.7；无 veil | FiPanel 的 component-panel 契约对应 surface-panel/radius-xl/border-card/shadow-card；space-64/36/16/28/20，font-32/24/22，weight-extrabold，leading-snug/normal/none；color-insights-entry-icon、radius-insights-icon、opacity-insights-pressed | RankingsStandingsLauncher.vue 的 .panel、.standings-launcher-card__header / __icon / __icon-img / __title / __chevron / --pressed；未复制其散落色值 |
| InsightsTeamPanel | 单张 FiPanel；真实 view 内纵向 stretch，球队选择器在上、摘要在下，间隔 22；转发 selectedTeamId 与 open-board | component-panel-*、space-22 | RankingsSurfacePanel.vue 的 .ranking-surface 分区间隔；遵循设计系统父级 wrapper 布局责任 |
| InsightsTeamSelector | 无卡片外壳和标题；保留原 scroll-view、scroll-left、scroll-with-animation、按钮 ID 和居中脚本；36 头像；选中无上移/阴影 | space-10/4/12/18/22/36，font-24/22，leading-none，radius-round，color-page/text-secondary/text-primary/surface/primary，weight-extrabold/bold；opacity-insights-pressed | RankingsSurfacePanel.vue 的 .pill-row / __list / __item：浅底、墨色选中与白字 |
| InsightsSummaryPanel | 无外壳；队名、名次胶囊和战术板同一 flex 行；长队名 min-width:0 省略，名次/链接不收缩；32 箭头；两格总进球/失球；保留 useAnimatedInteger、原 key 和 fi-value-pop | font-36/26/22/24/60，space-10/8/12/32/64/18/20，color-page/text-strong/text-secondary/text-muted/primary，radius-md/round，weight-extrabold/bold，leading-snug/none/normal/insights-value | RankingsStandingsLauncher.vue 的 __header / __name 截断 / __chevron，与 podium 浅底块的布局语言；摘要指标按设计 3.4 实现 |
| InsightsContributionSurface | FiPanel 内三段 scope-toggle、进球专属二级 pill-row、一句说明、key=viewKey 的列表与更多/收起；本地 tab/dimension，不重置父级四个 expanded；空态暂无数据 | border-width/color-border-chip/radius-round；space-6/64/20；font-26/24/22；color-insights-tab-idle/text-primary/surface/page/text-muted/text-caption；48 头像、14 进度条；color-insights-track/assist、color-primary；weight-bold/extrabold | RankingsSurfacePanel.vue 的 .scope-toggle / __thumb / __button 和 .pill-row / __item；改为设计要求的三段宽度及位移公式；列表沿用原 ContributionPanel 行数据与结构 |
| InsightsSkeleton | role=status、aria-label=洞察加载中；第一卡胶囊、队名行与两个指标；第二卡三段、说明和 3 行；全部由真实 view 排列 | 全局 fi-skeleton-*；space-card-gap/22/10/72/32/12/20/18/56/6/64/48/28/14，radius-md/round、border-width/color-border-chip、color-page；扫光与底色来自现有全局 skeleton 样式 | RankingsSkeleton.vue 的 fi-skeleton-stack、panel、heading、block、copy 与 controls；只复用已有样式，不 import 其组件 |
| InsightsLockedPanel | 模板、文案、script 和 action 事件原样；仅迁移样式，指标浅红纯色、墨色进度条和纯色遮罩 | color-insights-preview-metric/preview-overlay/preview-surface/lock-surface/track，color-text-primary/surface/secondary/caption/page；space-insights-preview-height/column/kicker-spacing，font-56/44，radius-insights-lock，shadow-insights-lock；间距和模糊均引用 space token | 保留本页原预览结构；卡片外观继续用 FiPanel，与榜赛页 .panel 对齐 |

滑块仍是 300ms cubic-bezier(0.3,0.9,0.3,1)：宽度 calc((100% - space-6 * 4) / 3)，三位置为 0、calc(100% + space-6)、calc(200% + space-6 * 2)。列表复用 220ms fi-rise-in，数字维持 420ms fi-value-pop。H5 reduced-motion 媒体查询关闭组件内非必要位移/过渡及骨架扫光；未修改全局动画或数字计数算法。

## 设计第 10 节逐项审查

| # | 结果 | 依据 |
|---|---|---|
| 1 删除冗余内容 | 满足 | 3 个旧组件与页面引用删除；球队列表标题和计数删除；原四块改为一张 Surface |
| 2 无渐变/散落值 | 满足 | src/pages/insights 下 gradient、hex、rgba、原始 rpx、primitive 引用搜索均无结果；75 个使用的 token 都能解析到定义 |
| 3 入口契约与度量 | 满足 | locked:boolean / emit(open) 保留；页面整个入口标签字节一致；从 navigationPageStyle 起的完整 script 字节一致，包含全部付费调用 |
| 4 会话与状态 | 满足 | useInsightsData.ts、sessionEffects.ts、sessionEffects.test.ts 字节未改；页面条件表达式及其顺序与基线一致，仅替换两个加载组件 |
| 5 贡献切换 | 满足 | 三段/进球双维度/定稿说明/每视图展开/空态已实现；真实 SFC 冒烟验证切换、助攻失球隐藏二级行、切回保留展开和维度 |
| 6 骨架 | 满足 | 两处 FiLoading 替换为 InsightsSkeleton；使用全局 fi-skeleton-*，本地无自定义渐变 |
| 7 边界/删除引用 | 满足 | check:boundaries 通过，ALLOWLIST 未改；删组件/图片无运行时引用；无跨页面或 api 组件依赖 |
| 8 helpers 与原测试 | 满足 | 新增 3 项覆盖固定选项、6 个映射分支及精确文案/长度/互异；原 7 项 helpers 用例和 session 用例保留；全量 710 通过 |
| 9 图标尺寸 | 满足 | ticket 容器64、内部图标36、入口箭头36、战术板箭头32、球队头像36、贡献头像48，全部用 space token |
| 10 小屏布局 | 满足实现；屏幕验收待完成 | 横向 scroll-view 与居中逻辑未改；摘要长名省略、名次和战术板不收缩；真实 wrapper 使用 stretch，避免 MP 宿主收缩。尚无小屏浏览器截图/微信真机证据 |

独立只读 review 未发现阻塞或重要问题，另外核验了微信产物的事件映射、组件注册与宿主布局边界。该审查没有代替屏幕视觉验收。

## TDD 与真实命令结论

helpers 先新增测试，运行失败：缺少 resolveContributionViewKey 导出，Bun 返回 1 fail / 1 error；随后实现类型、常量、映射，定向测试变为 20 pass / 0 fail（含现有会话测试）。第一次 type-check 发现本地 bun-test.d.ts 未声明 toBeGreaterThan/toBeLessThan，改用现有 toBe 布尔断言后通过，没有扩大公共类型声明。

命令均在小程序目录执行（scratch 脚本除外）：

| 命令 | 真实输出结论 |
|---|---|
| bun test | exit 0，710 pass / 0 fail，62 files（首次全量 12.95s） |
| bun run type-check | exit 0，vue-tsc --noEmit 无错误 |
| bun run check:boundaries | exit 0，依赖边界检查通过 |
| bun run verify:mp-components | exit 0，All mini-program custom components are registered. |
| bun run build:h5 | exit 0，DONE Build complete. |
| MINI_REVIEW_SKIP=1 bun run build:mp-weixin | exit 0，Build complete；边界、组件注册及生产 API 产物检查通过；保留已有版本，无登记/上传 |
| git diff --check | exit 0 |
| scratch check_invariants.py | exit 0；受保护脚本/文案条件/原测试/版本/边界脚本均保持，token 检查通过 |
| scratch smoke-components.ts | exit 0；真实 Vue SFC 编译及 VNode 事件冒烟通过 |

构建存在既有 uni-app 新版和 Browserslist 数据更新提示，未升级依赖。scratch 脚本位于 `.tmp/insights-refactor/`，不提交。

## 偏离、路径差异与未做测试

- 设计第 6 节遗漏了 index.vue 自身的背景渐变。为满足全文无 gradient 的硬性要求，除模板及组件导入接线外，还将 index 的背景遮罩改为纯色 token，并迁移同文件的原始样式值；script 运行逻辑没有改动。页面脚本接线没有新增业务函数，四个 toggle 直接在模板映射。
- 设计中的文件路径均存在；锁定样式行号已按真实选择器定位。按设计系统 ≤2rpx 归档规则，将锁定标题旧42rpx归入现有 font-44，避免新增近似刻度；其文案/结构/事件不变。
- 除上述样式归档和遗漏背景之外无功能偏离。新颜色（助攻绿、分段灰、预览纯色等）及确需保留的尺寸先加 primitive 再加 semantic；没有修改榜赛页、首页或战术板页。
- 未做微信开发者工具/真机视觉、H5 小屏截图、两端系统减弱动态效果的人工验收；终端编译、VNode 冒烟不能证明最终像素效果。需审查者核对小屏滚动/长队名、按压态、滑块、数字动画及骨架扫光。
- 未进行线上接口/数据库或支付测试：本任务不改数据链路，没有必要触碰生产数据。未上传小程序、未部署后端。

## H5 与微信跨端处理

- 入口、战术板、贡献分段/维度/更多均用 uni-app 的 @tap；不额外并绑 @click，避免一次操作触发两次。球队选择器保留原 button @click，uni-app 在 H5 处理点击、MP 编译为 tap；锁定面板原 @click/action 不动。
- 球队继续使用原横向 scroll-view、scroll-left 和 scroll-with-animation，以及 uni.createSelectorQuery().in(instance) 的居中测量。未引入 document/window 等 DOM API。真实父 view 负责纵向 stretch，避免小程序自定义组件宿主宽度收缩与 scoped 样式隔离问题。
- 新按压态使用 hover-class，H5 与微信均由 uni-app 实现，不使用 :hover；入口 hover-stay-time=80ms。锁定面板不新增按压事件。
- H5 reduced-motion 使用 CSS 媒体查询；微信系统偏好是否透传由运行环境决定，待真机核对。默认仅保留设计要求的 220ms/300ms 短动效，没有运行时读取系统设置或改动会话/加载逻辑。
- SVG 使用已有静态资源及 image mode=aspectFit，与榜赛页入口方式一致；所有新组件直接导入 .vue，微信构建产物注册验证通过。视图与事件双端编译通过不等于真机视觉已验收。
