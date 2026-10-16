# 洞察页重构设计

日期：2026-10-16
状态：待用户确认后交付实现
范围：`football_insight_mini/src/pages/insights/**`，加少量 `src/static`、`src/styles` 改动
不涉及：后端、任何接口、会话与数据加载逻辑、其他页面

## 1. 背景与目标

首页和榜赛页已按新的视觉与交互重构（图标引导卡、分段切换加过渡动画、胶囊行、统一骨架屏、纯色无渐变）。洞察页还停留在旧样式，和它们不一致，并且有大量"没有作用"的内容。

用户已确认的目标：**风格统一 + 整理内容，页面结构基本保持**；并明确要求**去掉顶部纯描述的卡片**（占面积太大）。

### 1.1 现状问题（均已读代码核实）

| 位置 | 问题 |
|---|---|
| `InsightsHeroCard`：Eyebrow"Insights"、标题"洞察不是榜单，而是贡献结构"、一整段说明、右上"球队洞察"、两张点阵图 | 纯描述，占首屏很大面积，首页、榜赛页都没有这种卡 |
| `InsightsTicketWatchEntry`：只有 "Ticket Watch / 回流看板 / 打开" 的标题行 | 这是全产品访问量最大、唯一付费来源的功能入口（洞察页是它的唯一入口），却做成像标题一样的小卡 |
| `InsightsTeamSelector`：整张卡，标题"球队列表 / 先选一个球队"，右上"N 支球队" | 标题和计数没有作用 |
| `InsightsSummaryPanel`（当前球队摘要）+ `InsightsTeamBoardEntry`（"球队战术板 / 把这张洞察卡变成观点"） | 两张分散的卡，后者文案冗长 |
| 四张 `InsightsContributionPanel` 上下堆叠，每张有 kicker 和标题 | 页面很长，信息层级不清；四个维度本质是同一份"贡献结构"的不同视角 |
| `FiLoading` 文字"洞察生成中 / 账号状态确认中" | 首页、榜赛页已用统一骨架屏 |
| 渐变 | 进度条、摘要卡、锁定面板、球队胶囊头像等多处 `linear-gradient`（见 6 节清单） |

## 2. 目标结构

从上到下：

```
品牌导航（不变）
┌ 回流看板入口卡（图标引导卡）
├ [加载中] 骨架屏
├ [游客] 锁定预览 / [权益暂停] 锁定提示 / [审核态] 基础提示 / [错误] / [无数据]（状态块沿用现有逻辑）
└ [有数据]
   ┌ 球队卡：球队胶囊行 + 当前球队摘要（含战术板入口）
   └ 贡献卡：进球 | 助攻 | 失球 分段切换 + 一句话说明 + 排行列表
```

**删除**：Hero 说明卡、独立的战术板入口卡、四张独立贡献卡、"先选一个球队/N 支球队"标题。

页面行为与数据流（登录、会员锁定、审核态、加载、错误、会话切换）**一行逻辑不改**。

## 3. 组件设计

所有新增组件放 `src/pages/insights/components/`，**不得 import 其他页面**（尤其不要 import `rankings` 的组件，边界脚本会拦截）；不得往 `check-import-boundaries.mjs` 的 ALLOWLIST 加例外。视觉遵循 `docs/mini-design-system.md`：颜色、间距、字号、圆角、边框、阴影、字重全部引用 token，不新增散落 hex。

### 3.1 `InsightsTicketWatchEntry.vue`（重写，保持名称与对外契约）

对外契约不变：`defineProps<{ locked: boolean }>()`、`emit('open')`。页面里的 `@open="openTicketWatch"`、`:locked` 绑定一行不改，现有付费度量（`insights_ticket_watch_entry` 的曝光与点击）因此保持有效。

形态参考榜赛页 `RankingsStandingsLauncher`（只参考样式，不 import）：

- 一行：64rpx 圆角图标块（浅红底 `rgba(var(--fi-primitive-red-rgb), 0.1)`，内放 36rpx 的 `src/static/user/ticket.svg`）+ 文案区 + 右侧。
- 文案区：标题"回流看板"（`--fi-font-32`、extrabold），说明一行（`--fi-font-24`、muted）：已登录显示"实时余票监控 · 最近回流速览"，`locked` 时显示"登录后查看实时余票与回流"。
- 右侧：已登录显示 36rpx 的 `src/static/rankings/chevron-right.svg`；`locked` 时显示"登录后"小胶囊。
- 卡片外观与榜赛页的 `.panel` 一致：`rgba(255,255,255,0.94)`、`--fi-radius-xl`、`--fi-border-card`、`--fi-shadow-card`。
- 点击用 `@tap`，带按压态 `hover-class` 降低透明度到 0.7，`role="button"` 与 `aria-label`。
- 去掉原来的 `__veil` 遮罩层。

### 3.2 `InsightsTeamPanel.vue`（新增，合并三块）

一张 `FiPanel`，内含两部分，间距 `--fi-space-22`：

- 上：`InsightsTeamSelector`（球队胶囊行，改成无外壳）。
- 下：`InsightsSummaryPanel`（改成无外壳，含战术板入口）。

Props：`teams`、`selectedTeamId`、`insight`；Emits：`update:selectedTeamId`、`open-board`。页面用 `v-model:selected-team-id` 绑定。

### 3.3 `InsightsTeamSelector.vue`（改造）

- 去掉 `FiPanel` 与 `FiSectionHeading`（"球队列表 / 先选一个球队 / N 支球队"全部删除）。
- 保留现有横向 `scroll-view`、选中胶囊居中滚动的脚本逻辑（`centerSelectedChip`）原样不动。
- 胶囊：未选中 `--fi-color-page` 底、`--fi-color-text-secondary` 字；选中 `--fi-color-text-primary` 底、白字、加粗；**去掉** 选中时的上移和阴影；头像 36rpx；无头像的占位头像改**纯色** `--fi-color-primary`（去掉渐变）。
- 内容文案保持"#名次 队名"。

### 3.4 `InsightsSummaryPanel.vue`（改造）

去掉外壳，只做摘要：

- 第一行：左侧队名（`--fi-font-36`、extrabold）+ "第 N 名"小胶囊（`--fi-color-page` 底）；右侧"战术板 ›"链接（`--fi-color-primary`、`--fi-font-26`、bold，箭头 32rpx 的 `chevron-right.svg`），点击 `emit('open-board')`，带按压态。
- 第二行：两个指标块（总进球、总失球），`--fi-color-page` 底、`--fi-radius-md`，数字沿用 `useAnimatedInteger` 和现有的 `fi-value-pop` 动画与 `:key` 重触发写法；**去掉**"赛季累计打进/赛季累计丢失"副文案和渐变底。
- 契约：`defineProps<{ insight: TeamInsight }>()`，`emit('open-board')`。

### 3.5 `InsightsContributionSurface.vue`（新增，取代四张贡献卡）

一张 `FiPanel`。

**Props / Emits**（页面继续使用 `useInsightsData` 里现有的四个 expanded 标志，**不改该 composable**）：

```ts
defineProps<{
  rows: Record<InsightContributionViewKey, InsightContributionRow[]>
  expanded: Record<InsightContributionViewKey, boolean>
}>()
emit('toggle', key: InsightContributionViewKey)
```

页面把 `goalsForByOpponentRows / goalsForByPlayerRows / assistsByPlayerRows / goalsAgainstByOpponentRows` 与四个 `*Expanded` 映射成上述两个对象；`toggle` 时翻转对应标志。

**内部状态**（组件本地，不进 composable）：`tab`（默认 `'goals-for'`）、`dimension`（默认 `'opponent'`）。

**视觉与交互**：

1. 顶部**分段切换**（3 段：进球 / 助攻 / 失球），形态与榜赛页 `.scope-toggle` 一致：`--fi-color-border-chip` 描边的胶囊容器、内部有一个随选中项滑动的**实心墨色滑块**（`--fi-primitive-ink`）、选中文字白色 bold、未选中 `#7f8490`；滑块位移用 `transform: translateX()` + 300ms `cubic-bezier(0.3, 0.9, 0.3, 1)`。三段时滑块宽度 `calc((100% - var(--fi-space-6) * 4) / 3)`，位移分别为 0、`calc(100% + var(--fi-space-6))`、`calc(200% + var(--fi-space-6) * 2)`。
2. 当 `tab === 'goals-for'` 时，下方出现**二级胶囊行**：按对手 / 按球员（样式同球队胶囊：选中墨色底白字，未选中 `--fi-color-page`）。助攻只有球员维度、失球只有对手维度，不显示二级行。
3. **一句话说明**（`--fi-font-24`、muted），取自 `INSIGHT_CONTRIBUTION_CAPTIONS`，取代原来的 Hero 描述和每张卡的 kicker/标题。
4. **列表**：沿用现有行的结构和数据（头像 48rpx、名称、备注、占比、进度条），默认显示前 3 行，超过 3 行显示"更多/收起"按钮（沿用 `getVisibleInsightContributions` / `shouldShowInsightToggle`）。列表容器用 `:key="viewKey"`，切换时播放进入动画（淡入加轻微上移，约 220ms，可复用现有 `fi-rise-in`）。
5. 所选视图无数据时，显示一行"暂无数据"（`--fi-color-text-muted`），不显示分段之外的空白卡。
6. 切换分段或维度**不重置**展开状态；切换球队时的展开状态行为与现在保持一致（不新增重置）。

**进度条**全部改**纯色**（去掉渐变）：`ink` → `var(--fi-primitive-ink)`、`red` → `var(--fi-color-primary)`、`green` → 现有绿色字面量的单色（取原渐变的深色端 `#0dbd73`，需要的话按规范先加 primitive 再引用）、`danger` → `var(--fi-color-primary)`。

### 3.6 `InsightsSkeleton.vue`（新增）

仿 `RankingsSkeleton`/`HomeSkeleton`：使用全局 `fi-skeleton-*` 样式类（`src/styles/fi-skeleton.css`），`role="status"`，`aria-label="洞察加载中"`。形状：一张球队卡（一行胶囊占位 + 队名行 + 两个指标块）和一张贡献卡（分段占位 + 说明行 + 3 行列表占位）。

用它替换页面里两处 `FiLoading`：`userLoading`（"账号状态确认中"）和 `loading`（"洞察生成中"）。

### 3.7 `InsightsLockedPanel.vue`（只改样式）

游客预览与会员锁定提示的**文案、结构、事件不变**，只去掉渐变（见 6 节）。

### 3.8 删除

- `InsightsHeroCard.vue`
- `InsightsTeamBoardEntry.vue`（战术板入口并入摘要）
- `InsightsContributionPanel.vue`（被 Surface 取代，行样式迁入 Surface）
- `src/static/user/member-card-dots.png`：删除 Hero 后若全仓再无引用则一并删除（先 grep 确认）。

## 4. `helpers.ts` 新增（纯函数与常量，必须有单测）

```ts
export type InsightContributionTab = 'goals-for' | 'assists' | 'goals-against'
export type InsightGoalsForDimension = 'opponent' | 'player'
export type InsightContributionViewKey =
  'goals-for-opponent' | 'goals-for-player' | 'assists-player' | 'goals-against-opponent'

export const INSIGHT_CONTRIBUTION_TABS: { key: InsightContributionTab; label: string }[]
  // [{goals-for,进球},{assists,助攻},{goals-against,失球}]，顺序固定
export const INSIGHT_GOALS_FOR_DIMENSIONS: { key: InsightGoalsForDimension; label: string }[]
  // [{opponent,按对手},{player,按球员}]
export const INSIGHT_CONTRIBUTION_CAPTIONS: Record<InsightContributionViewKey, string>
  // 每个视图一句话说明，4 句互不相同，长度大于 4 且小于 30
export function resolveContributionViewKey(
  tab: InsightContributionTab, goalsForDimension: InsightGoalsForDimension,
): InsightContributionViewKey
  // assists → assists-player；goals-against → goals-against-opponent；goals-for 按维度
```

说明文案（定稿，实现者直接使用）：

| 视图 | 文案 |
|---|---|
| `goals-for-opponent` | 本队的进球主要打在哪些对手身上 |
| `goals-for-player` | 本队的进球由哪些球员贡献 |
| `assists-player` | 本队的助攻由哪些球员贡献 |
| `goals-against-opponent` | 本队的失球主要来自哪些对手 |

单测覆盖：tab 顺序与标签、维度、`resolveContributionViewKey` 全部分支、说明文案互异且长度范围。

## 5. 页面 `index.vue` 的改动范围

只改**模板**与新增组件的接线，不改 `<script>` 里的会话、加载、watcher、`openTicketWatch`、付费度量调用：

- 删除 `<InsightsHeroCard />`、`<InsightsTeamBoardEntry />`、四个 `<InsightsContributionPanel>`、`<InsightsTeamSelector>`、`<InsightsSummaryPanel>`。
- `<InsightsTicketWatchEntry>` 保留原条件与绑定。
- 两处 `FiLoading` 改为 `<InsightsSkeleton />`，原有 `v-if/v-else-if` 条件顺序不变。
- 有数据时渲染 `<InsightsTeamPanel>`（`v-model:selected-team-id="selectedTeamId"`、`:teams`、`:insight="selectedInsight"`、`@open-board="goToTeamBoard"`）和 `<InsightsContributionSurface>`。
- 若 `FiLoading` 等导入变为未使用，一并清理。

## 6. 渐变清单（全部改纯色）

| 文件 | 内容 | 改为 |
|---|---|---|
| `InsightsSummaryPanel.vue` | `.briefing-card` 背景渐变 | `--fi-color-page` 纯色（见 3.4） |
| `InsightsContributionPanel.vue`（将删除） | 四种进度条渐变 | 见 3.5 纯色 |
| `InsightsLockedPanel.vue` | 约 131 行浅红到白渐变卡底；184 行进度条墨色渐变；194 行预览遮罩渐变 | 分别改为 `rgba(var(--fi-primitive-red-rgb), 0.06)`、`var(--fi-primitive-ink)`、`rgba(243, 243, 246, 0.4)` 纯色 |
| `InsightsTeamSelector.vue` | 无头像占位头像渐变 | `--fi-color-primary` 纯色 |

交付前全文搜索 `src/pages/insights` 下的 `gradient`，应无结果。

## 7. 必须保持不变的行为（回归点）

1. 登录、会员权益锁定、审核态、断网、错误、无数据各状态的**显示条件和文案**不变。
2. `useInsightsData.ts`、`sessionEffects.ts` 一行不改；现有的 `sessionEffects.test.ts`、`helpers.test.ts` 原有用例全部通过。
3. 付费度量：`openTicketWatch` 里的 `reportPaywallTap('insights_ticket_watch_entry')` 与 `reportTicketWatchEntryImpression`、`beginPaywallVisit` 不动；回流看板入口仍是 `InsightsTicketWatchEntry`，`@open` 语义不变。
4. 点击回流看板入口、战术板入口的跳转目标不变。
5. 球队切换、选中胶囊居中滚动、数字动画行为不变。
6. 横向滚动、按压态、`@tap`/`@click` 在小程序与 H5 两端都可用（跨端约束见 `football_insight_mini/AGENTS.md`）。

## 8. 验证与交付

- 命令：`bun test`、`bun run type-check`、`bun run check:boundaries`、`bun run verify:mp-components` 全部通过；新增 helpers 单测先写失败再实现。
- 视觉无法在终端验证，**实现者不要上传小程序**；提交后由审查者核对并上传体验版，用户看真机效果。
- 提交信息沿用风格：`feat(mini): ...`，subject 用英文，末尾加 Co-Authored-By 行；直接提交到 `main` 并推送，不新建分支；不要手改 `manifest.json` 的 `versionName`。
- 在 `CHANGELOG.md` 与 `FEATURES.md` 各补一条，并写明原因。

## 9. 范围外

- 任何后端或接口改动；新增洞察指标或数据；
- 战术板页、余票监控页、首页、榜赛页的改动；
- 抽取跨页面共享的分段切换组件（榜赛页已有自己的实现，本次不动它，后续可单独做）；
- 付费入口的文案、价格或转化改版。

## 10. 审查清单（Review 逐条核对）

1. 顶部说明 Hero、独立战术板卡、四张贡献卡、"先选一个球队/N 支球队"都已删除，没有残留的无用文案。
2. `src/pages/insights` 下无 `gradient`；新增颜色、间距、字号全部引用 token，没有新增散落 hex。
3. `InsightsTicketWatchEntry` 的 `locked` 与 `open` 契约不变，页面绑定未改，付费度量调用原样存在。
4. `useInsightsData.ts`、`sessionEffects.ts` 没有改动；各状态（加载/游客/锁定/审核/错误/无数据）条件顺序与原来一致。
5. 贡献卡：三段切换与滑块位置正确；进球才有二级维度；助攻、失球没有二级行；说明文案正确；"更多/收起"逐视图独立；无数据显示"暂无数据"。
6. 骨架屏替换了两处 `FiLoading`，使用全局 `fi-skeleton-*`，无自定义渐变。
7. 没有 import 其他页面，没有 ALLOWLIST 新增；所有被删文件无残留引用；`member-card-dots.png` 无引用才删除。
8. 新增 helpers 单测齐全，原有测试全部通过。
9. 图标尺寸统一（入口卡 64rpx 图标块内 36rpx、箭头 36rpx 或 32rpx），没有尺寸不一的零散图标。
10. 小屏手机下球队胶囊行可横向滚动，队名、"第 N 名"、"战术板 ›"同一行不溢出换行错位。
