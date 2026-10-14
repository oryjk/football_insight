# football_insight_mini

Football Insight 的微信小程序前端，采用 `uni-app + Vue 3 + TypeScript`。

当前策略不是替换现有 H5，而是在 monorepo 中作为独立前端子项目推进小程序版本。

协作文档：

- [AGENTS.md](AGENTS.md)：前端约定（分层、token、跨端约束、发布）
- [docs/mini-architecture.md](docs/mini-architecture.md)：结构规范（目录职责、依赖方向、stores/会话、页面拆分）
- [docs/mini-design-system.md](docs/mini-design-system.md)：视觉规范（三层 token、布局责任、Fi 组件契约）

## 小程序信息

- AppID: `wxc61da17a97f6eb1b`

## 小程序微信登录

当前小程序端只保留微信原生登录入口：

- 微信一键登录

微信一键登录使用的是独立于 H5 的新链路：

1. 用户点击“微信一键登录”
2. 小程序调用 `uni.login()` 获取 `code`
3. 请求后端 `POST /api/v1/auth/mini-wechat/login`
4. 如果当前微信已绑定账号：
   - 直接登录成功
5. 如果当前微信尚未绑定账号：
   - 弹出补全层
   - 可选填写邀请码
   - 选择头像
   - 输入昵称
6. 请求后端 `POST /api/v1/auth/mini-wechat/bind`

当前这条链路的产品约束是：

- 不要求补用户名
- 不要求补手机号
- 不要求设置密码
- 首次完成绑定后，后续可以直接用微信进入

### 启动静默恢复（第一阶段，2026-10-10 起）

小程序启动时会在后台自动恢复会话，不阻塞公开页面：

- 已有有效 token：通过共享 `/auth/me` 恢复同一账号，不调用微信 code 接口
- 无 token 但当前微信已绑定账号：后台调 `uni.login()` + `/auth/mini-wechat/login` 恢复原账号，无 toast、无跳转
- 首次未绑定（`binding_required`）：保持游客，不自动建号、不弹绑定表单、不写 token；需要登录时由用户显式点击触发
- 用户主动退出：持久化手动退出标记（`fi_manual_logout`），跨冷启动禁止自动恢复；只有显式认证操作（密码/注册/微信/绑定/测试登录）才解除
- 旧 token 401 失效：允许启动恢复尝试一次微信登录；断网/超时/403/5xx 保留已有会话，不探测微信
- 审核状态与认证独立：审核期间正常显示登录入口，允许微信静默恢复、显式登录、首次绑定和退出；审核查询不阻塞认证，会员/支付等展示规则继续保留
- 显式登录、注册、绑定或退出会替代本次启动的自动恢复，即使显式尝试已结束，迟到的启动恢复也不覆盖表单
- H5 不自动探测微信；已有 token 仍走共享会话恢复，无 token 时保持显式登录（密码/白名单测试账号）

详见 [docs/mini-architecture.md](docs/mini-architecture.md) 「请求层与会话」。

## 与 H5 的主要差异

小程序整体页面结构和数据模型是基于 H5 迁移过来的，但用户登录这块**不是照搬 H5**。

- H5 仍以“邀请码注册 + 账号密码登录 + 网页微信授权/绑定”为主
- 小程序改为“仅保留原生微信一键登录”
- 小程序微信登录依赖 `uni.login()` 和后端的 `/api/v1/auth/mini-wechat/*` 接口

这块是当前 H5 与小程序最大的行为差异，后续排查问题时需要把两条链路分开看。

### 头像处理

当前小程序端不会先把头像上传到对象存储。

而是：

- 用户用 `chooseAvatar` 选择头像
- 前端用 `getFileSystemManager().readFile(..., encoding: 'base64')`
- 把头像转成 `data:image/...;base64,...`
- 作为 `avatar_data_url` 提交给后端

后端会把这个值直接写入 `f_i_users.avatar_url`。

## 技术栈

- uni-app
- Vue 3
- TypeScript
- Bun

## 当前迁移范围

- 首页
- 榜单页
- 洞察页
- 赛程页
- 我的页
- 球队战术板详情页

当前第一版先打通：

- 小程序页面结构
- 线上 API 请求层
- 登录态本地存储
- 主要页面的数据读取和基础展示

## 本地开发

```bash
cd football_insight_mini
bun install
bun run dev:mp-weixin
```

然后用微信开发者工具打开：

- `football_insight_mini/dist/dev/mp-weixin`

如果微信开发者工具是 Windows 侧打开，推荐直接同步产物到 Windows 目录再导入：

```bash
bun run sync:mp-weixin:windows
```

或者直接一键启动编译和同步：

```bash
bun run dev:mp-weixin:windows
```

然后让微信开发者工具导入：

- `E:\projects\football_insight_mini`

## 构建

```bash
bun run build:mp-weixin
```

## 问题排查

- [小程序排查记录](docs/mini-troubleshooting.md)

## 接口说明

小程序不走 Vite 代理，请求地址规则是：

- 开发环境默认请求 local233 测试后端：`http://172.16.60.233:18092/api/v1`（需能访问该内网）
- 生产环境默认请求线上 API：`https://match.oryjk.cn/api/v1`

如需切换，可通过环境变量覆盖：

- `VITE_API_BASE_URL`
- `VITE_MINI_PROGRAM_VERSION`

小程序审核态判断走本项目后端的 mini-review 登记库（与发版时的版本号登记同库），请求：

- `GET /api/v1/mini-review/review-status?project_code=football_insight_mini&version={version}`

其中：

- `project_code` 固定为 `football_insight_mini`
- `version` 使用构建时生成的 `src/config/generatedMiniProgramVersion.ts` 里的版本号

生产构建中，如果接口返回 `is_reviewing=true`，小程序会进入审核模式：不加载会员、支付、用户会员等级、回流看板会员权益等相关信息。

通过 `bun run dev:mp-weixin` 启动的开发构建使用 `import.meta.env.DEV` 判断，忽略审核状态对功能展示的限制；「我的」、洞察和余票看板正常展示。设置页仍显示真实审核状态。这里的开发构建不等同于微信后台的开发版本或体验版：通过 `build:mp-weixin` / `mp:release` 生成的生产包继续按审核状态展示。

API 地址默认值：

- `bun run dev:mp-weixin` 默认直连 `http://172.16.60.233:18092/api/v1`，配置位于 `src/config/apiBase.ts`；微信开发者工具和真机需能访问 local233 内网 IP。
- `bun run build:mp-weixin` 默认连线上 `match.oryjk.cn`

也可在启动开发构建时显式指定同一后端，无需新增或提交 `.env` 文件：

```bash
VITE_API_BASE_URL=http://172.16.60.233:18092/api/v1 bun run dev:mp-weixin
```

如果微信开发者工具没有及时吃到最新产物，重新编译或重新打开项目后再看 `/dist/dev/mp-weixin`。

如果你走的是 Windows 侧开发者工具，优先导入 `E:\projects\football_insight_mini`，这样能避开 `\\wsl.localhost\...` 文件监听不稳定的问题。

如果要让小程序微信登录可用，后端还需要配置：

- `WECHAT_MINI_APP_ID`
- `WECHAT_MINI_APP_SECRET`

## 首页今日话题

首页独立读取 `GET /api/v1/daily-topics?team_id=77680&limit=5`，展示成都蓉城相关新闻的 AI 整理文章，横向最多五张卡片。文章不足五篇时不补造；话题请求失败不影响助力、赛果与榜单面板。404 显示“后端接口待接入”，成功空列表显示“暂无已发布话题”，其它失败提供重试。

Python 负责采集新闻全文、LLM 整理、逐篇四川话 TTS 和上传，Rust 只读取预生成批次。详情展示整理正文及每条原文的标题、发布机构、日期，可复制原文链接。日期来自发布批次，时间按北京时间显示。

每篇文章仅播放自己的 `audio.status=ready`、`dialect=sichuan` 的 HTTPS 音频，按不可变文章 ID 关联，不使用旧积分榜语音配置或示例音频。批次、文章正文或音频链接变化以及资源撤下均停止旧播放；隐藏页面暂停，退出页面销毁。未就绪时显示“四川话准备中”。真机播放仍需在微信后台配置实际音频域名。

接口数据与生成端约定见 [今日话题契约](../docs/daily-topics-contract.md)。

## 相关项目

- 工作区总说明: [football_insight](../README.md)
- 后端: [football_insight_service_backend_rs](../football_insight_service_backend_rs/README.md)
- 抓取器: [sina_csl_scraper](../sina_csl_scraper/README.md)

## 微信小程序发布（mp-weixin 上传）

发布小程序用一条命令（构建 + 向本项目后端登记版本号 + `miniprogram-ci` 上传），流程与 `registration_system_mini` 一致：

```bash
bun run mp:release -- --desc "本次变更说明"           # robot=1：日常开发版
bun run mp:release -- --robot 2 --desc "体验版说明"    # robot=2：体验版专用线
bun run mp:preview -- --desc "预览说明"               # 只传预览版，生成 dist/preview-qrcode.jpg
```

**robot 双轨约定**：`robot=1` 日常开发版本，随便传互不影响；`robot=2` 是体验版专用线——首次用 robot=2 上传后，需在公众平台「版本管理 → 开发版本」对该版本点一次「选为体验版」，之后每次 robot=2 上传的新代码会自动成为体验版内容。robot=1 的上传不会影响体验版。

**发版链路统一使用 Bun 与统一入口 `scripts/mini-release.mjs`**（`bun --no-env-file` 执行）：Git 发布门禁、版本登记、Vite/uni-app 编译、组件与 API 校验、`miniprogram-ci` 上传都由它编排，无需另行安装 Node.js。

流程细节（统一入口 `scripts/mini-release.mjs` + `scripts/mini-release-guards.mjs`）：

1. 清理外层 `bun run` 可能提前注入的 `VITE_*` / `UNI_*` 变量，以 `NODE_ENV=production` 和显式 `.env.production` 重建子进程环境，编译固定 `--mode production`；入口创建临时空 envDir 经 `FOOTBALL_RELEASE_ENV_DIR` 传给 `vite.config.ts`，隔离 `.env.local` / `.env.production.local` 等本地覆盖层，构建结束自动清理。生产 API 必须精确为 `https://match.oryjk.cn/api/v1`，`.env.production` 缺失或值不符即拒绝。
2. 正式 `mp:release` 先执行 Git 发布门禁：当前分支必须为 `main`、工作区干净、`fetch` 成功后本地 `HEAD` 与 `origin/main` 完全一致，任一不满足即拒绝。`build` 不需要 main 门禁，`preview`/`verify` 不触发门禁与版本登记。
3. 边界检查后显式执行一次 `scripts/sync-manifest-version.mjs`（不再走 prebuild 钩子），向本项目后端登记接口 `POST /api/v1/mini-review/allocate` 申请版本号。**登记库（数据库）是唯一权威**：最新版本仍在审核中则**复用**，已出审核则在库内最大版本基础上 `+0.0.1` 并标记审核中；仅当库内无任何记录时才以本地 manifest 为起点。本地 manifest 不参与后续分配（多台构建机结果一致），删库重置后版本号随库回落。若新版本导致 `src/manifest.json` / `src/config/generatedMiniProgramVersion.ts` 变化，发布会在构建与上传前停止；必须提交并 push 版本文件后重新执行。
4. 构建 `dist/build/mp-weixin` 并执行组件注册检查，再对编译后的 `config/apiBase.js` 独立求值 `exports.API_BASE_URL`（不用 require 缓存、沙箱内执行、限定时间），必须为 `https://match.oryjk.cn/api/v1`。任一步失败即中止，不能继续上传旧包。
5. `scripts/mini-ci.mjs` 在加载微信 SDK 前再次执行产物 API 校验；正式 upload 还会再次校验 Git 门禁与 `fetch` 后的 `origin/main`，因此直接调用底层脚本也不能绕过发布门禁。随后以 `manifest.json` 的 `versionName` 上传到微信后台（默认 robot=1，落在「版本管理 → 开发版本」）。`mp:preview` 只校验并上传已有生产产物，不构建、不登记版本。

验证入口：`MINI_REVIEW_SKIP=1 bun run build:mp-weixin` 只做生产构建与检查（不登记）；`bun run verify:mp-release` 只校验现有产物且无远程调用；`bun run test:mp-release` 执行发布链路回归测试。正式 `mp:release` 始终强制 `MINI_REVIEW_SKIP=0`，外层残留的离线变量不会跳过登记。后续不要再绕开统一入口自行拆分发布命令。

前置条件（缺失时脚本会明确报错）：

- 项目根 `private.<appid>.key`：微信公众平台「开发管理 → 开发设置 → 小程序代码上传」下载的私钥（已 git-ignore）；可用 `MINI_CI_PRIVATE_KEY_PATH` 覆盖路径。
- `.env.ci.local` 配置 `MINI_REVIEW_API_KEY`（git-ignore；与后端环境变量 `MINI_REVIEW_API_KEY` 保持一致）。纯本地离线构建可设 `MINI_REVIEW_SKIP=1` 跳过登记。

**默认在 local233（Ubuntu）直接构建并上传**：先提交、推送全部业务改动，确认 main 工作区干净，再从小程序目录执行。local233 已从 peiqian 安全复制上传私钥与 `.env.ci.local`，两者均为 0600 权限且被 Git 忽略；版本登记地址已改为 `https://match.oryjk.cn/api/v1`，不使用 peiqian 的 `127.0.0.1:8092` 地址。开发、构建与微信上传在 local233 完成，无需连接 Mac 或 peiqian。

```bash
cd /home/betalpha/projects/football_insight/football_insight_mini
bun install --frozen-lockfile
bun run mp:release -- --robot 2 --desc "本次变更说明"
```

新机器需重新配置上述私钥与 `.env.ci.local`（`chmod 600`），二者都不能提交到 Git；跨机器复制配置时需将登记地址改为生产域名。默认不要设置 `MINI_PROGRAM_VERSION`，让登记库自动分配；测试上传使用 robot=2，日常开发上传使用 robot=1。peiqian 仅作为 local233 无法发布时的备用构建环境。

上传后的收尾：

- 上传产物是**开发版本**：验证用 `mp:preview` 的二维码或在公众平台设为体验版；对外正式发布需在微信公众平台提审通过后点发布，CLI 不做这一步。
- 微信审核结束后，用 API key 调 `PUT /api/v1/mini-review/review-status`（body：`{"project_code":"football_insight_mini","version":"x.y.z","is_reviewing":false}`）把该版本标记为已过审，下一次构建才会分配新版本号。
- 若本次分配了新版本号，`src/manifest.json` 与 `src/config/generatedMiniProgramVersion.ts` 会被更新，需要单独提交；版本被复用（仍审核中）时无文件变更。
