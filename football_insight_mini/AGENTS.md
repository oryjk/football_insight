# football_insight_mini 前端约定

## 审核版本号

审核版本号由本项目后端的 mini-review 登记库统一分配（`f_i_mini_review_statuses` 表），**不再手动维护**。

- 发版用 `bun run mp:release`：构建前 `scripts/sync-manifest-version.mjs` 会调 `POST /api/v1/mini-review/allocate` 申请版本号，并同步写入 `src/manifest.json` 和 `src/config/generatedMiniProgramVersion.ts`。
- 登记库是唯一权威：最新版本仍在审核中则复用，已出审核则 `+0.0.1`；不要手动改 `manifest.json` 的 `versionName`。
- 如果用户说"改版本号"或"指定版本号"，用显式覆盖：`MINI_PROGRAM_VERSION=x.y.z bun run build:mp-weixin`（作为指定版本传给登记接口）。
- 纯本地离线构建（不登记、不递增版本）用 `MINI_REVIEW_SKIP=1 bun run build:mp-weixin`。
- 微信审核结束后要调 `PUT /api/v1/mini-review/review-status` 把当前版本标记 `is_reviewing=false`，否则后续构建会一直复用审核中的版本号。
- 若构建分配了新版本号，`src/manifest.json` 与 `src/config/generatedMiniProgramVersion.ts` 的变更需要单独提交。

运行时来源：`src/api/system.ts` 的 `MINI_PROGRAM_VERSION` 优先读 `src/config/generatedMiniProgramVersion.ts`，构建时生成。

## 分层设计：pragmatic layering（对齐 registration_system_mini）

采用注册系统同款的轻量分层，**明确排除**前端 DDD、use-case/port/adapter 严格分层和 `uni.*` 二次适配——uni-app 已承担跨端适配层，再包装只增加跳转成本，收益不足以覆盖复杂度：

```text
Page（index.vue 编排）
  ├─ use<Page>Data / use<Page>Page   # 只有重编排页面才抽 composable（多接口、轮询、复杂状态）
  ├─ api/                            # 原子 HTTP 调用（唯一碰 HTTP 的层）
  ├─ helpers / utils                 # 纯计算与 ViewModel 转换（可测部分全部在这层）
  ├─ stores                          # 仅真实的跨页面共享状态（登录用户、审核态配置）
  └─ components/                     # props 输入、emits 输出
```

请求层约定（`utils/request.ts` + `utils/apiError.ts`，对齐注册系统实现）：

- 错误统一为 `ApiRequestError`（`statusCode` + `networkFailed` + 响应体），判断用 `isUnauthorizedError()` / `isNetworkUnavailableError()`，**禁止按错误文案猜测状态**。
- 401 的清理登录态由消费方决定（如首页清 token 转登录浮层）；请求层不做全局导航，也**不做全局 401 回调注册**。
- 断网/超时文案由请求层归一，并发失败只弹一个提示框。

依赖方向由 `scripts/check-import-boundaries.mjs` 机器强制（已接入 `prebuild:mp-weixin` 与 GitHub Actions `.github/workflows/mini-boundaries.yml`，提交与发版链路都会执行）。import 与 `uni.request` 用 TypeScript AST 提取（Vue 只取 `<script>` 块，脚本依赖 devDependency `typescript`），说明符再解析成绝对路径判定，注释、字符串、模板文本里的示例代码不会误报：

1. 组件层（`src/components`、`src/pages/**/components`）禁止 import `api/**` 与认证存储；`src/components` 还禁止反向依赖页面层；
2. 页面 A 的任何文件禁止 import 页面 B 的任何模块（helpers、页面容器、局部组件一律算）——跨页面共享：逻辑放 `src/utils`，组件进 `src/components`；
3. `api/**` 只能依赖 request/config/types/utils（api 内部互引合法）；
4. 只有 `utils/request.ts` 可以直接调用 `uni.request`。

文件身份：pages.json 注册的页面入口和页面目录直属的 `*.vue`（`index.vue`、`RankingsContent.vue`、`MatchesContent.vue` 这类页面容器）属于页面层，允许依赖 api；只有 `components/` 子目录里的才算展示组件。页面深层既未注册也不在 `components/` 下的 `.vue` 会被直接判违规，逼着显式登记身份。

例外必须显式登记在脚本的 `ALLOWLIST`（文件 × 规则，跨页例外精确到放行目标路径）并注明原因与清理计划（当前：`FiAiChatSheet` 组件内调 AI api —— feature 模块待拆分；`api/ai.ts` 流式 `uni.request` —— enableChunked 无法走统一封装；`rankings/index.vue` 跨页内嵌 `matches/MatchesContent.vue` —— 榜单 tab 内嵌整个赛程视图，仅放行这一个目标）。允许/禁止案例的回归测试在 `scripts/check-import-boundaries.test.ts`（fixture 树：`scripts/fixtures/import-boundaries/`），改检查规则时必须同步跑。

Store 约定：只解决真实的跨页状态（登录用户、审核态配置）；页面 loading、弹层开关、筛选和表单状态留在页面/composable，不把请求结果全局化。

目录职责：

```text
src/
  api/           # 按业务域封装后端 API 原子调用，不承载页面 UI 状态
  components/    # 跨页面通用组件（Fi* 前缀命名），只放 API 稳定、真正复用的
  composables/   # 跨页面复用的组合式逻辑（use*）
  config/        # 运行时与环境配置
  pages/         # 页面；页面专属模块放各页面目录内
  styles/        # design token（fi-tokens.css）
  types/         # 后端 DTO、视图模型等共享类型
  utils/         # 请求、存储、工具方法
```

页面拆分模式（优先结构）：

```text
src/pages/<domain>/
  index.vue                 # 页面编排：生命周期、加载状态、导航、事件 wiring
  helpers.ts                # 本页面纯函数/格式化/视图模型转换（现状命名，等价于 *State.ts）
  use<Domain>Page.ts        # 可选：较重的页面级状态与动作编排 composable
  <domain>Actions.ts        # 可选：页面级提交动作和 API 编排
  components/               # 页面局部组件（XxxPanel/XxxCard/XxxSheet）
```

职责边界：

- 页面 `index.vue` 只承担编排：生命周期、页面业务状态、异步流程、错误处理、路由、Toast/Confirm。**不要继续往大页面里堆模板和请求。**
- 页面专属组件放 `src/pages/<domain>/components/`；只有稳定跨页面复用的组件才进 `src/components/`，命名带 `Fi` 前缀（如 `FiBottomSheet`）。
- 子组件通过 `props` 接收数据、`emits` 发出意图；不要在页面局部展示组件里直接调用业务 API。父页面保留业务状态和异步流程。
- 后端数据到展示模型的转换放页面 `helpers.ts`；跨页面复用的放 `src/utils/`。不要把转换逻辑散落在模板里。
- API 原子封装始终在 `src/api/<domain>.ts`；页面级 API 编排才放 `*Actions.ts` / `use*Page.ts`。
- 组件按**变化原因**拆，不按行数机械拆：非声明式页面或组件超过约 **600 行**要主动评估拆分；超过约 **1000 行**必须按「页面编排 / 局部组件 / actions / helpers」小步拆分。
- 当前超标文件（重构 backlog，触碰时优先拆分，不要顺手再加量）：`ticket-watch/index.vue`（4100+ 行）、`matches/MatchesContent.vue`（1100 行页面容器，已过 1000 行阈值）。`user`、`seat-swap`、`home` 均已按局部组件模式拆分完成（`home`：`index.vue` 只做编排 + `components/` 局部组件，可作参考样板）。
- 单次任务只做增量拆分或增量迁移，**不要顺手重写整套页面风格或路由结构**。

## Design token 规范

`src/styles/fi-tokens.css` 按三层组织（由 `App.vue` 全局导入，兼容 `uni.css` 旧变量），覆盖**颜色、间距、字号、行高、圆角、边框、阴影、字重**八个维度：

1. **primitive**：原始值（`--fi-primitive-*`），不直接在页面引用；
2. **semantic**：语义别名，结构 UI 统一引用这一层——颜色 `--fi-color-*`；间距 `--fi-space-<rpx>`（数字刻度，如 `--fi-space-18` = 18rpx）；字号 `--fi-font-<rpx>`；行高 `--fi-leading-*`；圆角 `--fi-radius-*`；边框/分割线 `--fi-border-*`、`--fi-color-divider*`；阴影 `--fi-shadow-*`；字重 `--fi-weight-*`；
3. **component**：组件契约（`--fi-component-*`），组件级复合样式引用（卡片、kicker/标题、关闭按钮等）。

规则：

- 新增页面、组件、共享壳层的颜色**必须引用 token**（`var(--fi-color-*)`），不要新增散落 hex；需要新色值时先加 primitive，再建 semantic 别名。
- 存量约 900 处散落 hex 按「触碰哪个文件就顺手迁移哪个」增量替换，不要求一次性重写；高频色（白、墨色、灰阶、红系、浅底）已有对应 token。
- 间距、字号、行高、圆角、边框、阴影、字重同样必须引用 token（刻度内取值优先；`--fi-space-*`/`--fi-font-*` 数字刻度覆盖高频值），不要在页面里硬编码这些值；刻度外的值先评估就近归档（视觉差 ≤2rpx 可并入相邻刻度），确需新值再进 token 文件。
- **颜色例外**：球队/球衣等业务数据值、插画装饰色（如球场图、海报绘制色）不是视觉 token，保留原样，可在相邻注释标注 decorative。

## 跨端约束（H5 与小程序兼容）

本项目同时编译 H5 和微信小程序，必须保证两端可编译可运行：

- 统一使用 `uni.*` API，不要直接调 `wx.*`；禁止浏览器 DOM API（`document.*`、`window.*`、`localStorage` 等），存储用 `uni.getStorageSync` / `uni.setStorageSync`。
- 微信专属能力（支付、订阅消息、`open-type` 等）必须用条件编译 `<!-- #ifdef MP-WEIXIN -->` 隔离，并提供 H5 降级路径。
- 样式统一 `rpx`，不混用 `px`/`vw`/`rem`；不用 `:hover`（用 `hover-class`）。
- **运行时 Vue 组件必须直接从 `.vue` 文件导入**（如 `import FiLoading from "@/components/FiLoading.vue"`），不要通过 barrel 文件二次导出运行时组件——uni-app 小程序编译器可能不追踪二次导出，导致 WXML 有标签但 JSON 缺 `usingComponents`，构建成功却渲染不出。类型和纯函数可以走 barrel。
- 新增页面/组件后执行 `bun run build:mp-weixin` 确认编译；构建尾部会自动跑组件注册检查，报 `Unregistered mini-program components` 时先核对 `.vue` 直接导入。

### mp-weixin 样式/布局陷阱（勿再犯）

小程序端 uni-app 组件编译为微信自定义组件，存在宿主节点与样式隔离，以下写法 H5 正常、小程序静默失效：

1. 不要用 `custom-class` + 父级 scoped 样式给子组件根节点做布局——父级选择器带作用域 id 且被组件样式隔离挡住，整条规则失效。布局类加在自己模板内的包裹 view 上。
2. flex 行容器里子组件宿主节点会收缩为内容宽，子组件内部 `width: 100%` 无法撑满。由 flex 容器侧解决（`flex-direction: column` + `align-items: stretch`）。
3. 通用原则：**H5 显示正常不代表小程序正常**，跨组件宽度/布局样式必须在 mp-weixin 端实际验证。

## 测试与验证

- 不按 TDD 开发前端；页面、样式、交互、UI 调整以 `bun run type-check`、构建和模拟器/人工验证为准，**不机械新增测试**。
- 只有涉及路由、接口调用、数据提交、权限、共享工具函数或关键业务状态变化时，才按风险补必要测试（现有 `helpers.test.ts` 模式保留）。
- 提交前至少 `bun run type-check`；涉及页面流程或路由时补跑 `bun run build:mp-weixin`。

## 小程序弹框滚动锁

微信小程序里如果弹框、底部 sheet、全屏 mask 打开后需要禁止底层页面滚动，优先使用真正页面入口里的 `page-meta` 做页面级锁：

```vue
<page-meta :page-style="pageLockStyle" />
```

```ts
const pageScrollLocked = ref(false)
const pageLockStyle = computed(() => pageScrollLocked.value ? 'overflow: hidden;' : '')
```

注意：

1. `page-meta` 必须放在 `src/pages/**/index.vue` 这类真正的小程序页面文件里。
2. 如果弹框逻辑在子组件中，子组件不要自己写 `page-meta`；应通过 `emit` 通知父页面切换 `pageScrollLocked`。
3. 只给普通 `view` 加 `height: 100vh; overflow: hidden` 通常锁不住小程序页面级滚动，只能作为辅助。
4. mask 仍然需要配合 `@touchmove.stop.prevent`，弹框内容区可用 `@touchmove.stop`，防止触摸事件穿透到底层。

这个项目里球队赛季弹框的滚动穿透问题，最终就是通过父页面 `page-meta` + 子组件 emit 锁状态解决的。后续遇到同类问题，优先按这个模式处理。
