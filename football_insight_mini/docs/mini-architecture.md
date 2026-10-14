# football_insight_mini 结构规范（mini-architecture）

本文档定义足球小程序（uni-app + Vue 3 + TypeScript）的**现行**结构约定：目录职责、依赖方向、会话与共享状态、页面拆分模式。修改代码前先读仓库根目录与本目录的 `AGENTS.md`；视觉规范见 [`mini-design-system.md`](./mini-design-system.md)。

视觉身份与产品契约由本项目决定；与报名系统（registration_system_mini）的对齐仅限工程方法（模块级 store、页面拆分模式、发布入口），不迁入其业务状态与 DTO。

## 目录职责

```text
src/
  api/           # 按业务域封装后端 API 原子调用，不承载页面 UI 状态
  components/    # 跨页面通用组件（Fi* 前缀），只放 API 稳定、真正复用的
  composables/   # 跨页面复用的组合式逻辑（use*）
  config/        # 运行时与环境配置（含构建时生成的 generatedMiniProgramVersion.ts）
  pages/         # 页面；页面专属模块放各页面目录内
  static/        # 图片等静态资源
  stores/        # 仅真实的跨页面共享状态（appSession / runtimeConfig / miniReview）
  styles/        # design token（fi-tokens.css，由 App.vue 全局导入）
  types/         # 后端 DTO、视图模型等共享类型
  utils/         # 请求、存储、纯计算与 ViewModel 转换
scripts/         # 构建/发布/校验脚本（mini-release.mjs 统一入口、边界与产物校验）
```

## 依赖方向与 import 边界

依赖方向由 `scripts/check-import-boundaries.mjs` 机器强制（统一发布入口与 GitHub Actions 都会执行）：

1. 组件层（`src/components`、`src/pages/**/components`）禁止 import `api/**` 与认证存储；`src/components` 还禁止反向依赖页面层；
2. 页面 A 的任何文件禁止 import 页面 B 的任何模块——跨页面共享：逻辑放 `src/utils`，组件进 `src/components`；
3. `api/**` 只能依赖 request/config/types/utils（api 内部互引合法）；
4. 只有 `utils/request.ts` 可以直接调用 `uni.request`。

例外必须显式登记在脚本 `ALLOWLIST`（当前仅：`rankings/index.vue` 跨页内嵌 `matches/MatchesContent.vue`）。回归测试在 `scripts/check-import-boundaries.test.ts`，改规则时必须同步跑。

文件身份：pages.json 注册的页面入口和页面目录直属的 `*.vue`（`index.vue`、`MatchesContent.vue` 这类页面容器）属于页面层，允许依赖 api；只有 `components/` 子目录里的才算展示组件。

## 请求层与会话

- `utils/request.ts` + `utils/apiError.ts`：错误统一为 `ApiRequestError`（`statusCode` + `networkFailed` + 响应体），判断用 `isUnauthorizedError()` / `isNetworkUnavailableError()`，禁止按错误文案猜状态。request 只读 token，不导航、不注册全局 401 回调。
- `api/auth.ts` 是原子请求层：登录/注册/绑定/测试登录/logout 不写存储、不在 finally 清 token；`getCurrentUser()` 仅 401 返回 null，其余错误抛出。
- 所有 token 写入和会话发布收口在 `stores/appSession.ts`（见下节）。

## 共享状态（stores）

Store 只解决真实的跨页状态；页面 loading、弹层开关、筛选和表单状态留在页面/composable，不把请求结果全局化。三个 store 都是模块级 `ref/computed` 单例，用 Vue 响应式，无 Pinia。

- `stores/appSession.ts`：登录用户会话。只读 refs：`currentUser`、`hasAccessToken`、`status`（`guest | loading | authenticated | error`）、`errorMessage`、`revision`。入口 `ensureSessionReady({ force? })`：无 token 不请求 `/auth/me`；同 revision 并发合并在途请求；成功缓存 60 秒；`force` 绕过缓存仍合并在途（会员购买成功后用 `force: true` 刷新）。
- 启动恢复入口 `bootstrapSession({ allowSilentWechat })`（仅 `App.vue` onLaunch 调用，后台执行）：已有 token 走 `ensureSessionReady`；无 token 或旧 token 被 401 失效时，仅当 `allowSilentWechat`（MP）且未手动退出，允许一次后台微信探测（`uni.login` + `/auth/mini-wechat/login`）；`binding_required`/失败静默回游客，不建号、不写 token、不弹表单。同次启动并发合并，微信探测每次启动最多一次；显式认证/退出替代本次启动恢复，迟到的审核或 me 不重启探测。H5 仍恢复已有 token，但不自动探测微信。
- 显式微信登录走 `loginWithMiniWechat()`：认证意图在调用瞬间、获取 code 前建立，code 获取与换 token 在同一操作身份内；被替代后迟到的 code/响应不再发认证请求。微信 code 提供方可通过 `createAppSession(source, { getMiniWechatCode })` 注入（默认 `uni.login`）。
- 手动退出标记 `fi_manual_logout`（`utils/authStorage.ts` 的 `hasManualLogout/setManualLogout/clearManualLogout`）：`logoutSession` 同步写入，跨冷启动禁止静默恢复；只有显式认证操作（密码/注册/微信/绑定/测试登录）解除；`ensureSessionReady` 与 401 失效不动该标记。
- `stores/runtimeConfig.ts`：`PublicSystemConfig` 共享（公告、会员规则等）。`ensureRuntimeConfig({ force? })`，成功缓存 60 秒、并发去重；刷新失败保留最后一次成功 config，但 `available`/`errorMessage` 反映本次失败。
- `stores/miniReview.ts`：`MiniReviewStatus` 共享。审核查询失败返回 null（兼容 `resolveSystemConfigUnderReview(null) === false`），暴露 `ready`（已尝试）与 `available`（已获取）区分状态；管理员 PUT 成功后调 `setMiniReviewStatus` 发布新值并使在途 GET 失效。`status` 保留真实审核登记值，`isUnderReview` 和个人页使用的 `shouldRestrictFeatures` 只在生产构建施加展示限制；`import.meta.env.DEV` 的开发构建始终不限制，包括查询尚未完成时。`utils/systemConfig.ts` 的兼容入口读取该 store 的有效展示限制，避免洞察和余票页重新按原始审核值限制功能。

### revision 与被替代操作

- 新登录意图、成功替换 token、退出、认证失效都会增加 `revision`。
- 异步结果、错误、`finally` 均校验捕获的 revision 与 token 快照：旧请求不能写 token、清新账号、清新 loading 或触发旧跳转；被替代的操作抛 `SessionSupersededError`（用 `isSessionSupersededError` 识别），页面静默放弃该次成功提示/跳转。
- 账号私有数据（用户信息、邮箱/订阅/兴趣区块、会员状态、余票轮询）跟随 revision：revision 变化立即清理旧账号结果、停止旧轮询；晚到的旧请求不得发布数据或重启轮询。公共比赛/榜单不随退出清空。
- 401/null 只失效当前 revision 的会话；断网、超时、403、5xx 保留当前 token/user，记录可观察错误，下次调用可重试。
- logout 先发带旧 token 的后端注销请求，再同步清本地；远端失败不恢复，旧 logout 完成不能清新登录账号。

## 页面拆分模式（优先结构）

```text
src/pages/<domain>/
  index.vue                 # 页面编排：生命周期、加载状态、导航、事件 wiring
  helpers.ts                # 本页面纯函数/格式化/视图模型转换（现状命名，等价于 *State.ts）
  use<Domain>Page.ts        # 可选：较重的页面级状态与动作编排 composable
  <domain>Actions.ts        # 可选：页面级提交动作和 API 编排
  components/               # 页面局部组件（XxxPanel/XxxCard/XxxSheet）
```

职责边界：

- 页面 `index.vue` 只承担编排：生命周期、页面业务状态、异步流程、错误处理、路由、Toast/Confirm。不要继续往大页面里堆模板和请求。
- 页面专属组件放 `src/pages/<domain>/components/`；稳定跨页面复用才进 `src/components/`，命名带 `Fi` 前缀。
- 子组件 `props` 接收数据、`emits` 发出意图；页面局部展示组件不直接调业务 API（组件层连 import api 都被边界检查禁止）。
- 后端数据到展示模型的转换放页面 `helpers.ts`；跨页面复用的放 `src/utils/`。
- API 原子封装始终在 `src/api/<domain>.ts`；页面级 API 编排才放 `*Actions.ts` / `use*Page.ts`。
- 组件按**变化原因**拆，不按行数机械拆：非声明式页面或组件超过约 600 行要主动评估拆分；超过约 1000 行必须按「页面编排 / 局部组件 / actions / helpers」小步拆分。
- 单次任务只做增量拆分或增量迁移，不顺手重写整套页面风格或路由结构。

已按此模式拆分的页面：`user`、`seat-swap`、`home`、`rankings`、`ticket-watch`、`support`（`useSupportPage.ts` 管加载/助力/倒计时，`useSupportPoster.ts` + `poster.ts` 管拉票海报）、`matches`（`MatchesContent.vue` 与独立入口共享 `useMatchesPage.ts` / `presentation.ts` / `components/`）。`home`、`ticket-watch`、`matches` 可作为页面编排参考。

### 首页今日话题

- `home/index.vue` 编排原有 overview / 赛果、支持面板及话题事件，话题独立加载，复制原文链接由页面执行。
- `api/dailyTopics.ts` 是唯一话题 HTTP 入口，读取 `/daily-topics?team_id=77680&limit=5`；`home/useHomeDailyTopics.ts` 管理独立请求、revision、错误与阅读选择，404 明确显示接口待接入。
- `home/dailyTopics.ts` 纯函数验证球队、内容 ID、文章与来源、真实日期；去重且最多五篇。不使用积分榜生成新闻或空列表的伪内容。
- `home/useDailyTopicAudio.ts` 管理 `uni.createInnerAudioContext` 生命周期，只播放对应文章的 ready HTTPS 四川话音频。批次、正文、链接或音频状态改变停止旧播放，隐藏暂停、退出销毁。
- `HomeDailyTopics`、`HomeDailyTopicCard`、`HomeDailyTopicSheet`、`HomeTopicAudioEntry`、`HomeTopicAudioPlayer` 均使用 props / emits，无业务请求。来源详情随文章展示标题、发布机构和日期，不新增未注册 web-view 路由。
- `types/dailyTopic.ts` 定义页面展示模型；`home/dailyTopics.test.ts` 验证条数上限、去重、空内容、来源、音频校验与播放签名失效。

## 生命周期与刷新约定

二级页的返回/分享按钮由 `FiSecondaryNav` 展示，页面通过 `secondaryNavigation.ts` 处理返回与 H5 公开链接复制；设置与支付页仅保留返回。话题详情的 `onShareAppMessage` 使用当前文章 ID 的详情路径，`share.ts` 从全文段落生成节选并负责按 Unicode 换行，`useTopicShareCard.ts` 在正文加载后预生成 5:4 Canvas 图片，`shareCardPhoto.ts` 限时下载第一张配图到本地，`shareCardDrawing.ts` 完整显示首图并与全文节选并排绘制，无图或加载失败使用文字卡片；首图变化也会使分享缓存失效；未完成时通过微信分享回调的 `promise` 等待生成。旧文章、错误页及销毁后的绘制结果按 revision 丢弃，同一 canvas 的任务串行执行，避免分享到旧正文。分享卡片不使用摘要替代全文，也不携带认证数据。

- `App.vue` 以后台方式执行启动恢复（`bootstrapSession`，仅 MP 允许微信静默探测，认证恢复不依赖审核状态）和公共配置/审核态预加载；公开页面不等待会话恢复，审核查询也不阻塞认证；无 token 的 H5 不自动登录。
- 审核展示与登录独立：「我的」页始终保留登录/首次绑定/退出，审核中登录后展示基础账号，会员内容仍按审核状态隐藏；首次审核查询完成前也不提前展示会员内容。
- 页面 `onShow` 走共享 store 入口（`ensureSessionReady` / `ensureRuntimeConfig` / `ensureMiniReviewReady`），不各自缓存 currentUser/config。
- 后台恢复/登录发布 `currentUser` 后，已在前台的页面通过 watch 会话状态刷新私有区，无需再次切 Tab：`user` 页经 `accountEffects.handleSessionRevisionChange`（同 revision 并发加载合并），首页助力面板经 `home/sessionEffects`，洞察页经 `insights/sessionEffects`（审核中不拉会员数据），余票页经 `ticket-watch/sessionSync`（恢复只重载数据，不自动开启监控/轮询，隐藏页面不启动轮询）。
- 余票当前看板有生命周期 generation（`useTicketWatchState.currentBoardGeneration`，单调递增不归零）：reset/切换球队/比赛上下文替换时递增；订阅状态、套餐加载、下单、支付确认及比赛 ID 解锁捕获 generation + session revision + 球队/比赛快照，每个异步阶段校验；同一 generation 内重叠操作以请求 owner 序号判定谁清 loading。同一比赛的普通库存轮询不递增 generation。
- 首页与洞察私有读取等待共享会话确认并按 owner 合并；跨会话先清缓存/选择，旧结果、错误和 finally 放弃。首页有效主队 PUT 后强制刷新，旧账号 PUT 不影响新读取；洞察首个 await 前捕获私有链身份，审核 gate 独立于账号。
- 余票 board owner 与订阅 generation 独立：reset 后立即允许新 current/history 请求；恢复先重载可见看板，真实 activeTab watcher 补载尚未加载的目标，已载 cache 复用并遵守 hidden/审核/会员门控。历史列表与用户选择每次库存读取共用单调 owner，旧 cache/display/error/toast/finally 不发布。
- 实际比赛替换统一同步复位订阅/解锁状态，正常同比赛 poll 保留合法操作。解锁 open/close/refresh/pay 按操作 owner 防护，支付开始使旧权益查询失效；原生支付返回后重新验证上下文，失效页面不再继续确认读取，已发出的真实支付不取消。
- 新写的异步流程统一用 `utils/latestGuard.ts`（`createLatestGuard(() => [session.revision.value, 业务 key])`）防旧结果覆盖：`begin()` 取令牌，发布结果前查 `isCurrent()`，`finally` 里 `owns()` 才清 loading；重新加载等需要作废在途操作时调 `invalidate()`。存量页面的手写 owner/generation 触碰时再迁移。海报类 canvas 的圆角路径、绘制提交、导出与保存相册统一用 `utils/canvasPoster.ts`。
- 共享数据失败允许下一次页面 `onShow` 或显式重试；不新增常驻定时器。余票看板轮询随页面离开/账号切换停止（`useTicketWatchPolling.ts`）。
- 统计口径：抢票/余票/回流相关统计统一以 `sale_start_at + 10 minutes` 起算，缺起售时间的比赛报错或跳过该统计入口。

## 新增页面 Checklist

- 先搜索已有同类页面、组件、helpers 或 API 封装可复用；后端字段以 `src/types/` 和 `src/api/` 实际 DTO 为准，不臆造。
- 页面是否只做编排；是否需要页面局部 `components/`；转换逻辑是否进 `helpers.ts` / `src/utils/`。
- 运行时组件直接从 `.vue` 文件导入（不走 barrel），新增页面/组件后跑 `bun run build:mp-weixin` 确认编译与组件注册检查。
- 涉及路由、接口调用、权限、数据提交、共享工具函数或关键业务状态时，按风险补行为测试（`helpers.test.ts` 模式）；纯 UI 以 type-check、构建和模拟器/截图验证为准，不机械新增测试。

## 验证命令

```bash
bun test                              # 全量行为测试
bun run type-check                    # vue-tsc
bun run check:boundaries              # import 边界
MINI_REVIEW_SKIP=1 bun run build:mp-weixin   # 离线 MP 构建（不登记版本）
bun run build:h5                      # H5 构建
```

发布链路（Git 门禁、版本登记、产物校验、上传）统一走 `scripts/mini-release.mjs`，见 README「微信小程序发布」。
