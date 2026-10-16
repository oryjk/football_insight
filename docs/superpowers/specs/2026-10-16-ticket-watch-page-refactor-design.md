# 回流看板页重构设计

日期：2026-10-16
状态：用户已确认路线，待交付实现
范围：`football_insight_mini/src/pages/ticket-watch/**`，加少量 `src/static`、`src/styles` 改动
不涉及：后端、任何接口、轮询节奏与监控语义、会员权益规则、付费商品

## 1. 背景与目标

回流看板是全产品最核心的页面：访问量最大（累计约 28,000 次），且是唯一的付费来源（会员的核心权益就是余票刷新速度和最近回流速览）。洞察页是它的唯一入口。首页、榜赛页、洞察页都已按新的视觉与交互重构，这里还停留在旧样式。

用户已确认的方向（三选一的第一项）：**理清首屏并补上刷新状态**；并追加要求：**去掉云南玉昆，只保留成都蓉城**。

### 1.1 现状问题（均已读代码核实）

| # | 问题 | 依据 |
|---|---|---|
| 1 | **页面从头到尾没有展示"刷新有多快、数据多新"** | 搜索所有组件模板，没有刷新频率、最近更新时间、下次刷新的任何展示；轮询间隔 `pollIntervalSeconds`（按会员等级 600 秒到 1 秒）只在 `useTicketWatchPolling.ts` 里用于定时请求。会员权益的核心价值对用户不可见 |
| 2 | **轮询失败时用户毫不知情** | `ticketWatchCurrentBoard.ts`：首次加载之后的 `poll` 失败，`hasLoadedCurrentBoard` 为真，错误被吞掉，页面继续展示旧数据，没有任何提示 |
| 3 | **"监控"是否开启不直观** | `isMonitoringActive` 只通过按钮文案"开始监控/停止监控"体现；不开启监控时数据不会自动刷新，用户可能不知道 |
| 4 | 头部说明卡纯描述，"回流看板"在同一屏出现三次 | `TicketWatchHeader.vue`：眉标、卡内徽章、徽章里的图标文字 |
| 5 | 球队切换写死两支球队，用普通按钮 | `TicketWatchHeader.vue` |
| 6 | 标签切换（当前/历史/历史统计）用普通按钮，无图标，没用共享的 `FiSegmented` | 同上 |
| 7 | 当前比赛卡里三个按钮地位相同 | `TicketCurrentMatchCard.vue`：开始监控 / 订阅提醒 / 获取比赛 ID |
| 8 | 页面卡片堆叠过长，优先级不清 | 最多 6 到 7 张可折叠卡 |
| 9 | 多处渐变 | 见 6 节清单 |

### 1.2 目标

- 用户一眼能看到：**监控是否开启、多久刷新一次、数据多久前更新、刷新是否出错**。
- 去掉没有作用的头部说明卡和球队切换，首屏直接是比赛与状态。
- 视觉与首页、榜赛页、洞察页统一：纯色、共享分段切换、图标尺寸统一、骨架屏对齐。
- **轮询、监控、会员、订阅、比赛 ID、关注区块等所有现有行为不变。**

## 2. 目标结构（「当前比赛」标签）

```
二级导航（不变）
标签切换（FiSegmented，3 段带图标：当前比赛 / 历史比赛 / 历史统计）
当前比赛卡
  ├ 标题行：球图标 + "A VS B"，右侧 "第 N 轮 · 日期 时间"
  ├ 刷新状态条（新增，见 3.1）
  ├ 两个指标块：命中区域 / 累计回流
  ├ 主按钮（整行）：开始监控 / 停止监控
  ├ 次按钮（并排两个）：订阅提醒 / 获取比赛 ID
  └ 一行提示文字（原 message）
[我的关注区块]（有关注内容时排这里）
最近回流速览
热区
[我的关注区块]（没有关注内容时保持原位置：热区之后）
价位库存卡（每个价位一张，可折叠）
供给洞察与决策结论
```

**删除**：头部说明卡（hero）、球队切换、会员胶囊（"V3专享"，含皇冠图标）。
各卡片的折叠、展开行为与默认状态**不变**。

## 3. 组件设计

### 3.1 `TicketRefreshStatus.vue`（新增）：刷新状态条

放在当前比赛卡内。**状态机**（互斥，按优先级自上而下判断）：

| 状态 | 条件 | 左侧文案 | 右侧文案 | 圆点 |
|---|---|---|---|---|
| 未开启监控 | `!monitoring` | `未开启监控 · 数据不会自动刷新` | `{相对时间} 更新`（无数据时 `尚未更新`） | 灰色，静止 |
| 刷新中 | `monitoring && refreshing` | `刷新中…` | `{相对时间} 更新` | 主色，脉冲动画 |
| 刷新失败 | `monitoring && pollFailing` | `刷新失败，将自动重试` | `上次成功 {相对时间}` | 红色（`--fi-color-primary`），静止 |
| 监控中 | `monitoring` | `监控中 · 每 {间隔} 刷新` | `{相对时间} 更新` | 绿色，脉冲动画 |

- 圆点脉冲动画必须遵守 `prefers-reduced-motion`（关闭动画）。
- 整条使用纯色浅底（`--fi-color-page`）和 `--fi-radius-md`，**不得渐变**。
- **间隔文案**（纯函数，见 4 节）：1–59 秒显示 `N 秒`；60 秒及以上整分钟显示 `N 分钟`，不是整分钟显示 `M 分 S 秒`。例：1→`1 秒`，15→`15 秒`，120→`2 分钟`，600→`10 分钟`，90→`1 分 30 秒`。
- **相对时间**（纯函数）：`null`→`尚未更新`；小于 5 秒→`刚刚`；小于 60 秒→`N 秒前`；小于 1 小时→`N 分钟前`；其余→`N 小时前`。
- **滚动刷新**：组件内部用**自己的**定时器维护本地 `now`，监控中每 1 秒更新、未监控每 15 秒更新；`ticking` 为假（页面不可见）时停止；组件卸载时清理。**不得**用页面级的每秒状态驱动全页重渲染（`freshnessNowMs` 保持现状 15 秒）。
- Props：`monitoring: boolean`、`refreshing: boolean`、`pollFailing: boolean`、`intervalSeconds: number`、`lastUpdatedAtMs: number | null`、`ticking: boolean`。无 emit。

### 3.2 `TicketCurrentMatchCard.vue`（改造）

对外事件契约不变：`monitor`、`subscribe`、`match-id`。

- 标题行：球图标（沿用）+ "A VS B"；右侧轮次与日期时间改成一行小字（`--fi-color-text-muted`），去掉原来单独一行的 meta。
- 在标题行与指标之间插入 `TicketRefreshStatus`（新增 props 向上透传，见 5 节）。
- 指标块保持两个（命中区域、累计回流），图标沿用，去掉渐变底，改纯色 `--fi-color-page` 底。
- 按钮：**主按钮**整行（监控切换，保持 `playCircleIcon` 与现有开始/停止文案和样式语义；运行中显示"停止监控"的次要样式也保持）；**次按钮**两个并排，描边样式（订阅提醒保留已开通状态样式）。
- 去掉会员胶囊与 `membershipLabel` prop；`useTicketWatchState` 里的 `membershipExclusiveLabel` 计算属性保留不动（不再传入）。
- `message` 提示文字保留在底部。

### 3.3 `TicketWatchHeader.vue`（改造）

- 删除 hero 卡与球队切换，以及对应的 `selectedTeam` prop 和 `team` 事件。
- 标签切换换成共享的 `FiSegmented`（`src/components/FiSegmented.vue`）：3 段，`v-model` 绑定 `activeTab`，每段带图标：

  | 段 | 图标（Lucide） |
  |---|---|
  | 当前比赛 | `activity` |
  | 历史比赛 | `history` |
  | 历史统计 | `chart-column` |

  图标按现有约定新增到 `src/static/ticket-watch/`：每个图标两张（未选中 `#7f8490`、选中 `#ffffff`），24×24、stroke-width 2、round 端点，文件头保留 Lucide 版权注释（参考 `src/static/insights/contribution-*.svg`）；`activity` 与 `history` 的路径可参考 `src/static/user/activity.svg`、`history.svg`；图标路径必须与 Lucide 官方源一致，实现者核对后再写入。
- 组件对外：`defineProps<{ activeTab: TicketWatchTab }>()`、`emit('tab', ...)`（保持事件名，页面里 `@tab="activeTab = $event"` 不变）。

### 3.4 去掉云南玉昆（只改界面，不删逻辑）

- 页面不再提供球队切换：删除 `index.vue` 的 `switchTeam` 函数与头部的 `@team` 绑定。
- `useTicketWatchState` 里 `selectedTeam` 保持常量初值 `'chengdurongcheng'`，**不删除**这个状态、`TicketWatchTeam` 类型、`yunnanyukun` 相关的加载分支和对应测试：它们现在不会被触达，作为休眠代码保留，**后续单独清理**，本期不动以避免回归（`effects.test.ts` 共 2665 行、覆盖大量加载路径）。
- 若有测试通过 `switchTeam` 驱动，做最小改动使其仍然成立，并在最终说明里列出。

### 3.5 `TicketBoardSkeleton.vue`（更新）

头部和比赛卡的骨架形状同步新结构：标签分段占位、比赛卡标题行、状态条占位、两个指标块、一个整行主按钮加两个次按钮占位。继续使用全局 `fi-skeleton-*` 类，去掉骨架里的渐变（见 6 节）。

### 3.6 其余卡片

`TicketRecentRefluxPanel`、`TicketRecentRefluxLock`、`TicketFocusPanel`、`TicketTrackedInterests`、`TicketInventoryPanel`、`TicketSupplyInsights`、`TicketHistoryTrend`、`TicketHistorySelector`、`TicketHistoryMatchCard`：**只做样式层面的统一**（去渐变、颜色/间距/字号/圆角/阴影改引用 token、图标尺寸统一），**不改结构、文案、事件、折叠逻辑**。

## 4. 纯函数与单测（放 `pages/ticket-watch/presentation.ts` 或 `helpers.ts`，必须有单测，先写失败的测试）

```ts
export function formatRefreshInterval(seconds: number): string
// 1→"1 秒"，15→"15 秒"，59→"59 秒"，60→"1 分钟"，90→"1 分 30 秒"，120→"2 分钟"，600→"10 分钟"
// 非有限数或小于 1 时返回空字符串（页面据此不展示间隔）
export function formatFreshness(nowMs: number, loadedAtMs: number | null): string
// null→"尚未更新"；<5s→"刚刚"；<60s→"N 秒前"；<1h→"N 分钟前"；否则"N 小时前"；loadedAtMs 晚于 now 时按"刚刚"
export type TicketRefreshState = 'idle' | 'refreshing' | 'failing' | 'live'
export function resolveRefreshState(input: { monitoring: boolean; refreshing: boolean; pollFailing: boolean }): TicketRefreshState
// 优先级：!monitoring→idle；refreshing→refreshing；pollFailing→failing；否则 live
```

注意：`resolveRefreshState` 中"刷新中"优先于"刷新失败"（重试进行时显示刷新中）。表驱动覆盖全部组合。

## 5. 状态与数据流（`ticketWatchCurrentBoard.ts` / `useTicketWatchState.ts`）

在现有基础上**只新增**两个只读状态，不改现有字段和语义：

- `currentBoardLoadedAtMs: Ref<number | null>`：**客户端**最近一次成功获取当前看板的时间（任何原因：initial、poll、手动刷新）。在 `hasLoadedCurrentBoard.value = true` 的成功路径设置 `Date.now()`；`resetCurrentBoard` 时复位为 `null`。
  - 这是**客户端拉取时间**，不是服务端数据时间（接口没有返回数据时间戳）。UI 文案按"X 前更新"表达即可，实现者不要在文案里写成"数据产生于"。
- `currentBoardPollFailing: Ref<boolean>`：`reason === 'poll'` 且 `hasLoadedCurrentBoard` 为真时请求失败置真；**不要改变现有行为**——仍然保留旧数据、不设置 `currentErrorMessage`、不弹错误卡；下一次成功时置假。`initial` 失败、`resetCurrentBoard`、切换账号/会话时复位为假。
- 另外新增 `isPageVisible: Ref<boolean>`（供 `ticking` 使用）：在页面现有的 `onShow`/`onHide` 里同步设置；不要改变 `pageVisible` 局部变量现有的用途。

页面把这些值接到当前比赛卡：`monitoring = isMonitoringActive`、`refreshing = currentBoardRequestInFlight`、`pollFailing = currentBoardPollFailing`、`intervalSeconds = pollIntervalSeconds`、`lastUpdatedAtMs = currentBoardLoadedAtMs`、`ticking = isPageVisible`。

**状态测试**（沿用 `effects.test.ts` 的写法）：
- 首次加载成功后 `currentBoardLoadedAtMs` 有值；每次 `poll` 成功都会更新；
- `poll` 失败：`currentBoardPollFailing` 变真，**旧数据保留、`currentErrorMessage` 仍为空**；下一次成功恢复为假；
- `initial` 失败的现有行为（显示错误）**不变**；
- 账号/会话切换与 `resetCurrentBoard` 复位两个状态；
- 现有全部测试保持通过。

## 6. 渐变清单（全部改纯色）

交付前在 `src/pages/ticket-watch` 下全文搜索 `gradient`，应无结果。当前各文件的命中行数：

| 文件 | 行数 |
|---|---|
| `components/TicketRecentRefluxPanel.vue` | 9 |
| `components/TicketHistoryTrend.vue` | 8 |
| `components/TicketInventoryPanel.vue` | 5 |
| `components/TicketBoardSkeleton.vue` | 4 |
| `components/TicketRecentRefluxLock.vue` | 4 |
| `components/TicketCurrentMatchCard.vue` | 3 |
| `components/TicketFocusPanel.vue` | 2 |
| `components/TicketTrackedInterests.vue` | 2 |
| `components/TicketSupplyInsights.vue` | 1 |
| `index.vue` | 1 |

处理原则：背景渐变→取其主色的纯色 token；进度条/强调条渐变→取其深色端的纯色 token；遮罩渐变→单色半透明 token；最近回流速览里各档位用渐变区分的，改成各自一个纯色强调色。需要新色值时先在 `src/styles/fi-tokens.css` 加 primitive 再建 semantic 别名；**优先复用已有 token**（尤其是已有的 `--fi-color-primary`、`--fi-color-page`、`--fi-color-text-*`、`rgba(var(--fi-primitive-red-rgb), …)`），**不要为一次性数值新增洞察式的大量专用 token**。

## 7. 必须保持不变的行为（回归点）

1. 轮询：`useTicketWatchPolling.ts` 的启动、停止、间隔、页面可见性逻辑不改；监控开关、会员锁定（取关公众号）、审核态行为不变。
2. 会员与付费：最近回流速览的锁定/解锁规则、`upgrade` 跳转、比赛 ID 弹层（V6 免费 / ¥5 解锁）、回流提醒订阅弹层的行为不变。
3. **付费度量**：`index.vue` 里的 `handleRecentRefluxUpgrade`、`handleMatchIdSheetUpgrade`、`reportRecentRefluxLockImpression`、两个 `watch`、`beginPaywallVisit()` 一行不改，`TicketRecentRefluxPanel` 的 `@upgrade`、`TicketMatchIdSheet` 的 `@upgrade` 绑定不变。
4. 关注区块、库存卡折叠、热区、供给洞察的数据与交互不变；历史比赛、历史统计标签的功能不变。
5. 页面进入遮罩（`pageEntered`）、分享、返回逻辑不变。
6. `effects.test.ts`、`helpers.test.ts` 与 `paymentActions.test.ts`、`purchaseTelemetry.test.ts` 等现有测试全部通过。

## 8. 约束

1. 只改 `football_insight_mini`，**不改后端、不改接口**；不要读取、搜索或修改 `football_insight_admin_android`。
2. 所有颜色、间距、字号、行高、圆角、边框、阴影、字重引用 token，不新增散落 hex。
3. 组件层不得 import `api/**`；页面间不得互 import；不得往 `scripts/check-import-boundaries.mjs` 的 ALLOWLIST 加例外。
4. 不要手改 `manifest.json` 的 `versionName`；**不要上传小程序**，由审查后统一执行。
5. 页面所有改动小步提交，提交信息 `feat(mini): ...`，subject 用英文，末尾加 Co-Authored-By 行；直接提交并推送 `main`，不新建分支；先 `git pull --rebase`，工作区若有不属于你的未提交改动不要碰。
6. 同步在 `CHANGELOG.md`、`FEATURES.md` 各补一条并写明原因。
7. 验证命令：`bun test`、`bun run type-check`、`bun run check:boundaries`、`bun run verify:mp-components`。视觉无法在终端验证，最终说明里逐条写清每个组件的结构、用到的 token 与参照的现有样式出处。

## 9. 范围外

- 把刷新速度做成升级引导（例如"升级 V6 每 15 秒刷新"的对比和入口）——属于变现方向，需要给付费度量新增入口白名单（后端改动），另行设计；
- 清理 `yunnanyukun` 相关休眠代码；
- 轮询策略、接口、数据口径；
- 最近回流速览的结构调整；
- 页面其他标签的信息结构调整。

## 10. 审查清单（Review 逐条核对）

1. 头部 hero 卡、球队切换、会员胶囊已删除；标签切换用共享 `FiSegmented`，3 段都有图标，图标文件按约定新增且路径与 Lucide 源一致。
2. 刷新状态条四种状态（未开启、刷新中、刷新失败、监控中）逻辑与文案和第 3.1 节一致；间隔与相对时间格式化函数有表驱动单测且边界正确。
3. `currentBoardLoadedAtMs` 只在成功路径更新；`currentBoardPollFailing` 只在 poll 失败（且已加载过）时置真，**旧数据保留且不出现错误卡**；`initial` 失败的原有行为不变；会话切换与重置时复位。
4. 状态条的滚动刷新在组件内部，**没有**用页面级每秒状态；页面不可见时停止；卸载时清理定时器。
5. 轮询、监控、会员、订阅、比赛 ID、关注区块、折叠逻辑零改动；付费度量的处理函数与 `watch` 原样存在。
6. 去掉云南玉昆只是界面层：`selectedTeam` 恒为 `'chengdurongcheng'`，休眠代码与对应测试保留。
7. `src/pages/ticket-watch` 下无 `gradient`；无散落 hex；没有为一次性数值新增大批专用 token。
8. 骨架屏与新结构形状一致，使用全局 `fi-skeleton-*`。
9. 当前比赛卡的按钮层级：主按钮整行、两个次按钮并排；事件契约 `monitor`、`subscribe`、`match-id` 不变。
10. 无 ALLOWLIST 新增、无跨页面 import；`bun test`、类型检查、边界检查、组件注册检查全部通过。
