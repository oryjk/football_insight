# 足球洞察 Web 管理后台

独立纯前端，React 19 + Vite + TypeScript + Bun + Tailwind v4 + shadcn/ui（new-york / neutral）+ lucide。
复用现有 Rust Admin JWT 和管理员账号，首个菜单为「商品管理」，第二个菜单为「设置」。API 随部署路径使用独立的同源前缀，项目没有数据库连接或微信密钥。

## local233 已部署入口

[足球洞察管理后台](http://172.16.60.233/football-admin/)，也可从机器首页导航进入。
使用现有管理员账号和密码；发布不会重置已有账号密码。

- Nginx 直接服务构建后的静态文件，API `/football-admin/api/v1/admin/**` 转到本机 Docker 后端 `18092` 的 `/api/v1/admin/**`。
- 全局 `/api/v1/` 保留 Betalpha Mesh 路由，未改为足球后端。
- 当前后端通过既有 SSH 隧道连接生产 `football_data`；保存、停用和删除商品会立即影响真实后续购买。
- 前后端发布与 peiqian 生产更新记录见 `../docs/admin-web-validation.md`。

重新发布前端：在 monorepo 根目录执行 `bash deploy/deploy-admin-web-local233.sh`。
脚本执行 Bun 测试和子路径构建，发布到共享静态目录的独立版本，并原子切换 `football-admin` 链接。
Nginx 路由参考 `deploy/nginx/local233-admin-web.conf`；现网配置归属主机级网关目录。

## 首次初始化

打开后台先请求管理员初始化状态。空数据库展示初始化页：账号固定为 `superadmin`，管理员自行填写并确认 8–128 字符的密码；没有预设初始密码。成功后返回登录页，手动登录；登录后可在「设置」修改账号或密码。已有管理员（包括停用账号）不显示初始化页，不覆盖现有凭据。状态检查失败提供重试；并发初始化返回 409 时重新检查并进入正常登录。

## 本地启动

```bash
bun install
bun run dev
```

默认开发端口 `5176`，代理 `/api` 到 `http://127.0.0.1:18093`。
若联调隔离测试后端（例如 `18094`）：

```bash
ADMIN_API_PROXY_TARGET=http://127.0.0.1:18094 bun run dev
```

也可复制 `.env.example` 为未跟踪的 `.env.local`。后端代码变更后必须先重启本地后端再验证；写操作只使用隔离测试数据库，不使用生产商品做删除测试。管理员账号由后端提供，本项目不创建账号或初始化商品。

## 测试与构建

```bash
bun test
bun run build
bun run preview
```

构建包含 TypeScript 检查，输出 `dist/`，预览端口 `4176`。静态资源部署前缀可以通过 `ADMIN_WEB_BASE_PATH` 配置，例如：

```bash
ADMIN_WEB_BASE_PATH=/football/admin/ bun run build
```

根路径开发使用 `/api/v1/admin`；子路径构建 `/football-admin/` 使用 `/football-admin/api/v1/admin`。
部署服务器需移除足球入口前缀再向 Rust Admin API 转发。已部署到 local233 内网 Nginx，未修改 jd/peiqian 的生产入口。

## 已实现

- 首次管理员初始化：先查询状态，固定 `superadmin`，本人设置密码，无预设密码、密码持久化或自动登录。
- 商品写入期间锁定导航和退出，避免切换设置导致商品请求取消；列表查询不锁定导航。
- 设置页修改当前管理员账号或密码：当前密码必填，新密码选填并要求确认；成功后撤销全部 Web / Android 会话、清除当前 token 并返回登录页，预填更新后的账号，不自动登录。输入密码只保留在页面状态和真实请求中，不写本地存储。
- 管理员登录、`me` 验证、真实 `logout`；401 返回登录页；按请求 token 比对失效事件，旧请求不能清除新登录。
- 默认深蓝黑 / 青绿主题，浅色切换并使用独立 localStorage key 持久化；存储不可用时当前页面仍可工作。
- Sidebar / Header / Main Content Card 三个主 Surface，窄屏导航抽屉和表格横向滚动。
- 服务端关键词、会员等级、启用状态筛选及分页；详情、新建、编辑（允许等级和金额改键）、启停、删除。
- 新建默认停用；购买范围明确选择「公开购买」或「指定账号」。指定范围至少选择一个账号，支持所有合法商品等级；V3 仅能指定账号。危险操作显示名称、业务键、影响和确认。
- 每个商品提供「购买记录」：显示服务端全量购买人数、支付订单数、退款订单数、原支付金额汇总及分页订单。列表包含真实昵称/头像、北京时间的支付/到期时间、实付金额、支付或退款状态和订单号；头像缺失或加载失败显示兜底图标。
- 到期来源分别显示真实快照、快照无到期时间「长期有效」、历史一年规则估算「历史计算」、缺少记录「未记录」，不会以当前账号会员到期时间替代历史订单。格式异常的日期单独提示「日期异常」。
- 指定购买账号使用真实用户搜索（昵称/账号标识、分页），提供已选标签与移除、手工 UUID 兜底。编辑保留原 `allowed_user_ids`；账号已删除或查询失败不丢 UUID，显示信息缺失不阻止保存。已有账号详情最多 4 个并发请求、复用本次已查询账号，关闭弹窗/切换范围会取消请求。
- 会员有效期以月为单位，默认为 12，范围 1–1200；编辑、详情和启停均保留该配置。
- 表格操作列固定在右侧，窄屏和横向滚动时仍可查看、编辑、启停、删除。
- 道具金额显示元，以十进制字符串 + BigInt 精确换为整数分，正数且不超过 2147483647 分。
- 名称和 ID 最多 200 Unicode 字符，ID 不含空白；备注最多 4000 字符；最多 100 个非 nil UUID，去重。
- 请求取消和异步响应检查、提交防重复与禁关闭、失败保留表单、404 可返回刷新、204 正确处理、安全错误显示。

商品与购买记录响应包含不可变的 `catalog_product_id`。弹窗固定打开时的商品身份，校验每次响应，发现删除后同键重建或旧接口缺少身份信息时停止展示并提示刷新；不会把替代商品的订单显示在旧商品名下。该标识只用于读响应校验，不进入商品 POST/PUT 请求。

商品购买记录消费 `GET /api/v1/admin/products/{encoded_key}/purchases?page=1&page_size=20`；汇总来自 API 的 `summary`，不会根据当前分页推算，成功支付统计包含后来已退款的订单，失败/待支付不在该查询中。累计金额是原支付金额，未扣退款。

账号搜索消费 `GET /api/v1/admin/users?query=&page=1&page_size=20`，选中已有 UUID 的显示信息通过 `GET /users/{encoded_id}` 回填。查询使用真实 API，不内置任何购买账号名单。显式选择公开范围才提交空名单；指定范围留空会提示错误。

这里维护的是微信道具映射；道具金额须匹配微信已发布道具和业务实付金额，不修改公开会员标价或等级权益。停用/删除影响后续新购买，历史订单使用原快照。商品列表来自真实 API，没有模拟数据或本地保存替代接口。

## API 契约

`POST /api/v1/admin/auth/login`、`GET /auth/me`、`POST /auth/logout`。
初始化通过 `GET /api/v1/admin/auth/setup` 返回 `{needs_setup,username:"superadmin"}`；`POST /auth/setup` 只发送 `{password}`，成功 204，并发或已初始化 409。

当前管理员凭据更新使用 `PUT /api/v1/admin/auth/credentials`，body 为 `{username,current_password,new_password?}`；新密码省略时保留原密码，成功返回 204 并撤销全部管理员会话。账号保存时修剪并转小写，范围为 3–64 位 ASCII 字母、数字及 `._-`；新密码为 8–128 个 Unicode 字符，不修剪空格。

商品使用 `/api/v1/admin/products`，所有请求携带 Bearer Admin JWT。
列表 query：`query`、`membership_tier`、`enabled`、`page`、`page_size`。
GET 列表返回 `{items,total,page,page_size}`；GET 详情、POST、PUT 返回 Product；DELETE 返回 204。
PUT URL 编码原业务键，响应使用新的 `membership:<tier>@<price_cents>` 键。

Product 与写入 DTO 见 `src/api.ts`；前后端绑定契约见根目录 `docs/superpowers/plans/2026-10-13-admin-web-products.md`。

## 目录与规范

`src/apiClient.ts` 管理网络和安全错误，`authSession.ts` 管理会话，`theme.ts` 管理主题；
`layout/` 为壳层，`features/products/` 为商品页面、表单和纯校验模型；`features/settings/` 为当前管理员设置和凭据校验/会话收尾模型，`features/setup/` 为首次初始化。
基础组件沿用 betalpha-admin 所需 shadcn 组件，主题与控件层沿用其 Token 体系；业务页面使用语义类名，样式分层见 `src/styles/README.md`。

自动测试使用真实本地 HTTP server 验证客户端行为，并覆盖表单业务边界及会话失效；浏览器联调与视觉验收由主任务统一执行和记录。

## 会员来源徽标

购买记录弹窗使用 shadcn Checkbox 提供「支付购买 / 推荐升级 / 未知来源」多选筛选，默认全选。切换来源重置第一页，全部取消提示选择来源；总数和分页来自服务端筛选。原支付订单及汇总通过 shadcn Collapsible 展开查看，仅在勾选支付购买时提供入口。昵称旁通过图标、文字及颜色显示「购买获得」「推荐升级」「未知来源」。缺失或不认识的来源值统一显示未知，不从等级、人数、期限外观推断。

支付订单使用实际历史订单来源，会员列表通过 `GET /api/v1/admin/products/{encoded_key}/members?page=1&page_size=20&origins=purchase,referral,unknown` 展示当前数据库等级对应的活跃账号（含已到期），提供当前到期时间与状态、独立分页及商品 UUID 校验。该列表按等级展示，不能证明账号购买了当前 SKU；来源依据当前权益授予，购买人数与金额仍仅来自成功支付订单。

未来推荐升级、购买结算会在同一事务记录来源；退款同时校验权益与来源并恢复原来源。人工调整或历史无法证实来源的账号显示未知。历史购买来源仅在原订单权益快照或普通年会员到期时间与当前权益精确一致时回填；不将无订单账号自动归为推荐升级。
