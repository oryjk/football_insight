# 足球小程序与报名系统工程对齐设计

日期：2026-10-09。状态：交接设计，尚未实施。

## 1. 目标与分工

用户已确认以报名系统为工程基线，并指定另一位 agent 实现、本对话负责 review；随后明确足球小程序不再需要 AI 功能。目标是移除小程序 AI 功能，并统一会话、发布和页面职责，减少重复状态及跨端问题；足球的产品、数据契约和视觉身份继续由本项目决定。

本设计与 [实施计划](../plans/2026-10-09-mini-registration-alignment.md) 一起交接。首批实现先移除 AI，再完成会话集中管理、所有剩余会话消费点迁移、运行配置共享、发布入口和基础组件规范。独立 H5 桥接及分包是后续独立项目，前置条件见第 8 节。

## 2. 核对基线

- 足球仓库：`3a27e5837ae17ba9b59c03790497d942868f1bed`。
- 报名系统参考：`18713e2`；其工作区存在未提交的活跃榜 UI 改动，不属于本次迁移依据，不修改该仓库。
- 两个 mini 的声明版本相同：uni-app `3.0.0-4080420251103001`、Vue `^3.4.21`、TypeScript `^4.9.4`、Vite `5.2.8`，工具统一 Bun。
- 报名系统通过模块级 `ref/computed` 实现 store；没有 Pinia。足球文档声明了 stores 职责，实际尚无 `src/stores/`。
- 足球 `api/auth.ts` 的登录/注册/绑定/测试登录会写 token，logout 在 finally 清 token；`getCurrentUser()` 仅对 401 返回 null，其他错误抛出。
- 足球首页和 `useAiChatSheet.ts` 有失败后清 token 的逻辑。AI 专用逻辑随功能删除，剩余业务的认证处理收口到 session。
- 足球生产 API 是 `https://match.oryjk.cn/api/v1`；编译模块导出 `API_BASE_URL`，不同于报名系统的 `getApiBaseUrl()`。
- 足球已有 AST 边界检查、组件注册检查和产物 API 检查；本次调查运行 `bun run check:boundaries` 通过。未运行完整构建或真机验证。

实现前重新核对上述源码与提交。若其他 agent 已改变接口，应更新计划中的路径和测试，不照搬旧快照。

## 3. 方案选择与边界

选择在现有 uni-app 项目内增量对齐。完整复制报名系统会引入球队/身份等足球不需要的状态；只修局部重复代码则不能解决跨页会话及发布约束。

首批不升级框架、不加入 Pinia、Wot UI、前端 DDD、通用 port/adapter 或额外 `uni.*` 包装。报名系统组件实现和流程可参考，不能复制其 Go DTO、`/api/v1/app` 前缀、球队身份、主题皮肤、旧 Neo 风格及微信自动登录策略。

不改 Rust、Python、数据库、域名配置、原生 TabBar、路由和生产部署。所有抢票/库存/回流统计继续以 `sale_start_at + 10 minutes` 起算，缺起售时间仍报错或跳过；贡献预计算仍由 scraper 写入 `f_i_team_insights`。

### 3.1 移除小程序 AI

- 删除 FiBrandNav 的“嗡嗡嗡～～”AI 按钮、open-ai 事件、openOnCurrentPage 属性、跳首页打开 AI 的 intent。保留足球品牌、透明导航能力及微信胶囊旁品牌位置，拆除按钮专属定位后仍验证两端布局。
- 移除首页、榜赛、洞察、换座、我的中的聊天 sheet、事件、状态、用户/config 加载。首页公共配置还服务公告和会员规则，不能随 AI 一并删除。
- 删除小程序专用 FiAiChatSheet、useAiChatSheet、api/ai、types/ai、config/aiChat、aiEntryIntent、aiChatStorage、静态小罗头像及专属测试。确认引用后删除 wechatCloud 工具及 App.vue 云初始化；保留普通微信登录/支付/订阅消息能力。
- 移除 frontend 的 AiChatMode/ai_chat_mode 消费和 AST 中两个 AI 例外，不改后端返回字段/路由，不改独立 H5/Android。`useAnimatedInteger` 服务洞察数字动画，继续保留。
- 清理现行文档及无引用的 AI 专用依赖，保留历史设计追溯；不为删除功能引入批量清除用户本地历史数据的操作。
- 验收要求两端所有页面无 AI 入口/弹层，页面切换不调用 AI 接口/初始化 AI 云能力，主流程及品牌布局正常；不能只用 v-if 隐藏入口而保留整套运行链。

## 4. 会话设计

依赖方向：页面/composable → `stores/appSession.ts` → 原子 `api/auth.ts` → request → authStorage。展示组件继续使用 props/emits，不能通过 store 绕过现有禁止调用业务 API 的边界。

### 4.1 状态与职责

- 模块级共享 `currentUser`、`hasAccessToken`、`status`、`errorMessage`、`revision`，对消费者暴露只读 ref/computed。
- `status` 为 `guest | loading | authenticated | error`。token 存在与已认证分开；权限依据已确认用户及后端授权，不把“有 token”当会员证明。
- 存储 key 继续为 `fi_access_token`。API 层不再写/清 token；只有 session 的有效结果才能发布用户与 token。request 仍只读取 token、归一网络错误，不注册全局 401 回调，不导航。
- 首屏无 token 时直接游客态，不调用 `/auth/me`，不静默微信登录。现有用户点击微信登录、binding_required、邀请码/密码、H5 白名单登录及登录后跳转保持原语义。
- 同一 revision 的初始化/刷新合并在途请求，成功用户缓存 60 秒。`force=true` 绕过成功缓存，仍合并同一 revision 在途请求；会员购买成功后强制刷新。
- 新登录意图、成功替换 token、退出及认证失效增加 revision。异步结果、错误、finally 均验证捕获的 revision、token 快照和请求身份；登录成功替换 token 时再次增加 revision，使登录等待期间使用旧 token 发出的刷新也失效。旧请求不能写 token、清新账号、清新 loading，或触发旧跳转。
- `getCurrentUser()` 返回 null 代表 401，当前 revision 清本地会话；断网、超时、403、5xx 抛错并保留该账号已有 token/用户，记录可重试错误。无已确认用户时不开放会员功能。
- 被替代的操作抛 `SessionSupersededError`，页面识别后静默放弃该次成功提示/跳转，不显示成普通网络失败。
- logout 先发起带旧 token 的后端注销请求，再同步清本地会话；后端失败也维持退出。旧 logout 完成不能清掉新登录账号。

### 4.2 消费方

“我的”及设置页的所有登录入口改用 session action。首页、洞察、会员购买、余票页面共享用户；换座、支持投票、战术板等使用共享登录标记。页面专属数据、筛选、弹层和轮询留在原页面模块。

账号 revision 改变时，页面立即清理旧账号的私有列表、邮箱、订阅及会员结果，停止旧账号轮询；相关异步数据发布也校验 revision。公共赛果/榜单不需要随退出清空。不要借本任务重写现有 ticket-watch 选择请求序号和轮询策略。

## 5. 运行配置与审核态

- `stores/runtimeConfig.ts` 共享 `PublicSystemConfig`；`stores/miniReview.ts` 共享 `MiniReviewStatus`。两者独立加载、独立错误，不能一个失败导致另一个失效。
- 成功值缓存 60 秒，同类并发调用合并；失败不标成永久已加载，下次页面 onShow/显式重试可重新请求，不新增常驻轮询。
- 公开配置保留最后一次成功值；审核查询失败返回 null，兼容现有 `resolveSystemConfigUnderReview(null) === false`。同时暴露 `ready/available/errorMessage`，不能把失败伪造为“后端确认已过审”。
- 本次不借对齐调整审核失败时的业务显隐。首次未知、后续刷新失败、管理员切换的页面行为均以现有实现建立基线，再保持一致。
- 管理员成功 PUT 后立即发布响应中的审核值，并使旧在途 GET 失效；所有消费者同步更新。用户管理权限和白名单仍由后端决定。
- `utils/systemConfig.ts` 作为兼容入口委托 store，纯转换函数继续保留。审核状态使用本项目 `football_insight_mini` 和生成版本号，不迁入报名系统项目编码。

## 6. 发布设计

`scripts/mini-release.mjs` 成为 build/upload/preview/verify 的统一入口。`mini-ci.mjs` 负责 SDK 调用，并在真实加载 SDK 前执行可复用的校验；不要形成两个脚本互相 import 的循环。

- 正式 upload 校验 monorepo：分支 main、工作区干净、fetch 成功、HEAD 等于刷新后的 origin/main。版本分配改变源文件时，在构建和上传前停止，提交推送版本文件后再执行。构建后再次检查。
- build 允许 feature 分支；离线 `MINI_REVIEW_SKIP=1` 不请求登记库。upload 强制登记，不继承离线 skip。preview 不登记/重建，不要求上传 Git 门禁，但必须验证已有生产产物。
- 保留足球 `/mini-review/allocate`、project code、app id 和 robot 默认 1 / 用户测试 2；不调用报名系统登记库。
- 用 Bun `--no-env-file` 执行脚本。清理继承的 `VITE_*` / `UNI_*`，只读取显式 `.env.production` 建立客户端变量，固定 NODE_ENV=production、uni `--mode production`。生产 API 必须精确为 `https://match.oryjk.cn/api/v1`。
- 隔离 Vite 自动读取 `.env.local` / `.env.production.local` 的覆盖层：统一入口创建临时空 envDir，由 `FOOTBALL_RELEASE_ENV_DIR` 传给 `vite.config.ts`；客户端变量来自前述显式生产文件。该变量只由入口内部生成并在 finally 清理目录。日常 dev/H5 未设置时保持现有配置。
- 客户端变量不包含私钥/API key；保留 CI/mini-review 所需非客户端变量，日志不打印密钥或完整环境。所有 `.env`、证书、私钥均不提交。
- 产物检查读取本项目 `exports.API_BASE_URL` 的真实值，不能只搜索字符串或因 require 缓存读到旧值。缺 app.json/模块、错误地址、执行异常/超时立即失败；求值环境不提供网络、process 或 require。
- 入口显式调用边界检查及版本同步，删除 prebuild 中重复登记链；保留构建后组件注册校验。任一步失败即终止，不能上传旧产物。

## 7. 页面与基础 UI

新增 `football_insight_mini/docs/mini-architecture.md` 和 `mini-design-system.md`，分别成为结构约定和足球视觉规范的入口。三层 token 和 Fi 命名继续沿用；更正 token 注释中“对齐报名系统 Neo”的过期描述。

首批 UI 只以已有 `FiLoading` 作示范：保持必填 `title/caption`、既有布局及颜色，用足球 token 表达存量样式。给 `FiBottomSheet` 记录既有 props/emits、滚动锁由 page-meta 所属页面负责的约定，本次不迁移动效或替换弹层实现。纯视觉修改用构建/模拟器/截图验收，不机械新增快照断言。

后续新组件按职责建立按钮、状态卡、弹层等统一契约；是否采用 Wot UI 由独立迁移试点决定，不作为首批依赖。运行时组件直接 import `.vue`，父页面 wrapper 负责跨组件布局，展示组件不碰 session/API。

## 8. 后续独立项目与进入条件

| 项目 | 当前事实 | 进入条件与交付边界 |
| --- | --- | --- |
| H5 一次性 code 桥接 | 独立 football_insight_h5 已存在，支持 URL token；当前 mini 没有 web-view 入口 | 先确认足球 Rust 是否有可复用的签发/兑换接口，再单独设计 TTL、单次消费、目标域名及旧客户端兼容；新增接口、mini 嵌页和 H5 兑换一起验收。首批不复制报名系统路由，不删除现有 token 接入。 |
| 普通分包 | 足球当前没有 pages.json subPackages | 先测主包实际大小，列出候选页面及静态引用；稳定保留五个 Tab 在主包，新增编译后跨包检查。无体积证据不机械分包。 |
| 更多基础 UI / Wot | 足球 Fi 组件承担自有视觉 | 首批稳定后选一个页面做试点，提供 H5/MP 对照和体积变化，review 后扩展；不批量改皮肤。 |
| 报名系统反向引入边界检查 | 足球已具备 AST 检查 | 单独在报名系统开展，不计入足球首批验收，不触碰其当前未提交工作。 |

## 9. 验收与 review

按实施计划的任务逐项交付源码提交、实际测试输出和两端验证证据；用户把提交/分支或 PR 交回本对话后进行 review。review 不由实现 agent 自报通过替代。

重点检查：AI 入口或运行链残留；删 AI 误删公共配置/数字动画/微信登录；旧账号请求覆盖新账号；网络失败误退出；会员变化不刷新；审核配置失败永久缓存；生产构建环境污染；版本更新产生脏树后仍上传；展示组件经 store 隐式请求。

首批通过需要：相关行为测试、type-check、边界检查、离线 MP 构建及组件/API 校验、H5 构建、关键两端场景。没有真机/模拟器权限时标注“未验证”，不能把构建通过当成视觉通过。该计划不授权发版、数据库迁移、生产重启或 Nginx 修改。
