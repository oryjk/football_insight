# 小程序大文件拆分：实现计划

适用项目：`football_insight_mini`。执行者：实现 agent。审查者：规划 agent（本计划作者）。

## 开工前（必做）

1. 在 local233 先同步最新代码，再开新分支，不要在旧分支上继续：
   ```bash
   cd /home/betalpha/projects/football_insight
   git fetch origin && git switch main && git pull --ff-only
   git switch -c codex/large-file-splitting
   ```
   上一轮弹层整理（`codex/sheet-component-cleanup`）已合入 `main`，本计划基于它之后的代码。
2. 先读：`football_insight_mini/AGENTS.md`、`docs/mini-architecture.md`（拆分模式与职责边界）、`docs/mini-design-system.md`、本计划。
3. 跑一遍基线，记录结果，之后每个阶段对比：
   `bun run type-check`、`bun run check:boundaries`、`bun test`（基线应为 641 个通过）。

## 背景：拆分依据与数据

`AGENTS.md` 规定：超过约 600 行主动评估，超过约 1000 行必须拆；按**变化原因**拆，不按行数机械拆。本仓库当前没有任何文件超过 1000 行，所以都是"建议拆"。

读过源码后，各文件的行数构成如下（`.vue` 为 template / script / style 行数）：

| 文件 | 总行数 | 构成 | 结论 |
|---|---|---|---|
| `pages/team-board/index.vue` | 728 | 238 / 221 / 267 | **拆**：唯一没拆过的页面，请求写在页面里；快照模板重复了两份 |
| `pages/ticket-watch/helpers.ts` | 906 | 纯函数 + 类型，涉及 8 个业务域 | **拆**（门面模式，见下） |
| `pages/home/helpers.ts` | 742 | 纯函数 + 类型，涉及 5 个业务域 | **拆**（门面模式） |
| `pages/ticket-watch/useTicketWatchBoards.ts` | 751 | 一个 composable 里塞了当前看板和历史看板 | **拆，风险最高，放最后** |
| `pages/seat-swap/index.vue` | 626 | 97 / **421** / 106 | **拆 script**：提交、删除、确认、取消等动作都在页面里 |
| `pages/seat-swap/components/SeatSwapPublishSheet.vue` | 686 | 151 / 275 / 258 | **评估后拆表单状态**（三步向导的状态与步骤逻辑） |
| `pages/rankings/components/RankingsSurfacePanel.vue` | 672 | 137 / 177 / **356** | **拆模板**：球队榜与球员榜是两个独立区块 |
| `pages/user/index.vue` | 622 | 102 / 279 / 239 | **不拆**：登录与账号副作用已抽到 `accountEffects.ts`、`loginFlow.ts`，剩下是接线和展示标签，再拆只增加跳转成本 |
| `pages/rankings/components/RankingsTeamSeasonSheet.vue` | 607 | 117 / **95** / **393** | **不拆**：393 行是样式，逻辑只有 95 行，拆组件不会减少复杂度 |

> 实现者发现表中判断与代码不符（例如某文件其实更适合不拆），停止并汇报，不要自行改变范围。

## 通用约束

- **纯重构，不改任何行为、文案、样式、接口调用、路由。** 抽出的函数/组件，输入输出与原来逐一对应。
- 运行时 Vue 组件必须直接从 `.vue` 文件导入，不要通过 barrel 二次导出（会导致小程序 `usingComponents` 缺失）。纯函数与类型可以走门面文件。
- 组件层（`components/`）不得 import `api/**` 与认证存储；请求和副作用留在页面或 `use*`/`*Actions` 里。页面之间不得互相 import。不得往 `check-import-boundaries.mjs` 的 `ALLOWLIST` 新增条目。
- 小程序有组件样式隔离：把模板搬进子组件时，**样式要一起搬**，跨组件共用的样式放进页面级 `*-shared.css`，用 `@import` 引入（现有模式：`ticket-watch/ticket-watch-shared.css`、`matches/matches-shared.css`）。不要依赖父组件 scoped 样式去给子组件根节点布局。
- 新增样式引用 `--fi-*` token，不新增散落 hex；迁移已有样式时保持原值。
- 不动 `football_insight_admin_android`；不改后端；不改 `manifest.json` 版本号。
- 每个阶段单独提交，英文 subject，例如 `refactor(mini): split team-board page into components and composable`。**不要 push，不要 `mp:release`，不要部署。**
- 现有测试不得删除或放宽断言。只有抽出了有业务风险的纯函数（见阶段 1 的看板状态更新函数）才补测试。
- 每个阶段结束都跑：`bun run type-check`、`bun run check:boundaries`、`bun test`，阶段 1、4 额外跑 `bun run build:mp-weixin`（`MINI_REVIEW_SKIP=1`），确认构建尾部无 `Unregistered mini-program components`。
- 改完更新 `docs/mini-architecture.md` 中"页面拆分进度"一段，并在 `CHANGELOG.md` 记一条（写明原因，如实写哪些没在微信开发者工具验证）。

## 阶段 1：`team-board` 页面拆分

**现状**：`index.vue` 里同时有加载（含 401 跳转登录）、发布观点、评论、点赞，以及一个自绘的发布浮层。洞察快照（`team-board-snapshot`）的模板在"帖子"和"发布浮层"里各写了一遍，结构完全相同。

**目标结构**

```text
src/pages/team-board/
  index.vue                      # 只做编排：onLoad/onShow、导航、把状态传给子组件
  useTeamBoardPage.ts            # 新增：board/loading/errorMessage、loadBoard、publishPost、submitComment、toggleLike、composer 状态
  helpers.ts                     # 保留 resolveTeamBoardLoadFailure；新增展示函数与看板状态更新纯函数
  team-board-shared.css          # 新增：多个子组件共用的样式
  components/
    TeamBoardHero.vue            # 顶部英雄卡（球队名、排名、说明、返回按钮）
    TeamBoardLauncher.vue        # "先选一张洞察卡"发布入口网格
    TeamBoardSnapshot.vue        # 洞察快照（帖子与发布浮层共用，消除重复）
    TeamBoardPostCard.vue        # 单个帖子：作者、正文、快照、点赞、评论区
    TeamBoardComposer.vue        # 发布观点浮层
```

**要点**

1. `formatShare`、`metricSuffix`、`formatPublishedAt` 搬到 `helpers.ts`（展示转换放 helpers，不散落在模板里）。
2. 看板状态的三处"不可变更新"（插入新帖、追加评论并 `comment_count + 1`、更新点赞摘要）抽成 `helpers.ts` 里的纯函数，例如 `prependPost(board, post)`、`appendComment(board, postId, comment)`、`applyLikeSummary(board, postId, summary)`，并在 `helpers.test.ts` 补对应测试（这些是数据提交后的关键状态变化，值得测）。
3. 子组件只通过 props 接收数据、通过 emits 发出意图（如 `submit-comment`、`toggle-like`、`open-composer`、`close`、`publish`）。**子组件里不得调用 `api/teamBoard`。** 评论草稿 `commentDrafts` 的归属由实现者选择（放 `useTeamBoardPage` 或 `TeamBoardPostCard` 内部），但要保证"评论成功后清空、失败后保留"的行为不变。
4. 发布浮层本次**保持自绘实现**，不要顺手改成 `FiBottomSheet`（那是另一个改动，会带来视觉变化）。
5. 401 处理（`session.invalidateSession(requestRevision)` + `rememberPostLoginRedirect` + `uni.switchTab`）必须原样迁移到 `useTeamBoardPage.ts`，不得简化。

**验收**

- `index.vue` 的 script 部分降到约 100 行以内，总行数明显低于 600。
- 快照模板只剩 `TeamBoardSnapshot.vue` 一份。
- 三项检查 + 构建通过，新增的纯函数测试通过。

## 阶段 2：两个 helpers 文件按业务域拆分（门面模式）

**为什么用门面**：`ticket-watch/helpers.ts` 被 17 个文件 import，`home/helpers.ts` 被 8 个文件 import，另有 `helpers.test.ts`。为了把风险压到最低，**保留 `helpers.ts` 作为门面**，里面只做 `export * from './xxx'` 的重导出；所有现有 import 路径与测试文件不用改。纯函数与类型走门面符合 `AGENTS.md` 的 barrel 规则。

### 2a. `ticket-watch/helpers.ts`（906 行）

按函数现有的业务域分组（行号为参考，以实际代码为准）：

| 新文件 | 内容 |
|---|---|
| `ticketWatchLoadHelpers.ts` | `TicketWatchLoadReason`、`TicketWatchHistoryLoadReason`、`TicketWatchCurrentLoadStrategy`、`resolveCurrentBoardLoadStrategy`、`resolveHistoryBoardLoadStrategy`、`selectCompletedMatches`、`selectRefluxStatsMatches`、`buildInventorySince`、`requireInventorySince` |
| `inventoryHelpers.ts` | `groupInventoryByPrice`、`summarizeInventoryBoard`、`resolveInventoryHeatLevel`、`resolveInventoryPriceTone`、`prioritizeInventorySections`、`isVip*`、`getInventoryEmptyStateLabel`、`formatInventorySectionSummary` 及相关类型 |
| `interestHelpers.ts` | `applyBlockInterest(s)ToSections`、`resolveBlockInterestHeatLevel`、`buildTrackedInterestSummary`、`formatTrackedInterest*`、折叠状态相关（`isTicketWatchSectionCollapsed`、`toggleTicketWatchSectionCollapsed`）及相关类型 |
| `refluxHelpers.ts` | `buildRecentRefluxBuckets`、`resolveRecentRefluxPanelMode`、`resolveInventoryRefluxFreshness`、`resolveRecentRefluxBucketRequiredTier`、`isRecentRefluxBucketUnlocked`、`buildHistoryRefluxTrend` 及相关类型 |
| `subscriptionHelpers.ts` | `isValidNotificationEmail`、`formatRefluxSubscriptionPrice`、`selectPurchasableRefluxSubscriptionPlans`、`isRefluxSubscriptionActiveForCurrentMatch`、`formatTicketWatchMembershipBadgeTier` |
| `matchIdHelpers.ts` | `TicketMatchIdSheetState`、`resolveMatchIdSheetState`、`buildMatchIdSourceLabel`、`buildTicketWatchMatchLabel` |

注意：

- 私有函数（如 `resolveTicketWatchBlockKey`、`resolveHistoryRefluxOpponentName`、`extractOffsetMinutes`、`formatDateWithOffset`）跟着使用它的函数走；如果被多个新文件共用，放进一个 `shared` 内部文件，不要重复实现。
- 先搞清楚函数之间的内部调用关系再分组，避免新文件之间形成循环依赖。
- `isValidNotificationEmail` 在 `user/helpers.ts` 里也有一份。页面之间不能互相 import，**本次不合并**；如发现两份实现不一致，在汇报里说明即可。

### 2b. `home/helpers.ts`（742 行）

| 新文件 | 内容 |
|---|---|
| `homeLoadHelpers.ts` | `HomeLoadPlan`、`resolveHomeHasAuthToken`、`resolveHomeLoadPlan`、`isHomeAuthExpiredError`、`shouldShowHomeSupportLoading` |
| `homeSupportHelpers.ts` | `resolveHomeSupport*` 系列 |
| `homePulseHelpers.ts` | `HomePulseLeadMatch`、`HomePulseTechStat`、`resolveHomePulse*`、`formatHomePulseCornerText`、`HOME_PULSE_RESULT_FRESH_WINDOW_MS`、技术统计相关私有函数 |
| `homeGuideHelpers.ts` | `HomeHeroGuide`、`HomeBriefingItem`、`HomeGuideLeaders`、`resolveHomeGuide*`、`buildHomeHeroGuide`、`buildHomeBriefingItems`、`buildHomeHeadlineBody`、`buildHomeWatchPoints`、`formatHomeEntityNames` 及相关私有函数 |
| `homeTeamSeasonHelpers.ts` | `HomeTeamSeasonMatch`、`resolveHomeTeamSeasonMatches`、`formatHomeTeamSeasonRecord` 及排序相关私有函数 |

**验收（2a、2b 共用）**

- `helpers.ts` 只剩重导出，导出符号集合与改动前**完全一致**（用脚本比对前后 `Object.keys(await import(...))` 或 `tsc` 声明导出，汇报里给出比对结论）。
- 每个新文件不超过约 300 行，且不出现循环 import。
- `helpers.test.ts`、`effects.test.ts` 等**一行不改**，全部通过。
- 其他文件的 import 路径一行不改。

## 阶段 3：`useTicketWatchBoards.ts` 拆分（风险最高）

**现状**：一个 `useTicketWatchBoards(state, refreshRefluxSubscriptionStatus, api, session)` 同时负责当前看板（含成都/渝坤两个球队变体）、历史列表与历史看板选择、历史趋势合计，以及一套"请求 owner / 序号"机制来防止旧请求写入新状态。`effects.test.ts`（约 2700 行）重度覆盖这一块，是你的安全网。

**要求**

1. **对外签名与返回值完全不变**：仍然导出 `useTicketWatchBoards(state, refreshRefluxSubscriptionStatus, api, session)`，返回同样的 9 个成员（`ensureRegions`、`loadViewerMembershipTier`、`loadCurrentTrackedInterests`、`loadCurrentBoard`、`loadHistoryMatches`、`loadBoardForTab`、`resetCurrentBoard`、`resetHistoryBoard`、`handleHistoryMatchSelect`）。调用方 `index.vue`、各 `useTicketWatch*` 和测试都不改。
2. 内部按职责拆成同目录文件，例如：`ticketWatchCurrentBoard.ts`（当前看板，含两个球队变体）、`ticketWatchHistoryBoard.ts`（历史列表、选择、趋势）、`ticketWatchBoardRequests.ts`（请求 owner/序号这套共享机制）。具体切法由实现者读完代码后决定，但**请求 owner 机制只能有一份实现**，不能复制。
3. 拆分前先写下函数之间的依赖图（谁调用谁、共享哪些闭包变量：`currentBoardRequestSeq`、`activeCurrentBoardRequest`、`historyBoardRequestSeq`、`historySelectionRequestToken`、`historyTrendRequestSeq` 等），放进汇报。共享的可变变量要显式传递或封装进一个对象，不要靠模块级全局变量（会让多个组件实例串状态）。
4. **不要顺手合并成都/渝坤的重复加载函数，也不要改任何并发/竞态语义。** 去重是后续另一个改动。
5. 如果读完代码后认为这一拆分无法在不改变行为的前提下完成，或收益不足以覆盖风险，**停止并汇报**，不要硬拆。

**验收**：`bun test` 全过且测试文件无改动；`useTicketWatchBoards.ts` 降到约 300 行以内（只剩装配）；汇报里给出依赖图和每个共享变量的归属。

## 阶段 4：页面 script 与局部组件拆分

### 4a. `seat-swap/index.vue`

script 有 421 行，其中提交发布、删除申请、确认候选、取消确认、已匹配后的取消、订阅消息授权都是"页面级提交动作与 API 编排"。按现有约定抽成 `seat-swap/useSeatSwapActions.ts`（或 `seatSwapActions.ts`），页面保留生命周期、加载、导航、弹层开关、滚动与区域展开等编排。要点：

- 保留 `submitting` 等状态的归属不变；动作函数通过参数/依赖注入拿到页面状态，不要读全局。
- `USE_SEAT_SWAP_MOCK_LARGE_DATA` 这类调试开关原样保留。
- `pageScrollLocked` / `page-meta` 滚动锁必须仍留在 `index.vue`（`page-meta` 只能放在真正的页面入口）。
- 弹层 `v-model:visible` 的接线不变。

### 4b. `SeatSwapPublishSheet.vue`

把三步向导的状态与步骤逻辑（`selectionStep`、`stagedCurrentRegionKey`、`stagedDesiredSeats`、`formErrors`、`form`、各 `can*`/`confirm*`/`goPrevious*`/`jumpToStep`、`resetFormForNewRequest`、`fillFormFromMine`）抽成同目录 composable `useSeatSwapPublishForm.ts`，组件只保留模板、props/emits 与对 `FiBottomSheet` 的接线。**先评估**：如果抽出后 composable 与组件之间需要传递的参数过多（超过约 8 个），说明边界不对，停止并汇报。

### 4c. `RankingsSurfacePanel.vue`

球队榜（`v-if="scope === 'team'"`）与球员榜（`v-if="scope === 'player'"`）是两个互不依赖的区块。拆成 `RankingsTeamList.vue`、`RankingsPlayerList.vue`，父组件保留榜单类别切换与滚动位置等共享状态。样式随各自区块搬走，共用样式放 `rankings-shared.css`。注意 `FiStandingsTable`、`FiPlayerGoalsTable`、`TeamAbilityHexagon` 仍直接从 `.vue` 导入。

**验收**：三项检查 + 构建通过；各文件行数明显下降（`seat-swap/index.vue` script 目标约 200 行内，`RankingsSurfacePanel.vue` 目标约 350 行内）。

## 明确不做

- `user/index.vue`、`RankingsTeamSeasonSheet.vue`：见上表，不拆。
- 其他 500 行上下的文件（`ticket-watch/index.vue` 573、`support/index.vue` 546、`home/index.vue` 539、`membership-purchase/index.vue` 507 等）：本次不动。
- 任何去重、命名统一、样式 token 迁移、行为调整、新功能。
- `ticket-watch` 的成都/渝坤重复加载函数合并。

## 汇报格式（给审查者）

按阶段 1–4 分别写：

1. 改动文件与 `git diff --stat`；各目标文件拆分前后的行数对比。
2. 每条"要点/要求"如何处理；偏离计划的地方及理由。
3. 三项检查（与构建）的实际结论，测试数量对比基线 641。
4. 阶段 2：导出符号集合前后对比的结论。阶段 3：依赖图与共享变量归属。
5. **没有实际验证过的项必须写"未验证"**（尤其是微信开发者工具里的表现），不要写成"已确认"。
6. 提交哈希列表。

## 审查者检查清单

- 重构范围：没有行为/文案/样式/接口调用变化；没有无关文件改动。
- 阶段 1：快照模板只剩一份；子组件无 `api` import；401 流程完整迁移；评论成功清空、失败保留；新纯函数有测试。
- 阶段 2：`helpers.ts` 仅为重导出；导出符号集合不变；无循环依赖；测试文件零改动。
- 阶段 3：对外签名与返回成员不变；请求 owner 机制只有一份；没有模块级可变全局；`effects.test.ts` 零改动。
- 阶段 4：`page-meta` 仍在页面入口；`ALLOWLIST` 未新增；子组件样式随模板迁移。
- 文档：`docs/mini-architecture.md` 的拆分进度已更新；`CHANGELOG.md` 如实记录未验证项。

## 开发者工具人工测试清单（供测试人员使用）

> **重要：本地后端连接的是生产库 `football_data`。** 下列涉及发布、评论、点赞、换座申请、取消、确认的操作都会**真实写入生产数据**。请只用专门的测试账号操作，能只看界面就不要提交；必须提交时记录下来，事后自行清理，不要用真实用户的数据做测试。

### 阶段 1：`pages/team-board/index?teamId=<球队 id>`（从洞察页进入球队战术板；需要已登录）

| 场景 | 看什么 |
|---|---|
| 首次进入 | 加载态 → 英雄卡（球队名、排名）→ 发布入口网格 → 帖子列表，布局和改动前一致 |
| 球队没有帖子 | 显示空状态文案 |
| 未登录进入 | 跳转到"我的"页登录，登录后回到该战术板 |
| 登录过期（401） | 同上，且不会反复跳转 |
| 点发布入口卡片 | 弹出发布浮层，顶部显示该洞察卡与快照，输入框为空 |
| 浮层关闭 | 点遮罩与"关闭"按钮都能关；关闭后再打开内容已清空 |
| 发布观点 *(写生产)* | 发布成功提示，新帖出现在列表最前；标题为空/内容为空时显示错误文案且浮层不关 |
| 评论 *(写生产)* | 评论成功后输入框清空、评论数 +1；评论失败时输入内容保留 |
| 点赞 *(写生产)* | "支持"按钮高亮状态与数字更新 |
| 分享与返回 | 右上角分享链接带 `teamId`；返回洞察页正常，没有上一页时回到洞察 tab |

### 阶段 2、3：`pages/ticket-watch/index` 与 `pages/home/index`

**ticket-watch**

| 场景 | 看什么 |
|---|---|
| 当前场次 tab | 库存按价位分组，热度/余票文案、VIP 区块、折叠展开 |
| 历史场次 tab | 历史列表、切换场次、回流趋势图；**快速连续切换多个历史场次**，最终显示的必须是最后点的那场，无闪烁错乱 |
| 切换球队（如页面提供成都/渝坤切换） | 两支球队的当前看板和历史列表各自正确，来回切换不串数据 |
| 会员权益锁 | 非会员/低等级时，近期回流面板按档位显示锁定或解锁 |
| 关注区块（兴趣） | 关注/取消关注后热度与"已关注"汇总更新 *(写生产)* |
| 比赛 ID 弹层、订阅弹层 | 能打开、关闭，文案与状态正确 |
| 轮询 | 停留数十秒，数据能刷新，且不影响正在查看的历史场次 |
| 审核态 | 版本处于审核态时，受限功能不展示 |

**home**

| 场景 | 看什么 |
|---|---|
| 首屏 | 英雄引导文案、简报跑马灯、观赛要点 |
| 本轮战报/脉冲 | 最近一轮结果、角标文案、技术统计条 |
| 主队/助力面板 | 主队下一场标签、剩余天数、排名文案；未登录与已登录两种状态 |
| 球队赛季弹层、技术统计弹层 | 能打开，赛季战绩汇总（如"x胜x平x负"）正确 |
| 登录过期 | 首页清登录态并引导登录，不报错 |

### 阶段 4：`pages/seat-swap/index` 与 `pages/rankings/index`

**seat-swap**（写操作较多，只用测试账号）

| 场景 | 看什么 |
|---|---|
| 页面浏览 | 区域分组展开/收起、点球场图筛选、清除筛选、滚动到指定区域 |
| 发布弹层 | 三步向导：选当前区域 → 选想换的区域 → 确认；"上一步"、点步骤条跳转、校验错误提示 *(提交写生产)* |
| 管理弹层 | 查看我的申请、编辑（回到向导且已预填）、删除申请 *(写生产)* |
| 候选确认/取消 | 确认某候选、取消确认、匹配后取消 *(写生产)* |
| 订阅消息授权 | 小程序里首次发布/确认时弹出订阅授权；拒绝时流程不卡住 |
| 滚动锁 | 任一弹层打开后，底层页面不能滚动 |

**rankings**

| 场景 | 看什么 |
|---|---|
| 球队榜 | 各榜单类别切换、积分榜、能力雷达图、点击球队打开赛季弹层 |
| 球员榜 | 各类别切换、射手榜表格、点击球员打开球员弹层与能力值 |
| 类别横向滚动 | 切换类别时位置保持合理，与改动前一致 |
| 内嵌赛程 | 榜单页里的赛程视图（`MatchesContent`）正常 |
| 海报 | 榜单海报生成与预览 |

### 无需测试（本计划不动）

`user` 页、排行榜的两个弹层样式、`support`、`membership-purchase`、`topics`。如果测试时发现这些页面变了，说明改动越界，请直接反馈。
