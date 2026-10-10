# 足球小程序第一阶段静默登录与剩余竞态修复计划

> **For agentic workers:** 使用 `superpowers:executing-plans` 逐任务实施。用户指定另一位 agent 实现，本对话负责 review；初稿只作交接；后续用户已授权本地 Kimi 实施、Codex review。遵守项目 AGENTS.md：认证、数据提交和关键状态变化补必要行为测试，纯 UI 不机械 TDD。

**Goal:** 微信小程序打开时自动恢复已绑定账号，首次未绑定继续游客态；保证主动退出有效，并修复迟到微信 code 和订阅请求切走再切回的竞态。

**Architecture:** 认证意图、微信 code 获取、token 发布与自动恢复由 `appSession` 统一管理；页面 flow 只处理显式操作的表单、提示和跳转。公开页面与后台会话恢复并行，私有区在已认证会话发布后刷新。看板生命周期 generation 使重置前的订阅请求永久失效。

**Tech Stack:** 现有 uni-app + Vue 3 + TypeScript + Bun，不增加状态管理或 UI 依赖。

**Spec:** 本文件「已确认设计」为本轮设计依据；同时阅读 [原工程对齐设计](../specs/2026-10-09-mini-registration-alignment-design.md) 与 [原实施计划](2026-10-09-mini-registration-alignment.md)。本轮明确覆盖其中“无 token 不自动微信登录”的旧要求，其余约束和前几轮修复继续有效。

**执行状态（2026-10-11）：** 业务已实施至 `183da4b`，代码 review、本地完整验证与文档交付完成；微信真机/开发者工具与真实支付尚未验收。见 [实施报告](../../reviews/2026-10-10-mini-silent-login-phase1-implementation.md) 和 [Codex Review](../../reviews/2026-10-10-mini-silent-login-phase1-review.md)。

## 全局约束

- 规划基线：`feat/mini-registration-alignment`，HEAD `91d11c2`。实施前核对实际提交和脏树，保护其他人的改动，不 reset/clean/stash 其工作。
- 修改范围为 `football_insight_mini` 和相关文档。Rust、scraper、数据库、独立 `football_insight_h5`、报名系统均不修改；现有足球 API 契约保持。
- 只恢复已有微信绑定账号，不自动建号、不自动绑定、不自动填写资料。邀请码/推荐关系、V1/V3、会员、订单、邮箱、订阅和支付规则不变。
- 只在 MP 自动获取微信 code；H5 保留现有密码/白名单测试登录，不调用 `uni.login`。
- AI 保持删除。足球品牌、TabBar、页面布局、生产 API `https://match.oryjk.cn/api/v1` 不变。
- 库存/回流仍从 `sale_start_at + 10 minutes` 起算；缺时间仍报错或跳过。
- request 层只读 token，不增加全局 401 回调或导航；展示组件只通过 props/emits 交互，不访问 API/session/authStorage。
- 所有 `.env`、密钥、证书不提交。不执行真实版本登记、upload/preview、管理员审核 PUT、生产部署或重启。
- 每个任务保留可审查的提交边界，完成后交回本对话复审；实现 agent 自报测试通过不等于 review 通过。

## 已确认设计

### 1. 自动恢复的业务行为

| 场景 | 必须行为 |
| --- | --- |
| MP 已有有效 token | 复用共享 `ensureSessionReady`，恢复同一个用户，不调用微信 code 接口 |
| MP 无 token，已绑定微信 | 后台获取 code、调用现有 `/auth/mini-wechat/login`，发布原账号 token/user；无成功 toast，无导航 |
| MP 旧 token 401 | 失效旧会话后允许当前有效的自动恢复操作尝试微信登录；外部退出/新登录若已替代该操作，立即停止 |
| MP 首次未绑定 | `binding_required` 作为正常游客结果：不建号、不弹绑定表单、不写 token、不跳转；需要登录时由用户显式操作重新获取 code |
| 用户明确点击登录 | 正常显式登录；`binding_required` 仍展示昵称、头像、邀请码/推荐码表单，失败可原地重试 |
| 用户主动退出 | 同步失效所有旧恢复/认证操作、清本地会话，持久化手动退出标记；关闭重开仍不自动登录 |
| 手动退出后明确重新登录 | 只有显式认证操作解除手动退出标记；页面刷新、force 刷新和自动恢复均不得解除 |
| `/auth/me` 断网/超时/403/5xx | 保留已有 token/user，不把错误当 401，不切到另一个微信账号；公开页面继续加载 |
| 无 token 时自动恢复失败 | 游客可浏览，不弹认证错误 toast/绑定 sheet/跳转；记录可诊断错误，显式登录可重试 |
| H5 | 有 token 正常恢复；无 token 游客；不自动微信登录，不自动选择默认测试账号 |

自动恢复只在 App 启动时安排，不在每个页面、每个轮询或每次 401 中反复触发。首次未知用户的自动探测，每次应用启动最多一次；同次启动失败后通过显式登录重试，下一次冷启动可重新尝试。不新增自动重试定时器。运行期间业务请求 401 仍按现有流程失效并展示登录引导，不在 request 层自动重发写操作。

### 2. 会话与认证意图的边界

- `ensureSessionReady({ force? })` 继续只恢复/刷新本地 token 对应的用户，保留 60 秒成功缓存、同一 revision/token 并发合并、会员购买后 force 刷新语义。
- 新增 `bootstrapSession(options: { allowSilentWechat: boolean }): Promise<CurrentUser | null>`，用于启动恢复。`false` 时只走已有 token 恢复；`true` 且未手动退出时允许一次后台微信探测。按同一次启动操作身份合并并发请求，迟到的 finally 不得清掉新的 promise。
- 新增 `loginWithMiniWechat(): Promise<MiniWechatLoginResponse>`，作为显式微信登录入口，认证意图在调用瞬间、获取 code 前建立。code 获取与 code 换 token 必须在同一个操作身份内，不允许旧 code 返回后才建立一个“全新意图”。
- 微信 code 的获取移入 session 管理的认证流程。可以直接使用 `uni.login` 并提供 `createAppSession` 的依赖注入点；不新增通用 port/adapter 框架。测试注入可控的 code promise。
- 现有 `loginWithWechatCode` 若保留，只作内部/兼容低层调用；业务页面不得在 session 意图之外先等 code，再调用它。扫描并迁移实际调用方。
- 自动恢复、显式密码/微信/绑定/测试登录、退出都有可核对的操作身份。显式操作替代后台恢复；从等待 code 开始，旧成功、失败、状态清理和后续 toast/导航均受防护。
- 仍须保留两种相反顺序：有效新登录在途时旧 token 401 不取消新登录；已被替代的旧登录在途时，当前账号 401 必须正常递增 revision、清理私有状态。
- 手动退出标记只由 session 修改。建议 key 为 `fi_manual_logout`，值 `true`；新增 `hasManualLogout/setManualLogout/clearManualLogout` 存储函数。不会清理不相关的本地数据。
- `binding_required` 的 bind_token 不进入持久化或自动恢复缓存。显式绑定使用其原有生命周期。

### 3. 启动与页面衔接

`App.vue` 用条件编译计算 `allowSilentWechat`，仅 MP 允许。后台启动恢复，不 await 它来阻塞公开赛果、榜单或首页；公共配置/审核加载仍独立处理。

保持审核期间的现有显隐和会员/支付限制。若启动时已经确认审核中，不发起新的自动微信探测；已有 token 的处理保持原策略。审核查询失败不得被描述为已过审，不借本任务改变审核失败策略。

不能只在 App 加一次登录调用：用户可能已经打开“我的”、洞察或余票页，后台恢复随后才完成。当前页面必须在已认证用户发布后更新私有区，无需再次切 Tab；在途游客加载不能覆盖它。退出/切换时继续立即清私有状态并停止旧轮询；隐藏页面不启动轮询，也不因恢复成功自动开启用户未开启的监控。

页面的初始化与恢复完成通知应合并同一会话的重复请求。用真实 Vue watcher 与页面 handler/composable 联动测试，不以复制 watcher 分支代替实际时序验证。

### 4. 看板 generation 与请求身份

- `useTicketWatchState` 新增 `currentBoardGeneration: Ref<number>`，初值 0，单调递增，重置时不能归零。
- `resetCurrentBoard()` 在清数据/复位 loading 前递增 generation；切换球队、会话变化，以及实际替换当前比赛上下文的入口都使旧 generation 失效。
- 同一比赛的普通库存轮询不递增 generation；只在上下文被替换/重置时递增，避免合法订阅请求不断被轮询取消。
- 订阅状态、套餐、下单及支付后处理捕获 generation、session revision、球队、比赛；每个异步阶段及成功/错误/finally 都验证。
- 同一 generation 内允许同类操作重叠时使用请求序号/owner；只有当前 owner 能清其 loading。不能改成 finally 无条件清理。
- 必须覆盖 A → B → A。仅比较球队/比赛 ID 不足以判断请求仍有效。

## Review Focus

1. 自动探测未知账号：不能意外建号、弹资料表单或反复调用微信；任务 2、3 验证。
2. code 等待期间第二次登录或退出、冷启动手动退出标记、旧 token 401 与新认证交错：旧操作不能恢复会话；任务 1、2 验证。
3. 首页公开数据先成功、会话后成功、首次直达私有页及隐藏页面：私有区正确更新，公开区不阻塞、轮询不自启动；任务 3 验证。
4. H5、审核中、断网/403/5xx、显式 binding_required 失败重试：不能改变平台与权限政策；任务 1、2、3 验证。
5. 看板 A → B → A 与旧请求先/后返回：旧结果和 finally 不覆盖新请求；任务 4 验证。

## Task 1：从 code 获取开始统一显式认证意图

**Files（相对 football_insight_mini）：**
- Modify: `src/stores/appSession.ts`、`src/pages/user/loginFlow.ts`、`src/pages/user/index.vue`。
- Test: `src/stores/appSession.test.ts`、`src/pages/user/loginFlow.test.ts`、`src/pages/user/accountEffects.test.ts`；更新实际受接口变化影响的 session fixture。

**Interfaces:** session 提供 `loginWithMiniWechat(): Promise<MiniWechatLoginResponse>`；`createAppSession` 增加可选 code provider 注入。`loginFlow.loginWithMiniWechat()` 保持页面接口，改调用 session 完整微信认证，不再自己获取 code。

- [x] 新增失败测试：第一次 code 慢，第二次 code 快且登录成功，用户退出，第一次 code 返回；认证 API 总调用数保持 1，最终 guest，无旧成功提示/导航。
- [x] 新增失败测试：code 在新密码/绑定/测试登录后返回或失败；不发旧认证请求，不提示旧错误，不清新表单/loading。
- [x] 运行 `bun test src/stores/appSession.test.ts src/pages/user/loginFlow.test.ts`，确认新竞态用例在现状失败；不得通过删除旧测试变绿。
- [x] 实现 session 持有整段认证操作；迁移 flow。保留前两轮的 action/loadUser 后防护与当前提交的失败重试上下文。
- [x] 用实际 Vue watch 连接 flow/effects 验证：正常密码成功、微信 authenticated、binding_required、错误后保留 bind_token、真正退出清表单。认证意图要在 watch 可见的时序内正确登记，不能依赖手工执行复制的 watcher。
- [x] 定向测试及 `bun run type-check` 通过，独立提交 `fix(mini): guard wechat auth from code acquisition`。

## Task 2：手动退出持久化与第一阶段自动恢复

**Files:**
- Modify: `src/utils/authStorage.ts`、`src/stores/appSession.ts`。
- Create/Test: `src/utils/authStorage.test.ts`；扩展 `src/stores/appSession.test.ts`。若现有相同测试入口已存在，复用它。

**Interfaces:** `hasManualLogout(): boolean`、`setManualLogout(): void`、`clearManualLogout(): void`；`bootstrapSession({ allowSilentWechat }): Promise<CurrentUser | null>`；保留 Task 1 的显式认证入口。

- [x] 新增行为测试：有效 token 不获取 code；无 token 的已绑定用户恢复原 user id/token；未知用户返回 null、无 token、无建号/绑定请求。
- [x] 新增行为测试：旧 token 401 后可自动恢复；me 断网/超时/403/5xx 不调用微信 API、不清已有会话。
- [x] 新增行为测试：并发 bootstrap 只获取一次 code、一次登录；未知/失败结果本次启动不反复探测；新 store 冷启动可再次探测。
- [x] 新增行为测试：主动退出时立即写标记；远端 logout 失败仍退出；新 store 读取同一存储后仍阻止自动登录；force ensure 不清标记；明确密码/微信/绑定/测试登录才能解除。
- [x] 新增交错测试：自动恢复等待 code/API 时退出或显式登录，迟到结果/错误/finally 全部失效；自身 me 401 与外部退出导致的 revision 变化不能混淆。
- [x] 保留并运行两种 401/登录交错用例及所有已有会话缓存测试；H5 `allowSilentWechat=false` 在无 token/401 时从不获取 code。
- [x] 实现，定向测试及 type-check 通过，独立提交 `feat(mini): restore bound wechat sessions silently`。

## Task 3：App 接入与恢复完成后的页面刷新

**Files:**
- Modify: `src/App.vue`、`src/pages/user/index.vue`、`src/pages/user/accountEffects.ts`、`src/pages/home/index.vue`、`src/pages/insights/index.vue`、`src/pages/ticket-watch/index.vue`、`src/pages/ticket-watch/useTicketWatchBoards.ts`、`src/pages/ticket-watch/useTicketWatchPolling.ts`。
- Inspect/Modify if needed: `src/pages/membership-purchase/index.vue`、`src/pages/support/index.vue`、`src/pages/team-board/index.vue`、`src/pages/seat-swap/index.vue`。
- Test: 扩展 `src/pages/user/accountEffects.test.ts`、`src/pages/ticket-watch/effects.test.ts`；新增 `src/stores/sessionBootstrap.test.ts`，或在现有会话测试中放等价行为用例。页面需要提取最小可测编排时放其自身目录，不建设新框架。

**Interfaces:** App 只安排 Task 2 的 bootstrap；页面消费 session 的已确认 currentUser/revision，私有加载仍留在页面/composable，不将业务列表存入全局 store。

- [x] 接入 MP/H5 条件编译和独立审核/公共配置预加载；公开首页请求不能 await 微信 code 或 me 请求。
- [x] 增加延迟认证联动测试：公开数据在 code pending 时已展示；自动恢复成功后，“我的”邮箱/管理员入口、首页助力资料、洞察会员数据、余票当前用户/档位/订阅自动刷新，无需切 Tab。
- [x] 增加测试：游客私有加载尚未完成时认证成功，旧分支不覆盖新结果；已隐藏/卸载页面不因后台恢复开启轮询。
- [x] 增加测试：审核中不显示/启动会员支付功能；H5 不自动调用 uni.login；首次未绑定与后台失败不弹 toast/sheet、不消费登录后跳转目标。
- [x] 明确私有操作等待认证或引导显式登录，不自动重放投票、下单、支付等写操作；token 存在不能当作会员授权。
- [x] 定向行为测试、type-check、boundaries 通过，独立提交 `feat(mini): refresh private views after session restoration`。

## Task 4：看板生命周期防止切走再切回的旧请求复活

**Files:**
- Modify: `src/pages/ticket-watch/useTicketWatchState.ts`、`src/pages/ticket-watch/useTicketWatchBoards.ts`、`src/pages/ticket-watch/useTicketWatchSubscription.ts`、`src/pages/ticket-watch/index.vue`。
- Inspect/Modify if needed: `src/pages/ticket-watch/useTicketWatchMatchId.ts`、`src/pages/ticket-watch/useTicketWatchInterests.ts`（共用同一上下文时加入防护，不重写其业务）。
- Test: `src/pages/ticket-watch/effects.test.ts`。

**Interfaces:** state 新增 `currentBoardGeneration: Ref<number>`；reset 与比赛上下文替换同步递增；消费者捕获 generation，不凭 match/team 相等重新承认旧请求。

- [x] 新增失败测试：旧 A 状态请求 pending → B → A → 新 A pending → 旧 A 返回；邮箱和 subscribed 不被写入，新 loading 仍 true。
- [x] 新增失败测试：新 A 先发布结果 → 旧 A 后成功/失败；保留新结果，不弹旧错误，不清新 owner。
- [x] 套餐加载、下单与支付确认加入同类用例；旧订单不能在上下文失效后拉起微信支付，支付后的旧结果不修改新 sheet。
- [x] 运行 `bun test src/pages/ticket-watch/effects.test.ts` 确认新用例现状失败。
- [x] 实现 generation 和必要的请求 owner 防护；保留上轮 reset loading 与 resetMatchIdSessionState 修复。普通同比赛库存轮询不失效合法订阅请求。
- [x] 原有选择乱序、轮询停止、账号切换、sale_start_at 起算用例继续通过，独立提交 `fix(mini): invalidate subscription requests across board resets`。

## Task 5：文档、最终验证与 review 交付

**Files:** `football_insight_mini/README.md`、`football_insight_mini/docs/mini-architecture.md`；新增 `docs/reviews/2026-10-10-mini-silent-login-phase1-implementation.md`。历史设计保留，通过链接说明本轮覆盖旧“不自动登录”条款，不伪造过去决定。

- [x] 文档明确：仅 MP、仅已有绑定账号自动恢复，首次未知继续游客；手动退出跨冷启动生效；H5 保持显式登录；不自动建号、不改邀请码/推荐/会员/支付。
- [x] 从 `football_insight_mini` 执行下列命令，记录真实 exit code 与测试数：

```bash
bun test
bun run type-check
bun run check:boundaries
bun run build:h5
MINI_REVIEW_SKIP=1 bun run build:mp-weixin
```

- [x] 检查组件注册和编译包真实 API 地址通过；版本文件没有因离线构建被改号，git diff 无 env/密钥/证书/生成产物。
- [x] H5 实际页面验收：密码登录成功/失败重试、白名单账号切换、游客公开浏览；不请求微信登录接口。
- [ ] **未执行，仍待端上验收：** 微信开发者工具/真机验收：老用户无 token 冷启动、过期 token、首次未绑定、手动退出重开、明确重新登录、慢 code 后退出、切球队再切回、绑定错误重试、跨 Tab 私有状态与轮询停止。
- [x] 如无法做真实端上验证，列出“未验证”的具体项目；不把构建或单元测试通过当成微信登录/支付/视觉验收通过。
- [x] 交付当前目录/分支、基线和最终 commit、逐任务改动、回归结果、真实端证据与未验证项，交回原 reviewer；不自行宣布最终 review 通过，不合并 main，不上传或部署。

## 提供给实现 agent 的单一提示词

```text
请按 docs/superpowers/plans/2026-10-10-mini-silent-login-phase1.md 实施。
项目 /Users/carlwang/projects/football_insight，规划基线 91d11c2，分支 feat/mini-registration-alignment。
先核对当前代码和工作区，保留已有修复；另一台机器请使用实际仓库路径。

本轮合并两个目标：
1. 第一阶段静默登录：仅微信小程序自动恢复已绑定账号；未知账号继续游客，明确登录后才补资料；手动退出持久生效；H5 保持显式登录；不自动建号、不改变会员/邀请码/推荐/支付规则。
2. 修复第三轮 review 的迟到微信 code 恢复已退出会话，以及看板 A→B→A 旧订阅请求覆盖新结果/loading。

认证身份必须在获取 code 前建立，自动恢复和显式登录共用 session 的完整防护；看板重置必须递增 generation，不只比较球队/比赛 ID。页面需在后台认证成功后刷新私有区，公开首页不等待认证。

按计划逐任务实现并保留清晰提交。以真实 session、Vue watcher 和可控延迟响应验证；保留前几轮全部回归测试，不能只复制 watcher 分支来证明页面行为。
使用 Bun，执行完整测试、类型检查、依赖边界、H5 与 MINI_REVIEW_SKIP=1 的小程序离线构建；真实微信/H5场景能验证就提供证据，不能验证则明确列出。
不要改后端/数据库/抓取器、独立 H5 或报名系统，不恢复 AI，不执行真实版本登记/上传/生产部署，不自行合并 main。
完成后给出分支、最终提交、逐任务修复说明、测试结果和未验证项，交回原 reviewer 复审。
```
