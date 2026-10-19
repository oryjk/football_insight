# 战术板分享、点赞与球队列表

日期：2026-10-19。延续 `2026-10-19-lineup-board-design.md` 的阶段三，并按本次用户要求加入点赞与列表排序，取代原文将点赞/排名列为范围外的约定。

## 产品规则

- 用户已明确选择：保存满 11 人后自动公开展示。未满 11 人的阵容继续仅供本人编辑；再次保存为不完整时从公开列表/分享读取中移除，稳定分享码与已有点赞保留。编辑页明确提示完整阵容保存后会公开。
- 每人每队一套阵容，更新保持分享码、创建时间与点赞，修改时间正常更新。
- 球队列表可匿名浏览；完整阵容的只读页支持分享给好友与生成含小程序码的分享图，并有“我也排一套”和“球队战术板”入口。
- 点赞需要登录且账号 active，同人同阵容最多一赞，可取消；允许为自己的阵容点赞。禁用作者的阵容不公开，禁用用户的赞不计数。
- 列表默认按创建时间倒序（更新不会重新顶到最新），另一排序按点赞数倒序；点赞数相同时按创建时间倒序，再以内部行 ID 稳定排序。页面显示点赞榜的名次，支持分页加载。
- 本轮只实现代码与本地验证、提交推送；数据库迁移、生产部署和微信上传另按用户指令执行。

## API 契约

现有 GET/PUT `/api/v1/lineups/me/{team_id}` 与名单接口不变。

公共对象：`author={display_name:string,avatar_url:string|null}`，不含用户 ID、账号、手机号。`team` 沿用 `{team_id,team_name,avatar_url}`。

1. `GET /api/v1/lineups/teams/{team_id}?sort=latest|likes&page=1&page_size=20`
   - 匿名可读，可带 JWT 获取本人的点赞状态。
   - 返回 `{team,items,total,page,page_size}`，每项 `{share_code,formation,author,created_at,updated_at,likes_count,liked_by_me}`。
   - `page>=1`，`page_size=1..50`；非法排序/分页返回 400，未知球队 404。
2. `GET /api/v1/lineups/shared/{share_code}`
   - 匿名可读，可带 JWT。返回 `{share_code,team,author,formation,slots,players,created_at,updated_at,likes_count,liked_by_me}`。
   - `players` 为数组，每项沿用名单球员字段并加入 `former:boolean`。`slots` 不变。
   - 不完整、不存在或作者禁用均返回 404；分享码限定 8 位 base62。
3. `PUT /api/v1/lineups/shared/{share_code}/like` 与 `DELETE` 同路径
   - JWT 与 active 用户必需；幂等设置/取消，不采用 toggle。返回 `{likes_count,liked_by_me}`。
   - 公开条件与读接口一致；并发重复请求不能重复计数。数据库事务协调目标阵容检查、点赞变更和结果读取。
4. `GET /api/v1/lineups/shared/{share_code}/wxacode`
   - JWT 与 active 用户必需；先确认阵容可公开，返回 `{image_base64,mime_type}`。
   - scene 为 `s=share&c=lineup&l=<share_code>`，page 为 `pages/lineup/view`，env_version 为 `release`。

可选 JWT：缺少表示游客；提供但无效/账号禁用返回 401，前端可清理登录态后匿名重载。所有个性化响应 no-store。错误由 typed application/domain error 映射 HTTP 状态，不按错误文案分支。

## 数据与架构

- 新增 `f_i_lineup_likes(lineup_id UUID REFERENCES f_i_lineups(id) ON DELETE CASCADE,user_id UUID REFERENCES f_i_users(id) ON DELETE CASCADE,created_at TIMESTAMPTZ)`，主键 `(lineup_id,user_id)`，必要索引支持用户外键与球队创建排序。
- 公开条件与排序由数据库执行，分页前筛选，不使用前端过滤或缓存虚假计数。来源仍为已有 `f_i_lineups`，不新增第二套阵容存储。
- lineup 的 domain/application/ports 不引入 SQLx、Axum、serde、Web DTO 或具体 adapter；查询、授权与分享流程由 application 负责，SQL/事务与微信请求在出站 adapter，Web 只鉴权提取/DTO/错误映射。
- lineup 定义 `LineupCodePort::image(code)`，返回自身协议无关 `LineupCodeImage {image_base64,mime_type}`，错误为 `LineupPublicError`。公共服务使用已有 `LineupUserAccess` 检查 active。
- share_code 的微信 adapter 提取可传 scene/page 的通用生成能力，保留旧 `WechatTicketWatchCode` API 与票务二维码 24h 成功/30s 失败缓存、不强刷 stable token 的行为；新增 `WechatLineupCode::new(app_id,secret)` 实现 lineup 消费方 port。多码缓存有界并合并同码在途请求。
- 前端 API 仅原子 HTTP，业务编排在页面 composable，展示组件只使用 props/emits；共享 helpers/components 放 lineup 目录，同一页面域不得跨页导入页面入口。新增 view/list 均注册在 lineup 分包，运行时组件直接从 `.vue` 导入，不新增依赖边界例外。

## 小程序流程

- 编辑页“分享”先验证满员；未登录沿用草稿与登录回跳，有改动先保存，保存成功后使用服务端分享码。只在完整已保存时提供卡片目标，避免发送自己的编辑页。
- 新 `pages/lineup/view?code=` 展示作者、球队、阵型、只读球场、创建/更新时间、点赞与分享；`scene` 解码兼容二维码中的 `l`。
- 新 `pages/lineup/list?teamId=` 展示球队选择、最新/点赞排序、作者/阵型/时间/点赞卡片及下一页；点赞按钮不穿透卡片导航。详情返回列表重新同步点赞状态。切队/切排序/卸载后旧响应不能污染当前结果，分页与点赞防重复提交。
- 分享卡片 path 带 `src=share&cid=lineup`；生成 750×1200 分享图，头像失败用号码占位，小程序码失败不导出无码图，沿用 canvas 原语与相册权限流程。分享预览/绘制状态按当前阵容隔离，避免在切换后发布旧图。
- 不引入用户自由文本或评论；不扩展独立 H5、Web 后台或 Android。小程序编译 H5 可显示功能提示；扫码的正式版限制保留说明，体验版可验证卡片跳转。

## 验证

- Rust 单测：公开门禁、授权、排序参数、点赞幂等与错误映射、微信 scene/page/缓存回归。
- 专用一次性 PostgreSQL：相同赞并发、取消幂等、作者/点赞者禁用、满员条件、创建排序与点赞排序稳定、用户/球队隔离，禁止使用生产用户写入测试。
- Bun：分享路径/scene、分享满员门禁、分页乱序、点赞失败回滚/防重、绘制纯函数；类型/依赖边界/离线微信构建/组件注册。
- 后端改动后重启 local233 后端再做只读联调；不对真实数据执行迁移。全仓既有 Clippy/架构债须与本轮结果分开记录。
