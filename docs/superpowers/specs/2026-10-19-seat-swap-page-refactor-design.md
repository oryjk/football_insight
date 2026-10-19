# 换座页重构设计（一期：规范化与地图浮窗）

日期：2026-10-19
状态：用户已逐节确认，待审阅书面 spec
范围：`football_insight_mini/src/pages/seat-swap/**`，加少量 `src/static`、`src/styles` 改动
不涉及：后端、任何接口、独立 H5（`football_insight_h5`）、换座业务规则与文案

## 1. 背景与目标

首页、洞察页、回流看板已按项目规范重构（token 化、字号字重层级、`FiPanel`/`FiSectionHeading`、骨架屏、纯色无渐变、分层隔离）。换座页还停留在旧样式。

换座的长期目标是**提高分享意愿和使用粘性**。用户确认分两期做：

- **一期（本文）**：规范化重构，加地图浮窗交互。只改小程序，所有换座行为不变。
- **二期（另行设计）**：拉新与留存，包括"求换座"分享卡、沿用上场座位一键发布、匹配提醒、本场动态。二期需要后端改动。

背景数据（生产库只读查询，2026-10-19）：上线以来 9 场比赛，每场 1–14 条换座请求，正式匹配仅 2 次（第 583 场）。冷启动是二期要解决的核心问题；一期为二期准备干净的组件结构。

### 1.1 现状问题（均已读代码核实）

| # | 问题 | 位置 |
|---|---|---|
| 1 | 字重几乎全是字面量 `400`，标题与正文没有层级 | 所有组件 |
| 2 | 约 100 处散落 hex、20 处写死字号、47 处写死间距/圆角 | 集中在 `SeatSwapCandidateCard`（30 处 hex）、`StadiumMap`（31）、`SeatSwapPublishSheet`（16）、`SeatSwapManageSheet`（14） |
| 3 | 候选卡 4 种状态色（绿/橙/蓝/紫）各自写死 hex；"已匹配"卡为红色渐变 | `SeatSwapCandidateCard.vue` |
| 4 | Hero 用文字符号"↔"作图标 | `SeatSwapHero.vue` |
| 5 | 卡片外观各自定义，未用 `FiPanel`/`FiSectionHeading` | Hero、`SeatSwapPoolList` |
| 6 | 加载态是一行文字"正在加载换座池..." | `index.vue` |
| 7 | 登录提示、"只在成都蓉城当前比赛开放"提示是虚线框 | `index.vue` |
| 8 | 球场地图 sticky 吸顶，约占半屏，列表可视区域小 | `index.vue` `.seat-map-section` |
| 9 | 背景照片淡出用渐变 | `index.vue` `.page-bg-fade` |
| 10 | `helpers.ts` 中 3 个旧 sticky 方案函数只被测试引用（死代码）；约 170 行开发 mock 与业务纯函数混放；`formatSeatLabel`、`statusLabel` 是对 `utils/stadiumRegions.ts` 的空包装 | `helpers.ts` |
| 11 | `index.vue` 脚本约 270 行，混合加载、筛选、分组、弹层、滚动 | `index.vue` |

### 1.2 目标

- 视觉与首页、洞察页、回流看板统一：token 化、层级清晰、纯色无渐变、骨架屏。
- 地图不再吸顶；划出视口后出现浮窗按钮，点开地图弹层快速筛选。**交互必须丝滑，有动画**（见第 4 节）。
- 代码分层清晰：页面只装配，数据、浏览、操作各自一个 composable，展示组件纯 props/emit。
- **发布、确认、取消匹配、撤销申请、订阅提醒、分享、登录跳转等所有现有行为不变。**

## 2. 目标结构

```
品牌导航（不变）
比赛卡 SeatSwapMatchCard：Lucide 图标块 + "A VS B" + 时间·球场 + "N 条意向"胶囊
  └ 默认插槽（一期不使用，留给二期"本场动态"）
提示条 SeatSwapNotice（未登录 / 本场不开放，条件与文案不变）
地图区 SeatSwapRegionPanel（普通文档流，不再 sticky；页面包裹节点挂 IntersectionObserver）
换座池 SeatSwapPoolList（FiPanel + FiSectionHeading）
底部 Dock SeatSwapDock（固定，结构不变）
浮窗按钮 SeatSwapMapFab（地图划出视口后出现，位于 Dock 上方右侧）
地图弹层 SeatSwapMapSheet / 发布弹层 / 管理弹层
```

加载中显示 `SeatSwapSkeleton`；错误态卡片与"重试"保持现有文案和行为，只做 token 化。

## 3. 组件设计

所有组件放 `src/pages/seat-swap/components/`。展示组件只通过 props 接收数据、emit 发出意图，不 import `api/**`、store、session；不跨页面 import；运行时组件直接从 `.vue` 文件导入。

### 3.1 `SeatSwapMatchCard.vue`（由 `SeatSwapHero.vue` 改名改造）

- Props 不变：`matchTitle`、`matchSummary`、`candidatesCount`；新增一个默认插槽（一期不传内容）。
- 外壳用 `FiPanel`。
- 一行：64rpx 圆角图标块（`rgba(var(--fi-primitive-red-rgb), 0.1)` 底，内放 36rpx Lucide `arrow-left-right`）+ 文案区 + 右侧"N 条意向"胶囊。
- 文案区：眉标"当前比赛 · 换座撮合"（`--fi-font-22`、bold、muted）；标题（`--fi-font-36`、extrabold、`--fi-color-text-strong`）；摘要（`--fi-font-24`、muted）。
- 胶囊：`--fi-color-page` 底、`--fi-radius-round`、`--fi-font-22`、bold。

### 3.2 `SeatSwapNotice.vue`（新增，取代 `index.vue` 里的两个 `.info-row`）

- Props：`text: string`、`actionLabel?: string`；Emits：`action`。
- `--fi-color-page` 实底、`--fi-radius-md`，**去掉虚线框**；左侧 32rpx Lucide `info` 图标（muted 色）；文字 `--fi-font-24`、`--fi-color-text-secondary`。
- 有 `actionLabel` 时右侧显示墨色胶囊按钮（沿用现有"去登录"样式，token 化）。
- 页面中的显示条件、文案、`goToUserPage` 跳转不变。

### 3.3 `SeatSwapRegionPanel.vue`（改造，首屏与弹层共用）

- 结构、props、`region-tap` 事件不变。
- 标题"座位区域"（`--fi-font-26`、bold），提示文字 `--fi-font-22`、muted。
- 图例点颜色改用 `--fi-color-seat-current`、`--fi-color-seat-desired`、`--fi-primitive-ink`。
- `.stadium-wrap` 的 `margin: 0 -24rpx` 改为 `calc(-1 * var(--fi-space-24))`。
- 新增可选 prop `embedded?: boolean`（默认 `false`）：为 `true` 时去掉自身背景和阴影，用于弹层内部。

### 3.4 `SeatSwapMapFab.vue`（新增，浮窗按钮）

- Props：`visible: boolean`、`label: string`、`clearable: boolean`；Emits：`open`、`clear`。
- 位置：`position: fixed`，右侧 `var(--fi-space-24)`；底部在 Dock 之上，即 `calc(var(--fi-space-24) + var(--window-bottom) + Dock 高度 + var(--fi-space-16))`。Dock 高度收拢成 component token `--fi-component-seat-swap-dock-height`，Dock 与浮窗共用同一个值。
- 外观：墨色实底胶囊（`--fi-primitive-ink`）、白字 `--fi-font-24` bold、`--fi-shadow-floating`；左侧 32rpx Lucide `map` 图标（白色版本）。
- `clearable` 为真时，标签右侧加一个独立的 ✕ 点击区（最小 64rpx 方形热区），点 ✕ emit `clear`；点其余部分 emit `open`。
- 组件常驻 DOM，用 class 切换进出场动画（见第 4 节），不用 `v-if`，以保证退场动画能播完；隐藏时 `pointer-events: none`。
- 无障碍：`role="button"`、`aria-label`（"打开球场地图"或"当前筛选 112 区，打开球场地图"）。

### 3.5 `SeatSwapMapSheet.vue`（新增，地图弹层）

- 基于 `FiBottomSheet`：`eyebrow="球场地图"`，标题为"点分区筛选意向"，`fitContent`，`height="80vh"`。
- 内容：`SeatSwapRegionPanel`（`embedded`），props 与首屏地图相同。
- Props：`visible`、`mode`、`regions`、`badges`、`filterKey`、`currentKey`、`desiredKeys`、`hasMyRequest`；Emits：`update:visible`、`region-tap`。
- 打开时页面用 `page-meta` 锁滚动：`pageScrollLocked` 加入 `mapSheetVisible`。

### 3.6 `SeatSwapPoolList.vue`（改造）

- 外壳 `FiPanel`；标题区 `FiSectionHeading`：kicker"换座池"，标题"按分区浏览"（总条数已在比赛卡胶囊展示，不重复）；筛选态时 kicker"目标座位"，标题为分区名，`meta` 插槽放"✕ 清除"胶囊。
- 分组行：分区名 `--fi-font-28` bold；条数 `--fi-font-22` muted；"命中目标座位"标签用 `--fi-color-seat-desired(-bg)`；"展开/收起"改为 32rpx Lucide `chevron-down` 图标，展开时旋转 180°（`transform`，220ms）。
- 列表容器 `:key` 跟随筛选 key，切换时播放 `fi-rise-in`；候选卡入场按第 4 节错开。
- Props、Emits、`id` 锚点（`buildSeatSwapRegionAnchorId`）不变。

### 3.7 `SeatSwapCandidateCard.vue`（改造）

- 外壳圆角 `--fi-radius-md`、边框 `--fi-border-card`、阴影 `--fi-shadow-card`、底色 `--fi-color-surface`。
- 人名 `--fi-font-28` bold；座位值 `--fi-font-26` extrabold；"当前座位/目标座位"标签 `--fi-font-20` bold。
- "当前座位"块用 `--fi-color-seat-current` / `--fi-color-seat-current-bg`，"目标座位"块用 `--fi-color-seat-desired` / `--fi-color-seat-desired-bg`。
- **状态胶囊简化**：统一中性底（`--fi-color-page`、`--fi-color-text-secondary`）+ 左侧 12rpx 圆点。圆点颜色：`communicable` 用 `--fi-color-seat-desired`；`waiting_peer_confirmation` 用 `--fi-color-text-muted`；`peer_confirmed_me` 胶囊整体改为主色强调（`rgba(var(--fi-primitive-red-rgb), 0.1)` 底、`--fi-color-primary` 字和点）；`matched` 见下。**删除橙、蓝、紫三组 hex**，状态文案不变。
- **已匹配卡**：红色渐变改为纯色 `--fi-color-primary` 底，边框 `--fi-color-primary-deep`；卡内文字、胶囊、半透明白层保持现有结构，白色和透明度改用 `rgba(var(--fi-primitive-white-rgb), …)`。
- 操作按钮：`--fi-font-28` bold、`--fi-radius-md`；主按钮 `--fi-primitive-ink` 底；次按钮描边。宽度和最小高度沿用现值，收拢为 component token。
- Props、Emits、按钮出现条件与文案不变。

### 3.8 `SeatSwapDock.vue`（改造）

- 结构、Props、Emits 不变。
- 圆角 `--fi-radius-md`；阴影改为 `--fi-shadow-floating`；"当前/目标"小块用 seat token；字号 `--fi-font-28`（CTA）、`--fi-font-22`（状态行）；字重 bold。
- 高度由 `--fi-component-seat-swap-dock-height` 约束，与浮窗定位共用。

### 3.9 `SeatSwapPublishSheet.vue`、`SeatSwapManageSheet.vue`（只改样式）

- 结构、文案、事件、表单逻辑（`useSeatSwapPublishForm`）不变。
- 散落 hex、字号、圆角、字重改 token；步骤条"已完成"态用 seat-desired token；危险按钮用 `--fi-color-primary`。

### 3.10 `StadiumMap.vue`（只改样式）

- 票价分区配色（蓝、绿、紫、黄、藏青、红、VIP 红，以及对应价位图例）是业务数据色，按设计系统的颜色例外**保留原值**，在规则旁标注 `/* decorative: 票价分区配色 */`。
- 其余文字色、底色、字号、间距改 token；当前/目标高亮色改 seat token。
- 交互逻辑（`canInteractSeatSwapMap`、`shouldDimSeatSwapRegion`）不变。

### 3.11 `SeatSwapSkeleton.vue`（新增）

- 使用全局 `fi-skeleton-*`，`role="status"`、`aria-label="换座池加载中"`。
- 形状：比赛卡（图标块 + 两行 + 胶囊）、地图区（一个大块）、换座池（标题行 + 3 张候选卡）。
- 替换 `index.vue` 的"正在加载换座池..."文字卡。

### 3.12 删除与改名

- `SeatSwapHero.vue` → `SeatSwapMatchCard.vue`。
- `index.vue` 的 `.info-row` 与 `.state-card` 加载态样式随替换删除。

### 3.13 图标

新增 Lucide 图标：`arrow-left-right`、`info`、`map`、`chevron-down`。按 `src/static/ticket-watch/` 的约定放到 `src/static/seat-swap/`：24×24、stroke-width 2、round 端点，文件头保留 Lucide 版权注释，路径与 Lucide 官方源一致（实现者核对后写入）。需要的颜色版本：`arrow-left-right` 主色；`info`、`chevron-down` muted 灰；`map` 白色。

## 4. 动效规格

**原则**：只动 `transform` 与 `opacity`；滚动可见性用 IntersectionObserver，不在 `onPageScroll` 中逐帧计算；所有动效在 mp-weixin 真机上验收。

新增 motion token（`src/styles/fi-tokens.css`，取值来自已有的 `FiBottomSheet`、`FiSegmented`，其他页面现有动画本期不改）：

| token | 值 |
|---|---|
| `--fi-motion-duration-fast` | 120ms |
| `--fi-motion-duration-base` | 220ms |
| `--fi-motion-duration-slow` | 280ms |
| `--fi-motion-ease-out` | `cubic-bezier(0.22, 1, 0.36, 1)` |
| `--fi-motion-ease-in` | `cubic-bezier(0.4, 0, 1, 1)` |
| `--fi-motion-ease-spring` | `cubic-bezier(0.3, 0.9, 0.3, 1)` |

| 场景 | 动效 | 时长 / 曲线 |
|---|---|---|
| 浮窗出现 | 从 `translateY(24rpx) scale(0.9)`、`opacity: 0` 到原位 | slow / ease-out |
| 浮窗消失 | 到 `translateY(16rpx) scale(0.95)`、`opacity: 0` | 200ms / ease-in |
| 浮窗文字"地图"↔分区名 | 旧文字上移 8rpx 淡出，新文字自下 8rpx 淡入（两层交叉） | base / ease-out |
| 浮窗按压 | `scale(0.94)`（`hover-class`） | fast |
| 地图弹层开合 | 复用 `FiBottomSheet` 现有动画 | 240ms / 220ms |
| 弹层内点有意向分区 | 分区高亮约 120ms → 关闭弹层 → 关闭动画结束后再滚动列表（串行，不抢帧） | 关闭 220ms，滚动 300ms |
| 弹层内点无意向分区 | 只切换筛选，不关闭弹层 | — |
| 首屏地图点有意向分区 | 设置筛选，平滑滚动到分组（原 220ms 改为 300ms） | 300ms |
| 筛选结果切换 | 列表容器 `:key` 切换，播放 `fi-rise-in` | base |
| 分组展开 | 候选卡逐张 `fi-rise-in`，错开 40ms，最多错开 5 张（第 6 张起延迟 200ms） | base |
| 展开箭头 | `rotate(180deg)` | base / ease-out |
| 骨架屏→内容 | 内容整体淡入 | 200ms |

- **防抖**：地图包裹节点可见性变化后 150ms 再更新浮窗显隐，避免边缘反复进出造成闪烁。
- **reduced-motion**：H5 用 `prefers-reduced-motion` 媒体查询关闭位移和缩放，只保留不超过 150ms 的淡入淡出；小程序端按设计系统约定缩短时长、取消位移；滚动改为 `duration: 0`。
- 浮窗与弹层不同时出现：弹层打开时浮窗隐藏（`shouldShowSeatSwapMapFab`）。

## 5. 分层与数据流

```
pages/seat-swap/
├ index.vue               装配：模板、page-meta 滚动锁、IntersectionObserver、onShow、onShareAppMessage
├ useSeatSwapPage.ts      （新）数据：getCurrentSeatSwap + getTicketWatchRegions、loading、errorMessage、mock 开关、loadPage
├ useSeatSwapBrowse.ts    （新）浏览：筛选 key、分组、展开、徽标计数、我的目标分区、浮窗显隐与文案、地图点击处理
├ useSeatSwapActions.ts   不改
├ helpers.ts              纯函数
└ fixtures.ts             （新）开发 mock，从 helpers.ts 原样迁出
```

- `index.vue` 现有的计算属性和函数按职责迁入两个 composable，**名称、语义、调用顺序不变**；`useSeatSwapActions` 的入参不变（`loadPage` 改由 `useSeatSwapPage` 提供）。
- `useSeatSwapPage` 是唯一调用 `api/**` 的地方（页面目录内，符合边界规则）。
- IntersectionObserver 在 `index.vue` 的 `onReady` 中通过 `uni.createIntersectionObserver(instance)` 创建，观察地图包裹节点 `#seat-swap-map-anchor` 相对视口的可见性，在 `onUnload` 中 `disconnect`。observer 回调只写入 `useSeatSwapBrowse` 暴露的 `mapInViewport`。
- 分享 `onShareAppMessage` 的标题和 `buildSharePath('/pages/seat-swap/index', 'swap')` 不变。

### 5.1 `helpers.ts` 改动

- **删除**死代码：`shouldShowSeatSwapStickyMap`、`readSeatSwapViewportScrollTop`、`resolveSeatSwapStickyMapThreshold` 及其测试用例。
- **删除**空包装 `formatSeatLabel`、`statusLabel`，调用处改为直接使用 `utils/stadiumRegions.ts` 的 `formatSeatSwapSeatLabel`、`seatSwapStatusLabel`。
- **迁出** `buildSeatSwapMockRegions`、`buildSeatSwapMockCurrentResponse` 及私有的 `buildSeatSwapMockRequest` 到 `fixtures.ts`，内容不变。
- **新增**（先写失败的表驱动单测）：

```ts
export function resolveSeatSwapMapFabLabel(filterName: string): { text: string; clearable: boolean }
// '' 或纯空白 → { text: '地图', clearable: false }；'112 区' → { text: '112 区', clearable: true }

export function resolveSeatSwapStaggerDelayMs(index: number, step = 40, maxSteps = 5): number
// 0→0，1→40，5→200，9→200；负数或非有限数→0

export function shouldShowSeatSwapMapFab(input: {
  available: boolean
  mapInViewport: boolean
  sheetOpen: boolean
}): boolean
// 仅当 available && !mapInViewport && !sheetOpen 时为真；表驱动覆盖全部 8 种组合

export type SeatSwapMapTapOutcome = { filterKey: string; closeSheet: boolean; scrollTo: string | null }
export function resolveSeatSwapMapTap(input: {
  currentFilterKey: string
  tappedKey: string
  hasBadge: boolean
  fromSheet: boolean
}): SeatSwapMapTapOutcome
// filterKey 沿用 resolveSeatSwapBrowseFilterKey；
// hasBadge 且新 filterKey 非空 → scrollTo = tappedKey，fromSheet 时 closeSheet = true；
// 其余 → scrollTo = null，closeSheet = false
```

`resolveSeatSwapMapTap` 统一首屏地图与弹层地图的点击规则，替代 `index.vue` 里的 `handleMainMapTap` 分支逻辑，首屏行为与现在一致。

## 6. Token 改动（`src/styles/fi-tokens.css`）

只新增以下 token，优先复用已有值，不为一次性数值批量造 token：

| 层 | token | 取值来源 |
|---|---|---|
| primitive | `--fi-primitive-green-deep` | 现有 `#167348` |
| primitive | `--fi-primitive-green-soft` | 现有 `#eef8f2` |
| primitive | `--fi-primitive-red-soft` | 现有 `#fff1f0` |
| semantic | `--fi-color-seat-current` | `var(--fi-color-primary)` |
| semantic | `--fi-color-seat-current-bg` | `var(--fi-primitive-red-soft)` |
| semantic | `--fi-color-seat-desired` | `var(--fi-primitive-green-deep)` |
| semantic | `--fi-color-seat-desired-bg` | `var(--fi-primitive-green-soft)` |
| semantic | motion 6 个 | 见第 4 节 |
| component | `--fi-component-seat-swap-dock-height` | Dock 实测高度 |
| component | `--fi-component-seat-swap-action-width` / `-min-height` | 现值 168rpx / 152rpx |

实现前先 grep `fi-tokens.css`，已有等值 token 时直接复用并在交付说明中写明。

## 7. 背景遮罩

`index.vue` 的 `.page-bg-fade` 渐变改为两层纯色：照片区一层 `rgba(var(--fi-primitive-white-rgb), 0.5)` 浅遮罩（高度与照片可见区一致），其下内容区为 `--fi-color-page-soft` 实底。真机效果若过渡生硬，作为单独问题再议，不在本期引入渐变例外。

## 8. 必须保持不变的行为（回归点）

1. 发布、编辑、撤销发布、确认换座、取消匹配确认、已匹配撤销申请（含说明）的条件、文案、接口调用不变；`useSeatSwapActions.ts` 一行不改。
2. 订阅提醒（`miniProgramNoticeEnabled`）流程不变。
3. 未登录可浏览脱敏意向；"去登录"跳转 `/pages/user/index` 不变。
4. 非开放比赛时 Dock 不显示、提示文案不变。
5. 换座池分组、默认展开（`resolveDefaultExpandedSeatSwapRegionKeys`）、过滤掉我自己的请求、徽标计数、按目标分区筛选的结果不变。
6. `onShow` 重新加载、`onShareAppMessage` 不变；DEV mock 开关 `VITE_SEAT_SWAP_MOCK_LARGE_DATA` 仍可用。
7. 发布弹层、管理弹层打开时页面滚动锁不变；新增的地图弹层同样加锁。
8. 现有 `helpers.test.ts` 用例（除删除的 3 个死代码函数）全部通过。

## 9. 约束与交付

1. 只改 `football_insight_mini`；不改后端、接口、`football_insight_h5`；不读取或修改 `football_insight_admin_android`。
2. 颜色、间距、字号、行高、圆角、边框、阴影、字重、动效时长与曲线全部引用 token；`src/pages/seat-swap` 下除标注 decorative 的票价分区配色外无散落 hex；无 `gradient`；无 `font-weight` 数字字面量。
3. 不往 `scripts/check-import-boundaries.mjs` 的 ALLOWLIST 加例外；不跨页面 import。
4. 不手改 `manifest.json` 的 `versionName`；**不上传小程序**，由审查后统一用 `--robot 2` 上传体验版。
5. 小步提交，`feat(mini): ...` / `refactor(mini): ...`，subject 英文，末尾加 Co-Authored-By；提交并推送 `main`，先 `git pull --rebase`。
6. `CHANGELOG.md`、`FEATURES.md` 各补一条并写明原因；`docs/mini-design-system.md` 补充 motion token 与 seat token 说明。
7. 验证命令：`bun test`、`bun run type-check`、`bun run check:boundaries`、`bun run verify:mp-components`、`bun run build:mp-weixin`。动效与布局以 mp-weixin 构建产物和真机为准，交付说明里逐条写明每个组件的结构、所用 token 和参照出处。

## 10. 范围外（二期）

- "求换座"分享卡/海报，以及分享落地后直接展示可互换视图；
- 沿用上场座位一键发布（需要后端提供历史请求）；
- 互补座位出现时的订阅消息提醒覆盖面评估；
- 比赛卡插槽中的"本场动态"（N 人在换、目标分区热度、距开赛时间）；
- 简化发布门槛（先选分区、后补排号与联系方式）；
- 独立 H5 换座页同步。

## 11. 审查清单

1. 地图不再 sticky；划出视口后浮窗出现，回到视口时消失；防抖 150ms 生效，边缘无闪烁。
2. 浮窗无筛选时显示"地图"，有筛选时显示分区名和 ✕；✕ 清除筛选，其余区域打开弹层；文字切换为交叉过渡。
3. 弹层内点有意向分区：高亮→关闭→滚动，串行执行；点无意向分区只切换筛选不关闭；首屏地图点击行为与旧版一致。
4. 所有动画只动 transform / opacity，时长与曲线引用 motion token；reduced-motion 降级生效。
5. `src/pages/seat-swap` 下无 `gradient`，除 decorative 注释外无 hex，无数字字重。
6. 候选卡状态色仅剩中性加主色强调，橙、蓝、紫 hex 已删除；已匹配卡为纯色主色。
7. 比赛卡、换座池使用 `FiPanel`/`FiSectionHeading`；提示条无虚线框；骨架屏替换文字加载态。
8. `helpers.ts` 死代码、空包装已删，mock 迁至 `fixtures.ts`，新纯函数有表驱动单测。
9. `index.vue` 只做装配；数据与浏览逻辑在两个 composable 中；`useSeatSwapActions.ts` 无改动；第 8 节回归点逐条成立。
10. 边界检查、类型检查、组件注册检查、单测、mp-weixin 构建全部通过；无 ALLOWLIST 新增。
