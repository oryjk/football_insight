# 足球小程序工程对齐实施交付记录

日期：2026-10-09。状态：首批 Task 0–6 实施完成，待 review。

- 计划：[../superpowers/plans/2026-10-09-mini-registration-alignment.md](../superpowers/plans/2026-10-09-mini-registration-alignment.md)
- 设计：[../superpowers/specs/2026-10-09-mini-registration-alignment-design.md](../superpowers/specs/2026-10-09-mini-registration-alignment-design.md)
- 分支：`feat/mini-registration-alignment`（本地，未 push）
- 基线：main `b307a17`（拉取时与 origin/main 一致，工作树干净）
- 提交范围：`b307a17..HEAD`，6 个提交，全部改动在 `football_insight_mini/` 内（55 个文件，+3356/−2271）

## 提交清单

| 提交 | 任务 | 说明 |
| --- | --- | --- |
| `d4833c1` | Task 0 | feat(mini): 移除小程序 AI 聊天功能及专用调用链（21 文件，+15/−1982） |
| `fe13279` | Task 1 | feat(mini): 新增 appSession store 收口认证写入与会话竞态 |
| `f65320b` | Task 2 | refactor(mini): 页面与余票看板迁移到共享 appSession 会话（12 文件，+262/−68） |
| `77e7dd6` | Task 3 | feat(mini): 新增 runtimeConfig 与 miniReview 共享 store（11 文件，+729/−42） |
| `f5c3559` | Task 4 | feat(mini): 统一小程序发布入口与 Git/环境/产物门禁（13 文件，+1009/−122） |
| `e7bfeb8` | Task 5 | docs(mini): 新增架构与设计系统文档，FiLoading 迁入设计 token（6 文件，+247/−14） |

## 基线与最终验证（2026-10-09 在 `football_insight_mini/` 实际执行）

基线（改动前）：`bun test` 283 pass / 0 fail（28 个文件）。

最终套件（全部 exit 0）：

| 命令 | 结果 |
| --- | --- |
| `bun test` | 369 pass / 0 fail，3681 expect，30 个文件 |
| `bun run type-check` | vue-tsc --noEmit 通过，无输出 |
| `bun run check:boundaries` | ✅ 依赖边界检查通过 |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | DONE；组件注册检查通过；产物 API 校验 `https://match.oryjk.cn/api/v1` |
| `bun run verify:mp-components` | All mini-program custom components are registered |
| `bun run verify:mp-release` | mode=production；编译包 API 校验通过 |
| `bun run build:h5` | DONE Build complete |

版本文件核对：构建前后 `src/manifest.json` 与 `src/config/generatedMiniProgramVersion.ts` MD5 不变（`3f91a6fd…` / `1eb47e6d…`），versionName 保持 `1.0.64`；离线构建走 skip 短路分支，无远程 allocate/SDK 请求（构建日志 `[sync-manifest-version] kept manifest versionName=1.0.64`，管线无 mini-ci 步骤）。构建后 `git status` 干净。

## 逐任务结果摘要

### Task 0：移除小程序 AI 功能（d4833c1）

- 删除 11 个文件：FiAiChatSheet、useAiChatSheet、api/ai(+test)、types/ai、config/aiChat(+test)、aiEntryIntent、aiChatStorage、wechatCloud、ronaldinho-avatar.png。
- FiBrandNav 移除 AI 按钮/`openOnCurrentPage`/`open-ai`，`syncAiButtonWithMenuCapsule` 改为 `syncBrandWithMenuCapsule`（品牌 64rpx 高度、H5 布局、胶囊垂直居中保留）。
- 五个页面移除聊天挂载/事件/状态/onShow intent；首页 `getPublicSystemConfig`（公告/会员规则）保留。
- `PublicSystemConfig.ai_chat_mode` 移除；AST ALLOWLIST 两项 AI 例外移除（榜赛内嵌 MatchesContent 例外保留）；App.vue 移除 initWechatCloud。
- 验证：bun test 275 pass/0 fail（基线 283 − 8 个 AI 测试）；type-check、boundaries、两端构建通过；源码与两端产物全量搜索 AI 关键词零命中。
- 实际发现：README、check-import-boundaries.test.ts、package.json/bun.lock 均无 AI 相关内容，无需改动（与计划文件清单的预期不同，已核实）。

### Task 1：appSession 收口认证写入（fe13279）

- 新增 `src/stores/appSession.ts`：`createAppSession(source)` 注入点 + `useAppSession()` 单例；只读 refs `currentUser/hasAccessToken/status/errorMessage/revision`；action 按计划契约（ensureSessionReady、loginWithPassword、registerAccount、loginAsTestUser、loginWithWechatCode、bindWechatAccount、logoutSession、invalidateSession）；`SessionSupersededError`/`isSessionSupersededError`。
- `api/auth.ts` 六个方法改原子请求（不写 token、logout finally 不再清理）；`getCurrentUser` 保持仅 401→null。
- user/settings 页登录、绑定、测试登录、退出入口全部改走 session；superseded 静默放弃（不 toast/不跳转）；binding_required 页内表单保持。
- 测试 44 pass（auth.test + appSession.test + loginHelpers.test）：deferred A/B 登录、退出期间登录返回、交错竞态（旧 token me/401 晚到不覆盖 B）、401 清会话、5xx/断网保留会话可重试、logout 竞态、60s 缓存/force/并发合并均有对应用例。

### Task 2：会话消费点迁移（f65320b）

- App.vue 有 token 才非阻塞 ensureSessionReady；首页/洞察/会员购买/换座/助力/战术板/ticket-watch 三 composable 的用户读取与页面清 token 逻辑迁入 session。
- 会员支付成功 `ensureSessionReady({ force: true })`；各私有异步结果捕获 revision 核对发布；ticket-watch 页 watch revision 即时清理看板/兴趣/邮箱/会员镜像并停止旧账号轮询。
- 全量搜索证据：`setAccessToken` 写调用仅存在于 `stores/appSession.ts`；`getAccessToken` 残余读取仅 App.vue 启动门槛、userActivity/request 低层用途、user 页只读判断。
- 测试：ticket-watch 71 pass（67 既有 + 4 新增 revision 用例）；全量 312 pass。
- 已知边界：`useTicketWatchSubscription.ts` 的回流订阅刷新未加 revision 防护（不在计划文件清单；revision watch 已即时清理，下次 onShow 以新会话重载）。

### Task 3：runtimeConfig 与 miniReview 共享（77e7dd6）

- 两个 store 各自并发去重、60s 成功缓存、force 绕缓存合并在途；失败 ready=true/available=false、不永久缓存、下次可重试；两 store 独立失败互不影响。
- runtimeConfig 失败保留最后成功 config，ensure 返回 null；miniReview 失败 status=null（isUnderReview=false，保持既有显隐基线）。
- `setMiniReviewStatus` 递增 epoch，在途旧 GET 成功/失败均不能覆盖 PUT；设置页管理员 PUT 后调用。
- `utils/systemConfig.ts` 委托 miniReview store，`resolveSystemConfigUnderReview(null) === false` 保持；未引入报名系统 fail-closed 政策。
- 测试：store 15 个 + systemConfig 委托 3 个新用例；全量 330 pass。

### Task 4：统一发布入口与门禁（f5c3559）

- 新增 `scripts/mini-release.mjs`（`runMiniRelease(command, args, { projectRoot, env, runStep, checkGitState })`）与 `scripts/mini-release-guards.mjs`（`loadProductionEnvironment`/`verifyUploadGitState`/`normalizeReleaseApiBase`/`parseMiniCiArgs`，纯函数无副作用）。
- package scripts：`build:mp-weixin`/`mp:release`/`mp:preview` → `bun --no-env-file scripts/mini-release.mjs <build|upload|preview>`；新增 `verify:mp-release`、`test:mp-release`；移除 prebuild 版本同步链。
- upload 顺序：Git(fetch) → boundaries → 登记一次 → Git 再校验（版本文件变脏即停）→ production 构建 → 组件注册 → 产物 API 求值 → Git/fetch 再校验 → SDK upload；任一失败即中止。
- 环境隔离：清理继承 VITE_*/UNI_*，固定 NODE_ENV=production，只读显式 `.env.production`；临时空 envDir 经 `FOOTBALL_RELEASE_ENV_DIR` 传给 vite.config.ts（未设置时日常 dev/H5 行为不变），finally 清理。
- 产物校验改 `vm.runInNewContext` 独立求值 `exports.API_BASE_URL`（1s 超时、沙箱无 require/process/网络）；同路径重写为本地值第二次必拒绝；缺 app.json/模块/执行异常均拒绝。
- mini-ci 在 import SDK 前复用 Git/产物校验；参数只接受 robot 1|2 与 desc；直接调底层脚本无法绕过门禁。
- 测试：`mini-release.test.mjs` + mini-ci + verify-mp-api-base 共 49 pass；全量 369 pass。离线构建日志确认 skip 短路、无 allocate 请求、版本文件不变。
- 注意：调研阶段 shell redaction 失效导致 `.env.ci.local` 的 `MINI_REVIEW_API_KEY` 值进入过实现 agent 上下文；该文件被 gitignore、未提交、未写入任何产出。如介意可轮换该 key。

### Task 5：文档基线与 FiLoading 示范（e7bfeb8）

- 新增 `football_insight_mini/docs/mini-architecture.md`（依赖方向/stores/revision/页面拆分/生命周期/验证命令，写 Task 0–4 后的足球现状）与 `docs/mini-design-system.md`（三层 token、布局责任、FiLoading/FiBottomSheet 契约、page-meta 滚动锁、reduced-motion、后续项目）。
- FiLoading 样式迁入 11 个 `--fi-component-loading-*` token（新增 4 primitive + 4 semantic）；原值→token 逐值相等（对照表见该任务交付），编译产物 app.wxss 中 token 实际值已核对；props `{ title, caption }` 不变。
- `fi-tokens.css` 头部过期 Neo 引用已修正；AGENTS.md/README.md 指向新文档。
- 验证：type-check、boundaries、两端构建通过；全量 369 pass。

## 未验证项（需要人工/review 补充）

以下条目实现侧无条件验证（无模拟器/真机/微信开发者工具），**不是**已通过：

1. H5 与微信开发者工具中五个 Tab 的视觉、透明品牌导航、窄屏与胶囊对齐（Task 0 删除 AI 按钮后）。
2. 两端运行时无 AI 请求/云初始化（仅源码与产物静态确认，未做真机网络抓包）。
3. 真实页面交互流程：游客首页、显式登录后跨 Tab、A/B 账号切换慢请求、断网重试提示、会员刷新、审核状态切换显隐、设置页测试账号切换、换座/投票、余票进出轮询停止（行为已由单元测试覆盖，真实端上未跑）。
4. FiLoading token 化前后的两端视觉（已用原值→token 逐值相等 + 产物 token 值核对证明值未变，未做截图比对）。
5. 真实 `mp:release` 上传、真实版本登记、微信 SDK 调用——按计划未执行，仅有离线构建与 fixture 测试。

## 范围声明

- 无后端（Rust）、scraper、数据库、Nginx、生产配置改动；生产后端未部署/未重启。
- 未 push 分支；未做真实版本登记/微信上传/管理员审核 PUT。
- 报名系统仓库 `/Users/carlwang/registration_system` 全程只读参考，未修改。
- API 契约、会员/审核规则、`sale_start_at + 10 minutes` 统计口径、视觉均保持；设计第 8 节项目（H5 一次性 code 桥接、分包等）不在本次范围。

## 复审修复记录（2026-10-09，基于 d822fb8 的 review 发现）

review 发现 4 个问题，逐问题一个 commit 修复，均在 `football_insight_mini/` 内：

| 提交 | 问题 | 根因与修复 |
| --- | --- | --- |
| `872fe6c` | P2：旧 token 401 取消在途新登录 | `ensureSessionReady` 的 401 分支无条件递增 revision，把在途登录意图 B 的 revision 作废。修复：新增 `pendingAuthIntents` 计数（登录/注册/绑定意图期间 >0），401 分支在有认证意图在途时只清旧 token 会话状态，不递增 revision；无意图时 401 清会话语义不变。`src/stores/appSession.ts` |
| `45f49f0` | P1：user 页未完成共享会话迁移 | `loadUser` 直接调 `getCurrentUser`/邮箱 API 写本地状态，无 revision 防护。修复：抽取 `src/pages/user/accountEffects.ts`（注入 session + API 依赖），所有私有请求成功/错误/finally 校验捕获 revision；`currentUser`/`hasAccessToken` 改 session 只读别名；`watch(session.revision)` 清理邮箱/弹层/绑定态等私有状态。 |
| `21552d8` | P1：余票订阅迟到响应恢复旧账号数据 | `useTicketWatchSubscription` 全无 revision 防护；interests toggle、matchId 解锁/支付、历史看板缓存有漏。修复：订阅状态/套餐/下单/支付全流程捕获 revision + 比赛快照校验；`watch(session.revision)` 增补订阅 sheet/loading/matchId 弹层复位；历史 sections 缓存写入前校验。 |
| `cae09bf` | P2：产物校验沙箱暴露宿主能力 | `runInNewContext` 注入宿主创建的 `exports` 对象，可经 `exports.constructor.constructor` 拿到宿主 `Function`→`process`。修复：不传任何宿主对象，`exports` 由被求值代码在沙箱 realm 内自建；取回导出值只接受字符串原始值；跨 realm 异常只读字符串 message。`scripts/verify-mp-api-base.mjs` |

新增回归测试 13 个（deferred/逃逸 fixture，均验证修复前失败、修复后通过）：

- `appSession.test.ts`：`登录 B 在途期间旧 token 的 401 先到，只清旧会话不取消 B`（修复前 B 被抛 SessionSupersededError）。
- `pages/user/accountEffects.test.ts`（6 个）：A/B 切换迟到响应不写邮箱/不结束 loading/不 toast/不关弹层；正常路径发布；`resetSessionScopedState` 清理。
- `pages/ticket-watch/effects.test.ts`（3 个）：订阅状态刷新/套餐/支付完成后会话被替代，不恢复邮箱/subscribed、不关弹层不 toast（已用 stash 暂存修复验证旧实现下 3 个全部失败）。
- `verify-mp-api-base.test.ts`（3 个）：constructor 链逃逸改宿主 `process.env`（修复前实测 `Received: "yes"`）；沙箱无 process/Bun/require/fetch/XMLHttpRequest；非字符串导出拒绝。

修复后验证（2026-10-09 实际执行，全部 exit 0）：

| 命令 | 结果 |
| --- | --- |
| `bun test` | 382 pass / 0 fail，3736 expect，31 个文件（基线 369 + 13） |
| `bun run type-check` | 通过 |
| `bun run check:boundaries` | ✅ 通过 |
| `bun run build:h5` | DONE Build complete |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | DONE；组件注册检查通过；产物 API 校验 `https://match.oryjk.cn/api/v1`（修复后的沙箱校验对真实产物通过） |

版本文件 MD5 仍不变（1.0.64），无远程登记/上传；未验证项与上文一致（真机/视觉项本轮未新增验证）。与 review 描述的偏差：user 页 revision watch 未清 `h5TestUsers`（后端公共白名单，非会话私有，清了会导致 H5 登录入口闪烁）；interests 的 finally 清 loading key 不做 revision guard（纯 UI 状态，必须无条件清避免锁死）。
