# 小程序弹层与共享组件整理：实现计划

适用项目：`football_insight_mini`。执行者：实现 agent。审查者：规划 agent（本计划作者）。

## 背景与结论

代码审查发现两处结构问题：

1. `src/components/` 里的 `SeatSwapCandidateCard.vue`、`SeatSwapManageSheet.vue` 只被 `pages/seat-swap` 使用，却放在跨页面共享目录里。
2. 各页面有 13 个 `*Sheet.vue`，不含 `FiBottomSheet`：
   - `user` 的 3 个、`ticket-watch` 的 2 个、`rankings` 的 3 个、`matches` 的 2 个，各自重写了遮罩/卡片/关闭按钮。
   - `home` 的 3 个走页面级封装 `home/components/HomeSheetShell.vue`。

**重要：`FiBottomSheet` 不能直接替换这些弹层。** 读过源码后确认的差异：

| 项 | `FiBottomSheet` | 现有表单类弹层（如 `UserNotificationEmailSheet`） |
|---|---|---|
| 高度 | 固定高度，默认 `88vh`，`height` 与 `maxHeight` 同值 | 随内容自适应 |
| 内容区 | 固定用 `<scroll-view>` | 普通 `view` |
| 头部 | 有 title/eyebrow 才渲染，带 `✕` 关闭 | 自定义 kicker/title，无 `✕` |
| 层级 | `z-index: 40` | `80`（user）、`90`（ticket-watch 的 `TicketMatchIdSheet`） |
| 底部留白 | 无 `--window-bottom` | `bottom: var(--window-bottom, 0px)`（H5 有底栏时避让） |
| 显隐 | `visible` 变 false 后延迟 220ms 才卸载（关闭动画） | `v-if="visible"` 立即卸载 |

所以第一步是给 `FiBottomSheet` 加**向后兼容的可选 prop**，再迁移少量试点，每一步都要人工复核后再推进。

## 范围

**本计划做：**

- 任务 A：把两个 `SeatSwap*` 组件挪回 `pages/seat-swap/components/`。
- 任务 B：给 `FiBottomSheet` 增加内容自适应高度等可选能力，并更新契约文档。
- 任务 C：试点迁移 `user` 页的 3 个表单弹层到 `FiBottomSheet`。

**本计划不做（后续另开计划，不要顺手做）：**

- `RankingsPlayerSheet`、`RankingsTeamSeasonSheet`、`RankingsPosterSheet`：形态特殊（榜单/海报），保持现状。
- `MatchesRoundSheet`、`MatchesTechStatsSheet`：是带四周留白的浮层卡片，不是贴底 sheet，保持现状。
- `TicketMatchIdSheet`、`TicketSubscriptionSheet`：层级 21/90 与页面其他浮层有叠放关系，需单独评估。
- `HomeSheetShell` 及 3 个 home 弹层：是否并入 `FiBottomSheet` 等任务 C 验收后再评估。
- `team-board/index.vue` 拆分、`ticket-watch/helpers.ts` 拆分。
- 任何视觉改版、token 迁移、动效改动。

## 通用约束

- 先读：`football_insight_mini/AGENTS.md`、`docs/mini-architecture.md`、`docs/mini-design-system.md`（尤其 `FiBottomSheet` 一节与「弹框滚动锁约定」）。
- 运行时 Vue 组件必须直接从 `.vue` 文件导入，不要新增 barrel 二次导出。
- 样式用 `rpx`；新增样式引用 `--fi-*` token，不新增散落 hex。`FiBottomSheet` 现有的写死颜色**不要顺手改**。
- 统一使用 `uni.*`，不要直接用 `wx.*`、`document.*`、`window.*`。
- 不改 `src/pages/**` 之外与本计划无关的文件；不动 `football_insight_admin_android`。
- 不得提交 `.env`、密钥；**不要执行 `bun run mp:release`、不要 push、不要部署**。提交只在本地，由审查者确认后再决定推送。
- 每个任务单独一个提交，提交信息用英文 subject，例如 `refactor(mini): move seat-swap-only components into the page`。
- 改动只做增量，不重写页面风格。

## 任务 A：挪回 SeatSwap 组件

**目标**：`SeatSwapCandidateCard.vue`、`SeatSwapManageSheet.vue` 从 `src/components/` 移到 `src/pages/seat-swap/components/`。

**步骤**

1. 确认引用：`grep -rn "SeatSwapCandidateCard\|SeatSwapManageSheet" src`。已知只有 `pages/seat-swap/index.vue` 和 `pages/seat-swap/components/SeatSwapPoolList.vue`；若发现其他引用，停止并汇报。
2. 用 `git mv` 移动两个文件，保留历史。
3. 修正两个文件内部的相对路径 import，以及上述两处引用方的 import 路径。仍然直接从 `.vue` 导入。
4. 确认这两个组件不 import `api/**` 与认证存储（页面局部组件同样受边界规则约束）。

**验收**

- `bun run type-check` 通过。
- `bun run check:boundaries` 通过，且 `scripts/check-import-boundaries.mjs` 的 `ALLOWLIST` 没有新增条目。
- `bun test` 通过。
- `bun run build:mp-weixin` 构建通过，尾部组件注册检查无 `Unregistered mini-program components`。
- `src/components/` 下不再有 `SeatSwap*`。

## 任务 B：给 `FiBottomSheet` 增加可选能力

**目标**：让 `FiBottomSheet` 能承载「内容自适应高度」的表单弹层，同时**不改变现有 4 个使用方的表现**（`MembershipRulesSheet`、`MembershipAgreementSheet`、`SeatSwapPublishSheet`、`SeatSwapManageSheet`）。

**新增 props（全部可选，默认值保持现有行为）**

- `fitContent?: boolean`，默认 `false`。为 `true` 时卡片高度随内容，`max-height` 取 `height`（默认仍是 `88vh`），不再固定 `height`。
- `zIndex?: number`，默认 `40`。通过内联样式或 CSS 变量作用于遮罩层。
- `respectWindowBottom?: boolean`，默认 `false`。为 `true` 时遮罩 `bottom` 使用 `var(--window-bottom, 0px)`，对齐 `user` 弹层现有的 H5 底栏避让。

**实现要点与风险（必须逐条处理并在汇报中说明结果）**

1. `<scroll-view>` 在自适应高度的 flex 容器里，微信小程序下可能高度塌陷或无法滚动。`fitContent` 为 `true` 时，优先尝试给 `scroll-view` 设 `max-height`；如果在微信开发者工具里无法正常显示/滚动，改为 `fitContent` 模式下用普通 `view` + `overflow-y: auto` 作为内容容器。**必须在开发者工具验证，不能只看 H5。**
2. 保持现有的 `@touchmove.stop.prevent`（遮罩）与 `@touchmove.stop`（内容）防穿透。
3. 保持 220ms 关闭动画逻辑不变。
4. 默认参数下 DOM 结构与 class 不变，避免影响现有 4 个使用方。
5. 不要改写 `FiBottomSheet` 现有颜色/动效数值。

**文档**：更新 `docs/mini-design-system.md` 的「FiBottomSheet」小节，把新 props 写进契约（含默认值与用途）。该小节标题含「记录不改实现」，更新时改为说明「新增均为向后兼容的可选 prop」。

**验收**

- 默认参数行为与改动前一致：对上述 4 个现有使用方逐个确认模板与调用没有被改动，且在开发者工具中打开仍与之前一致（汇报里附一句各自确认结果）。
- `fitContent` 在内容很少（约 3 个控件）与内容很长（超出 88vh）两种情况下都表现正确：前者贴合内容，后者可在弹层内滚动。
- `bun run type-check`、`bun run check:boundaries`、`bun test`、`bun run build:mp-weixin` 全部通过。

## 任务 C：试点迁移 `user` 的 3 个表单弹层

**对象**：`UserNotificationEmailSheet.vue`、`UserPasswordLoginSheet.vue`、`UserWechatBindSheet.vue`，以及它们的使用方 `pages/user/index.vue`（含 settings 页若有引用）。

**映射**

- 外层 `sheet-mask`/`sheet-card` 替换为 `<FiBottomSheet>`，使用 `fitContent`、`:z-index="80"`、`respect-window-bottom`。
- 原 `section-kicker` → `eyebrow`，`section-title` → `title`；删除各文件内重复的 mask/card 样式。
- 原「取消 / 确认」按钮行放进 `footer` slot（或保留在默认 slot，以视觉最接近原样为准）；保留 `.primary-action`、`.sheet-actions` 等业务样式。
- 原来的 `close()` 逻辑保持：保存中（`saving`）时不允许关闭，此时需同时禁用遮罩关闭（`:close-on-mask="!saving"`），并处理头部 `✕`：保存中点 `✕` 也不应关闭。**若 `FiBottomSheet` 当前无法在保存中拦截 `✕`，不要自行大改，汇报并建议是否加 `closable` prop。**
- 事件保持：`update:visible`、`save` 等对外契约**不变**，父页面 `user/index.vue` 的接线尽量不动。

**特别注意**

- `UserWechatBindSheet` 用 `v-if="bindState"` 控制显隐，没有 `visible` prop。迁移时需要在组件内派生 `visible = !!bindState`。`FiBottomSheet` 关闭时会保留 220ms 动画，期间如果父级已清空 `bindState`，内容会瞬间变空（闪烁）。必须处理：关闭动画期间保留最后一份快照，或确认父级不会在关闭瞬间清空。汇报里写清处理方式。
- 三个弹层原来都没有 `page-meta` 滚动锁（锁属于页面入口）。**本任务不新增、不改变滚动锁行为**；如发现迁移后底层页面出现滚动穿透，汇报现象，不要自行加 `page-meta`。
- `UserPasswordLoginSheet` 有 `max-width: 720rpx; margin: 0 auto`（宽屏居中）。`FiBottomSheet` 目前是满宽。**不要为此改 `FiBottomSheet`**；如视觉差异明显，汇报后再定。
- 输入框（`input`）在 `scroll-view` 里可能出现键盘遮挡或聚焦偏移问题，需在开发者工具里实际点击输入框验证。

**验收**

- 三个弹层打开、关闭、保存中、保存失败、遮罩点击、软键盘弹出（输入框聚焦）均与迁移前一致；差异必须逐条列出。
- 每个文件删除重复的 mask/card 样式后，组件文件行数净减少。
- `bun run type-check`、`bun run check:boundaries`、`bun test`、`bun run build:mp-weixin` 全部通过。
- `src/pages/user/helpers.test.ts`、`loginFlow.test.ts` 等相关测试不得改弱（不得删除或放宽断言）。

## 汇报格式（实现 agent 完成后给审查者）

按任务 A / B / C 分别写：

1. 改了哪些文件（`git diff --stat`）。
2. 上面每条「必须处理」的风险：怎么处理的，在哪个环境验证的（H5 / 开发者工具 / 仅类型检查）。**没有实际验证过的要明确写「未验证」，不要写成「已确认」。**
3. 四条验证命令的实际输出结论。
4. 任何偏离本计划的地方及理由。
5. 提交哈希列表。

## 审查者检查清单

- `git diff` 里没有任何 `src/pages/**`、`src/components/**` 之外的无关改动，没有 `football_insight_admin_android` 改动。
- 任务 B 默认参数下，`FiBottomSheet` 的模板结构、class、动画逻辑与改动前一致，4 个现有使用方文件没有被改。
- 任务 C 的对外契约（props/emits）未变，父页面接线改动最小；`saving` 期间的关闭拦截没有被削弱。
- `UserWechatBindSheet` 的关闭闪烁问题有明确处理。
- `ALLOWLIST` 未新增；没有新增 `wx.*`/DOM API；没有新增散落 hex。
- `docs/mini-design-system.md` 已同步新 props。
- 汇报里「未验证」的项，由审查者决定是否要求用户在开发者工具里人工验证后再合入。

## 开发者工具人工测试清单（供测试人员使用）

页面路径来自 `src/pages.json`。每个弹层都要测：打开、遮罩关闭、关闭按钮、关闭后立即再打开、键盘弹出（有输入框时）、底层页面是否随手势滚动。

### 回归（任务 A、B 后必测：这些页面原本就在用 `FiBottomSheet`，表现必须与改动前一致）

| 页面 | 弹层 | 怎么打开 |
|---|---|---|
| `pages/seat-swap/index` | 发布/修改换座弹层（`SeatSwapPublishSheet`） | 点页面底部 dock 的发布入口；或在候选列表点某个候选触发预填发布 |
| `pages/seat-swap/index` | 管理我的换座弹层（`SeatSwapManageSheet`，任务 A 会移动它的文件位置） | 已有发布的账号，点 dock 上的「管理」 |
| `pages/membership-purchase/index` | 等级与升级规则（`MembershipRulesSheet`） | 点页面里的「规则」链接行 |
| `pages/membership-purchase/index` | 服务协议（`MembershipAgreementSheet`） | 未勾选协议时点支付，会弹出 |

`seat-swap` 已有页面级滚动锁（`pageScrollLocked`），测弹层打开后底层页面不能滚动；`membership-purchase` 同理。

### 任务 C 迁移对象（`pages/user/index`，我的页）

| 弹层 | 怎么触发 | 测试前提 |
|---|---|---|
| `UserNotificationEmailSheet`（回流提醒邮箱） | 已登录，在账号信息区点邮箱行的「填写/编辑」 | **当前版本不能处于审核态**：`systemConfigUnderReview` 为真时该弹层不渲染。先确认测试用的审核登记状态，否则看不到入口 |
| `UserWechatBindSheet`（微信绑定） | 小程序里微信一键登录，后端返回「需要绑定」时自动弹出（`loginFlow.ts` 设置 `miniWechatBindState`） | 需要一个**尚未绑定账号的微信**。没有现成账号时先和实现者确认如何构造，不要拿生产用户数据做绑定测试 |
| `UserPasswordLoginSheet`（密码登录） | **仅 H5**（`#ifdef H5`），小程序端不渲染。H5 下点密码登录入口 | 用 `bun run dev:h5` 在浏览器测，开发者工具里看不到 |

任务 C 的重点观察：

1. **保存中**（邮箱弹层点「保存邮箱」后、请求未返回时）：点遮罩、点 `✕`、点「取消」都不应关闭。
2. **保存失败后**：弹层仍打开，输入内容不丢。
3. **微信绑定弹层关闭瞬间**：内容不应先变空再消失（闪烁）。
4. **键盘**：邮箱、密码、昵称等输入框聚焦时，输入框不被键盘遮挡，弹层可滚动到按钮。
5. **层级**：弹层应盖住底部导航栏，不被其他元素压住。
6. **H5 底栏**：H5 下弹层底部不被底栏遮挡（对应 `respectWindowBottom`）。
7. 内容少时弹层贴合内容高度，不是大面积空白；内容多时能在弹层内滚动。

### 无需测试（本计划不动）

`rankings`、`matches`、`ticket-watch`、`home` 的各类弹层，以及 `team-board`。如果测试时发现这些页面表现变了，说明改动越界，请直接反馈。
