# 足球小程序与报名系统工程对齐实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 `superpowers:executing-plans` 逐任务实施。用户指定另一位 agent 实现、本对话负责 review；不要自动在本对话启动实现。项目 AGENTS.md 优先：共享业务逻辑和发布流程补行为测试，纯 UI 不机械 TDD。

**Goal:** 移除足球小程序 AI 功能，对齐报名系统的会话、共享配置、发布与页面职责，保持剩余产品、足球 API 和视觉。

**Architecture:** Vue 模块级 ref/computed store 管理真正跨页状态；页面/composable 编排 API 和生命周期，展示组件通过 props/emits 交互。统一 Bun 发布入口保留足球现有登记库、AST 边界及组件检查。

**Tech Stack:** 现有 uni-app + Vue 3 + TypeScript + Vite + Bun；不添加状态/UI 框架。

**Spec:** [2026-10-09-mini-registration-alignment-design.md](../specs/2026-10-09-mini-registration-alignment-design.md)。执行前同时阅读。

## 全局约束

- 基线足球 HEAD `3a27e5837ae17ba9b59c03790497d942868f1bed`；实施前核对漂移和脏树，保护他人改动。报名系统参考 HEAD `18713e2`，不修改该仓库。
- 首批任务 0–6 全部完成后才称“首批工程对齐完成”；Task 0 的 AI 移除是用户明确要求，不可省略。设计第 8 节项目单独排期，不纳入此次实现。
- 不升级框架、不引入 Pinia/Wot/DDD/port/adapter，不改 Rust/Python/数据库/路由/TabBar/生产配置。
- 生产 API 精确为 `https://match.oryjk.cn/api/v1`，project code 为 `football_insight_mini`，沿用足球 DTO 和 `exports.API_BASE_URL`。
- 保留邀请注册、账号密码、微信 binding_required、H5 白名单测试登录和登录后跳转。不从报名系统迁入自动微信登录或球队身份。
- 共享数据成功缓存 60 秒；失败允许下一次页面 onShow 或显式重试；不新增常驻定时器。
- 展示组件无 API/session/authStorage 依赖；原 request 仍只读 token，不导航、不全局注册 401 回调。
- MP 运行时组件直接 import `.vue`，两端构建都需验证。纯 UI 验证采用模拟器/截图，不用静态源码断言冒充业务测试。
- 保留 `sale_start_at + 10 minutes` 和缺起售时间报错/跳过；贡献仍在 scraper 预计算。
- 所有 `.env`、密钥、证书不提交。不运行真实 upload/preview/版本登记/管理员审核 PUT，不部署或重启生产。

## Review Focus

1. A 登录/刷新晚于 B 登录或退出返回：旧结果、401 和 finally 都不能改 B；任务 1、2 的 deferred promise 测试覆盖。
2. `/auth/me` 断网/403/5xx 与 401/null：前者保留当前会话、可重试，后者仅失效当前 revision；任务 1、2 覆盖。
3. 公共配置/审核查询失败与管理员切换竞态：失败不永久缓存，旧 GET 不覆盖新 PUT；任务 3 覆盖。
4. shell 与本地 env 覆盖、编译模块缓存：生产值固定，真实产物重新求值；任务 4 的环境及同路径重写 fixture 覆盖。
5. 新版本写文件、组件检查失败、fetch 失败、直接调 SDK 脚本：均不得上传旧包；任务 4 的 runner/SDK spy 覆盖。

## 交接与执行方式

实现 agent 顺序执行，每个任务独立提交；未验证的项目在交付说明中列出。Task 0 的 AI 删除独立交回 review；任务 1–3 为会话/config 一组，任务 4 为发布一组，任务 5–6 为规范与最终验收一组。每组记录问题、修复提交和复验结果；一次性实现也须保留这四组清晰提交边界。

参考根目录默认 `/Users/carlwang/registration_system/registration_system_mini`；另一台机器上该路径可能不存在。只需读取参考文件并理解模式，本计划及设计给出了足球的接口与行为，不能因此复制未知参考契约。

开始时记录 `git status --short`、HEAD、现有相关测试和构建基线；不 stash/reset/clean 别人的改动。需要隔离时使用项目允许的 worktree，并在交付中给出实际目录与分支。

以下文件均相对 `football_insight_mini/`，除非另有说明。

## Task 0：移除小程序 AI 功能

**Files:**
- Modify: `src/components/FiBrandNav.vue`、`src/pages/home/index.vue`、`src/pages/rankings/index.vue`、`src/pages/insights/index.vue`、`src/pages/seat-swap/index.vue`、`src/pages/user/index.vue`、`src/App.vue`、`src/types/system.ts`、`scripts/check-import-boundaries.mjs`、`AGENTS.md`、`README.md`。
- Delete（先核对剩余引用）: `src/components/FiAiChatSheet.vue`、`src/composables/useAiChatSheet.ts`、`src/api/ai.ts`、`src/api/ai.test.ts`、`src/types/ai.ts`、`src/config/aiChat.ts`、`src/config/aiChat.test.ts`、`src/utils/aiEntryIntent.ts`、`src/utils/aiChatStorage.ts`、`src/utils/wechatCloud.ts`、`src/static/ai/ronaldinho-avatar.png`。
- Test: 现有 `scripts/check-import-boundaries.test.ts`；两端页面及网络验证。仅清理确定 AI 专用且无引用的 package 依赖/锁文件项，不升级其他依赖。

**Interfaces:**
- FiBrandNav 保留 `transparent?: boolean` 和品牌展示；移除 `openOnCurrentPage` / `open-ai`。调用方不再传旧 props/event，不新增空实现占位。
- 小程序 `PublicSystemConfig` 不再消费 ai_chat_mode，服务端额外字段正常忽略；公告/微信登录/会员规则字段保留。
- 移除 AST allowlist 的 FiAiChatSheet 与 api/ai 项，保留榜赛嵌入 MatchesContent 的精确例外。

- [ ] 全量搜索 `FiAiChatSheet|useAiChatSheet|aiEntryIntent|aiChatStorage|wechatCloud|config/aiChat|api/ai|types/ai|open-ai|open-on-current-page|ai_chat_mode|ronaldinho-avatar`，记录实际引用，勿用模糊 `ai` 搜索决定删除（会误命中 email/failed）。
- [ ] 删除导航 AI 按钮、点击跳转/intent、按钮专属 CSS/定位。品牌胶囊对齐继续保留，可将 syncAiButtonWithMenuCapsule 中品牌定位改成职责明确的品牌定位函数；不能连品牌高度和 H5 布局一起删。
- [ ] 从五个页面删除聊天挂载、打开事件、状态、onShow intent 消费及专用用户/config 请求；保留首页 getPublicSystemConfig 的公告/会员等实际用途。
- [ ] 删除专属模块、头像与测试，移除 App.vue 的 initWechatCloud；核对云工具没有新共享用途后删除。保留 uni.login、微信支付/订阅、useAnimatedInteger 和洞察贡献展示。
- [ ] 清理类型、AST 例外及当前文档。删除旧源码断言中已失效的 AI 假设，保留非 AI 行为测试；不删后端 AI 路由，不改独立 H5/Android，不清用户历史存储。
- [ ] `bun test`、`bun run type-check`、`bun run check:boundaries` 通过；用现有命令做 `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` 与 `bun run build:h5`，不真实上传/登记。
- [ ] 在 H5/微信开发者工具查看五个 Tab、透明品牌导航、窄屏和胶囊对齐；操作页面不出现 AI 请求/云初始化，普通登录、公告、会员、余票、换座仍正常。搜索源码/检查构建产物确认无专属聊天链。截图和网络证据交回 review。
- [ ] 独立提交 AI 删除，再继续 Task 1；后续任务不创建/维护已经删除的 AI store/composable。

## Task 1：收口认证写入和会话竞态

**Files:**
- Create: `src/stores/appSession.ts`、`src/stores/appSession.test.ts`。
- Modify: `src/api/auth.ts`、`src/api/auth.test.ts`、`src/pages/user/index.vue`、`src/pages/user/settings/index.vue`。
- Reference: 报名系统 `src/stores/appSession.ts`；足球 `src/types/auth.ts`、`src/utils/authStorage.ts`、`src/utils/apiError.ts`。

**Interfaces:**
- API 现有名字/参数/DTO 保留，移除 token 写入和 logout finally 清理：`login/register/loginAsH5TestUser/loginWithMiniWechat/bindMiniWechatAccount/logout` 成为原子请求；`getCurrentUser(): Promise<CurrentUser | null>` 保持仅 401→null。
- `useAppSession(): AppSession` 返回单例；`createAppSession(source: typeof authApi = authApi): AppSession` 提供本模块行为测试注入点，不构建通用适配框架。
- `AppSession` 包含只读 refs：`currentUser`、`hasAccessToken`、`status`、`errorMessage`、`revision`，以及下述 action。
- `ensureSessionReady(options?: { force?: boolean }): Promise<CurrentUser | null>`。
- `loginWithPassword(payload: LoginPayload): Promise<AuthResponse>`、`registerAccount(payload: RegisterPayload): Promise<AuthResponse>`、`loginAsTestUser(userId: string): Promise<AuthResponse>`。
- `loginWithWechatCode(code: string): Promise<MiniWechatLoginResponse>`、`bindWechatAccount(payload: MiniWechatBindPayload): Promise<AuthResponse>`、`logoutSession(): Promise<void>`。
- `invalidateSession(expectedRevision: number): void` 只失效匹配 revision；`SessionSupersededError` 和 `isSessionSupersededError(error: unknown): boolean` 供消费方识别旧操作。

- [ ] 增加行为测试：无 token 不请求 me；同 revision 并发 ensure 只有一次请求；成功后 60 秒内复用、超过后刷新；force 绕过成功缓存仍合并在途请求。
- [ ] 增加 deferred 测试：登录 A pending，开始 B，B 返回，再返回 A，最后 token/user 属于 B，A 抛 SessionSupersededError；退出期间登录返回不能恢复会话。
- [ ] 增加交错测试：已有 A token，开始登录 B，B 等待期间用 A token 发出 me 刷新；B 成功后 A 的 me/401 晚到，不能覆盖或清掉 B。成功发布新 token 时增加 revision，刷新也验证 token 快照。
- [ ] 增加错误测试：401/null 清当前会话；超时/断网/403/5xx 保留当前 token/user，error 可观察，下次调用可重试；旧账号 401/finally 不改新账号状态。
- [ ] 实现 store 和原子 API。所有发布和 finally 同时核对 revision/请求身份，避免旧 promise 清掉新 promise；返回只读状态。
- [ ] 替换“我的”和设置中的登录/绑定/测试登录/退出入口。微信 binding_required 保持页内表单状态；被替代的操作不得 toast 成功或登录后跳转。
- [ ] logout 在清本地前发起旧 token 后端请求，立即本地退出；远端失败不恢复。测试 logout A pending → B 登录 → A 完成，B 保持登录。
- [ ] 同步修改 auth API 测试，断言原子请求不会写存储，而 session 会写；`resetPassword` 和邮箱 API 契约保持。
- [ ] 运行 `bun test src/api/auth.test.ts src/stores/appSession.test.ts src/pages/user/loginHelpers.test.ts`、`bun run type-check`、`bun run check:boundaries`。提交原子 API + store + 登录消费者为一个可运行变更。

## Task 2：共享读取、账号数据清理与刷新

**Files:**
- Modify: `src/App.vue`、`src/pages/home/index.vue`、`src/pages/insights/index.vue`、`src/pages/membership-purchase/index.vue`、`src/pages/seat-swap/index.vue`、`src/pages/support/index.vue`、`src/pages/team-board/index.vue`、`src/pages/ticket-watch/useTicketWatchBoards.ts`、`src/pages/ticket-watch/useTicketWatchState.ts`、`src/pages/ticket-watch/useTicketWatchMatchId.ts`。
- Test: `src/pages/ticket-watch/effects.test.ts`（现有）；必要时在对应页面目录抽纯状态 helper 并补其 `.test.ts`，不为了测试重写整页。
- Modify/inspect: `src/pages/ticket-watch/index.vue`（生命周期门面）和 `src/pages/ticket-watch/useTicketWatchPolling.ts`（轮询停止与账号切换）；保留已有选择请求序号及停止逻辑。

**Interfaces:**
- 消费 Task 1 的 session refs/actions；组件仍接页面 props。
- 无有效用户时沿用各业务登录提示；网络失败显示重试信息而非清 token。首页 AI 专用 currentUser 已在 Task 0 删除，不重新添加该用途。
- 会员购买完成使用 `ensureSessionReady({ force: true })`；私有异步结果捕获 `revision`，不同则丢弃。

- [ ] `App.vue` 以非阻塞方式初始化已有 token，会话失败不阻断公共首页；无 token 的 H5/MP 不自动登录。
- [ ] 替换上述页面/composable 的用户读取与页面清 token 逻辑。保留 userActivity/request 等低层读取 token 的用途；通过全量搜索清单确认没有业务页绕过 store 写 token。
- [ ] 引用用户/session state；删除独立缓存的旧 currentUser，不将 loading、筛选、请求结果全放进 store。
- [ ] 对账号私有结果建立 revision 清理：用户信息、邮箱/订阅/兴趣区块、会员状态立即失效；旧请求不能发布数据或重启轮询。公共比赛/榜单保持可用。
- [ ] 用 deferred 测试覆盖 ticket-watch 用户 A 的兴趣结果晚到不污染 B、断网不误清会话、退出后旧请求不重启轮询；保留现有历史选择与计时器测试。
- [ ] 验证会员购买后新等级共享给洞察/余票页面；后端付费/授权仍为权威，不凭 token 开放会员内容。
- [ ] 运行 Task 1 测试及 `bun test src/pages/ticket-watch`、type-check、boundaries；提交共享消费点迁移。

## Task 3：公开配置和审核状态共享

**Files:**
- Create: `src/stores/runtimeConfig.ts`、`src/stores/runtimeConfig.test.ts`、`src/stores/miniReview.ts`、`src/stores/miniReview.test.ts`。
- Modify: `src/utils/systemConfig.ts`、`src/utils/systemConfig.test.ts`、`src/App.vue`、`src/pages/home/index.vue`、`src/pages/user/index.vue`、`src/pages/user/settings/index.vue`、`src/pages/membership-purchase/index.vue`；用 `rg -n 'loadSystemConfig|getPublicSystemConfig|getMiniReviewStatus' src` 找齐余票/洞察等消费者。
- Reference: 现有 `src/api/system.ts`、`src/api/miniReview.ts`、`src/types/system.ts`；报名系统 `src/stores/miniReview.ts` 只参考加载结构，不复制其 fail-closed 显隐政策。

**Interfaces:**
- `useRuntimeConfig()`：只读 `config: PublicSystemConfig | null`、`loading/ready/available/errorMessage`，`ensureRuntimeConfig(options?: { force?: boolean }): Promise<PublicSystemConfig | null>`。
- `useMiniReviewStatus()`：只读 `status: MiniReviewStatus | null`、`isUnderReview`、`loading/ready/available/errorMessage`，`ensureMiniReviewReady(options?: { force?: boolean }): Promise<MiniReviewStatus | null>`。
- `setMiniReviewStatus(status: MiniReviewStatus): void`：发布管理员 PUT 成功结果并使先前 GET 失效。两个 store 内部各自请求去重、60 秒成功缓存；失败返回 null/记录错误，不永久缓存。
- `utils/systemConfig.ts` 的既有加载函数委托 miniReview store；纯函数 `resolveSystemConfigUnderReview(null) === false` 保持。刷新失败兼容调用返回 null，不能用旧成功值冒充本次确认成功。

- [ ] 测试并发去重、60 秒缓存、失败后重试、两个配置独立失败；公共配置刷新失败保留最后成功 config，但 available/error 反映本次失败。
- [ ] 测试审核查询失败返回 null/isUnderReview false 的兼容规则，ready 与 available 可区分“已尝试”和“已获取”。页面显隐按基线保持。
- [ ] 测试审核 GET pending → 管理员 PUT 得到新状态 → 旧 GET 返回，最终状态保持 PUT 结果；所有页面观察同一值。
- [ ] `App.vue` 非阻塞预加载，页面 onShow 使用共享入口；管理员切换调用 setMiniReviewStatus。没有页面组件直接调用 store/API。
- [ ] 运行配置 store 测试、现有 systemConfig/API 测试、type-check、boundaries。提交后交回第一组 review。

## Task 4：统一发布入口与门禁

**Files:**
- Create: `scripts/mini-release.mjs`、`scripts/mini-release.test.mjs`、`scripts/mini-release-guards.mjs`（纯校验共享模块）。
- Modify: `package.json`、`vite.config.ts`、`scripts/mini-ci.mjs`、`scripts/mini-ci.test.ts`、`scripts/sync-manifest-version.mjs`、`scripts/verify-mp-api-base.mjs`、`scripts/verify-mp-api-base.test.ts`、`README.md`、`AGENTS.md`。
- Reference: 报名系统 `scripts/mini-release.mjs` 和测试；足球编译模块导出不同，不直接复制 artifact evaluator。

**Interfaces:**
- `runMiniRelease(command: 'build'|'upload'|'preview'|'verify', args: string[], options)`，注入 `projectRoot/env/runStep/checkGitState`，便于测试不启动真实 SDK/登记库。
- `loadProductionEnvironment(projectRoot, inherited)`、`verifyUploadGitState(projectRoot, { fetch, runGit })` 放 guards，供入口和底层 CI 复用；SDK 加载使用动态 import，纯函数 import 没有上传副作用。
- `verifyMiniProgramApiBase(projectRoot)` 既有签名保持；每次独立求值 `exports.API_BASE_URL`，限定执行时间，不用 require 模块缓存。
- package scripts：`build:mp-weixin`→`bun --no-env-file scripts/mini-release.mjs build`；`mp:release`→同脚本 upload；`mp:preview`→preview；新增 `verify:mp-release`→verify、`test:mp-release`→`bun test scripts/mini-release.test.mjs`；`check:boundaries` 保留。移除 prebuild 版本同步，改为入口显式执行一次。

- [ ] 写 fixture/runner 行为测试，替换现有“源码包含字符串”及旧 script 精确串断言：验证调用顺序和上传 spy 未被调用，测试不 import 立即执行上传的模块。
- [ ] 测试 main/clean/HEAD==origin/main、detached/feature/dirty/unpushed/fetch failure 拒绝正式上传；build 不需 main 门禁，preview/verify 不调用版本登记。
- [ ] 测试 `.env.production` 缺失/非法生产 API 拒绝；shell 中本地 API、NODE_ENV=development、UNI_*、VITE_* 被重建；`.env.local/.env.production.local` 的污染变量不能进入产物。
- [ ] 实现临时空 envDir 和 vite 配置接入（仅发布入口）；在 finally 清理。使用显式生产文件的 VITE 变量；保留私钥/登记所需非客户端变量，不打印完整环境。
- [ ] 实现 upload 顺序：Git 校验 → boundaries → 登记一次 → Git 再校验（版本文件变化即停）→ 显式 production 构建 → 组件注册 → API 校验 → Git/fetch 再校验 → SDK upload。build 在离线 skip 时不得登记。
- [ ] 测试版本变更产生脏树后没有构建/上传；外层 MINI_REVIEW_SKIP=1 的 upload 仍登记；任何 runner 失败均中止，不继续上传旧包。
- [ ] 底层 mini-ci 在 import SDK 前校验：upload 复用 Git 和产物校验，preview 校验产物；参数只接受合法 robot/desc，默认 robot 1，测试上传参数保留 robot 2。
- [ ] 测试同一目录第一次校验生产值后重写为本地值，第二次必须拒绝；缺 app.json/API 模块、错误导出、执行异常/超时拒绝。仅含生产 URL 字符串但导出错误也拒绝。
- [ ] 调整版本脚本为显式 CLI + 可测试函数，导入无副作用；保留已有登记库语义、版本回写和离线指定版本处理。测试使用 stub，不写真实版本源文件。
- [ ] 运行 `bun test scripts/mini-release.test.mjs scripts/mini-ci.test.ts scripts/verify-mp-api-base.test.ts`、边界脚本测试；执行离线 MP 构建（最终命令见 Task 6），核对真实产物。更新 README/AGENTS 后提交并交回第二组 review。

## Task 5：文档基线与一个基础组件示范

**Files:**
- Create: `docs/mini-architecture.md`、`docs/mini-design-system.md`（均位于 mini 子项目）。
- Modify: `AGENTS.md`、`README.md`、`src/styles/fi-tokens.css`、`src/components/FiLoading.vue`。
- Reference: 报名系统当前 `docs/mini-architecture.md`、`docs/mini-design-system.md`；足球现有 Fi 组件。

**Interfaces:**
- `FiLoading` 保留必填 props `{ title: string; caption: string }`，不增加 API/store 调用。
- 结构文档定义页面→store/API、props/emits、helpers、生命周期和私有数据 revision；视觉文档定义足球 primitive/semantic/component token 与布局责任。
- 文档记录 FiBottomSheet 既有 visible/title/eyebrow/height/closeOnMask/compactFooter、update:visible/close 契约；不改其实现。

- [ ] 写清足球自身现行视觉及增量对齐原则，去掉 token 注释中的过期 Neo 引用；不要复制报名系统五主题或把足球改为 D 风格。
- [ ] 将 FiLoading 现有颜色、字号、间距、圆角、阴影迁入足球语义/component token。需要新 token 时提取原值，保持视觉。
- [ ] 补 `.vue` 直接导入、父 wrapper 布局、page-meta 滚动锁及 reduced-motion 验证约定；记录 Wot/H5/分包为后续项目。
- [ ] H5 和 MP 查看既有使用 FiLoading 的支持页加载态、长 title/caption 和窄屏，确认组件宿主宽度/换行不变；无需新增镜像实现的快照测试。
- [ ] type-check、boundaries、两端构建通过后提交规范与示范，供最终 review。

## Task 6：最终验证与交付记录

**Files:**
- Create: 仓库根 `docs/reviews/2026-10-09-mini-registration-alignment-implementation.md`，或实施日同主题文件（仅记录实际证据，不预填“通过”）。

- [ ] 实现结束重新记录 HEAD/status。逐任务提交只包含本次业务/文档改动；不夹带 .env、上传私钥、dist、node_modules、其他 agent 工作。
- [ ] 在 mini 根执行下列命令，记录退出码、测试数量、构建产物及具体失败；失败必须修复，或明确交付为未完成：

```bash
bun test
bun run type-check
bun run check:boundaries
MINI_REVIEW_SKIP=1 bun run build:mp-weixin
bun run verify:mp-components
bun run verify:mp-release
bun run build:h5
```

- [ ] 离线构建前保存版本文件差异，确认没有远程 allocate/SDK 请求，构建后检查 manifest/generated version；若已有版本文件发生变化，解释来源并正确处理，不能顺手提交无关版本变更。
- [ ] 两端场景：所有 Tab 无 AI 入口/专用请求；品牌导航布局；游客首页；显式登录后跨 Tab；A/B 切换及退出的慢请求；断网重试；会员刷新；审核状态与配置失败；设置页测试账号切换；换座/投票；余票进入/离开时轮询停止；基础组件布局。
- [ ] 标明场景发生在 H5、微信开发者工具或真机及其版本；没有条件验证的条目标记未验证。不得把测试 fixture 的成功当成真实微信上传或生产验证。
- [ ] 交付实际分支/提交范围、逐任务结果、测试/截图位置、未验证项、API/版本文件是否变更、是否存在后端改动。首批应没有后端改动，生产后端未部署/未重启。
- [ ] 将提交范围或 PR 交回本对话。Reviewer 逐项核对本设计/计划、检查 diff 和竞态测试，针对问题给出文件行号与复现条件；实现 agent 修复后再 review。P0/P1、数据串账号、误清登录、上传门禁绕过属于阻断项；其余发现也须修复或明确达成取舍后关闭。

## 可直接交给实现 agent 的指令

> 请读取根/mini AGENTS.md，以及本计划和关联设计。先按 Task 0 移除足球小程序 AI 入口、聊天组件及专用调用链，再按 Task 1–6 完成工程对齐，保持剩余足球 API、会员/审核/统计口径和视觉。参考报名系统只读，不改其工作区；不引入 Pinia/Wot，不动后端/数据库/生产，不做真实版本登记或微信上传。任务分组提交并提供实际验证结果；注意 AI 删除后保留公告/数字动画/微信登录，以及 API 层 token 副作用、账号 revision、网络失败与 401 区分、配置失败重试及发布环境隔离。H5 一次性 code 桥接与分包按设计第 8 节另行规划。本对话负责最终 review，完成后返回实际分支/提交范围及证据。
