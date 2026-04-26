# football_insight_mini

Football Insight 的微信小程序前端，采用 `uni-app + Vue 3 + TypeScript`。

当前策略不是替换现有 H5，而是新建一个独立仓库并行推进小程序版本。

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
   - 填写邀请码
   - 选择头像
   - 输入昵称
6. 请求后端 `POST /api/v1/auth/mini-wechat/bind`

当前这条链路的产品约束是：

- 不要求补用户名
- 不要求补手机号
- 不要求设置密码
- 首次完成绑定后，后续可以直接用微信进入

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
cd /Users/carlwang/football_insight/football_insight_mini
bun install
bun run dev:mp-weixin
```

然后用微信开发者工具打开：

- `/Users/carlwang/football_insight/football_insight_mini/dist/dev/mp-weixin`

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

- [小程序排查记录](/Users/carlwang/football_insight/football_insight_mini/docs/mini-troubleshooting.md)

## 接口说明

小程序不走 Vite 代理，请求地址规则是：

- 开发环境默认请求本地后端：`http://127.0.0.1:8092/api/v1`
- 生产环境默认请求线上 API：`https://match.oryjk.cn/api/v1`

如需切换，可通过环境变量覆盖：

- `VITE_API_BASE_URL`
- `VITE_MINI_PROGRAM_VERSION`
- `VITE_MINI_PROGRAM_APP_ID`（可选）

小程序启动页面会用 `VITE_MINI_PROGRAM_VERSION` 请求后端 `GET /api/v1/system_config?version={version}`。如果后端返回 `is_under_review=true` 且 `matched=true`，小程序会进入审核模式：不加载会员、支付、用户会员等级、回流看板会员权益等相关信息。

当前项目已经内置：

- [`.env.development`](/Users/carlwang/football_insight/football_insight_mini/.env.development)
- [`.env.production`](/Users/carlwang/football_insight/football_insight_mini/.env.production)

也就是说：

- `bun run dev:mp-weixin` 默认连本地 `8092`
- `bun run build:mp-weixin` 默认连线上 `match.oryjk.cn`

如果微信开发者工具没有及时吃到最新产物，重新编译或重新打开项目后再看 `/dist/dev/mp-weixin`。

如果你走的是 Windows 侧开发者工具，优先导入 `E:\projects\football_insight_mini`，这样能避开 `\\wsl.localhost\...` 文件监听不稳定的问题。

如果要让小程序微信登录可用，后端还需要配置：

- `WECHAT_MINI_APP_ID`
- `WECHAT_MINI_APP_SECRET`

## 相关项目

- 工作区总说明: [football_insight](/Users/carlwang/football_insight/README.md)
- 后端: [football_insight_service_backend_rs](/Users/carlwang/football_insight/football_insight_service_backend_rs/README.md)
- 抓取器: [sina_csl_scraper](/Users/carlwang/football_insight/sina_csl_scraper/README.md)
