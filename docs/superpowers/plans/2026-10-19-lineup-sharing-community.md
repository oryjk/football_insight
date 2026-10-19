# Lineup sharing and community implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** 完成战术板分享、点赞和按球队浏览/排序，保留已确认的重置弹窗修改。

**Architecture:** 保持 lineup 六边形模块；应用消费自身 ports，Web DTO 与 SQL/微信 adapter 分离。小程序页面 composable 管理异步状态、展示组件只发操作意图，view/list 属于既有 lineup 分包。

**Tech Stack:** Rust 2024/Axum/SQLx/PostgreSQL；uni-app/Vue3/TypeScript/Bun。

**Spec:** `docs/superpowers/specs/2026-10-19-lineup-sharing-community-design.md`

## Global constraints

- 保存满 11 人后自动公开；未满员不公开；分享码与创建时间在覆盖保存时保持不变。
- 同人同阵容最多一赞，可取消；active 账号才能点赞；禁用作者与点赞者按 spec 隐藏/不计数。
- HTTP DTO、serde、SQLx/Axum 和具体 adapter 不进入领域/应用/port；前端不得从展示组件请求 API。
- 前端只用 bun、后端用 cargo；不改 Android、不提交密钥、不手改版本号。
- 用户已追加授权完成后的生产迁移、部署与 robot 2 上传；默认提交推送当前 main（项目既有约定），本地后端修改后重启验证。

## Review focus

1. 完整阵容改回不完整，旧链接、列表和点赞写入都应不可见。
2. 并发重复点赞/取消不重复计数；禁用用户不能继续通过旧 JWT 写入。
3. 同赞数、同创建时间、换队/换排序与翻页，结果稳定且旧响应不串入。
4. 未登录分享、保存失败或编辑中切队，不发送错误阵容/旧分享图。
5. 二维码缓存隔离 scene/page，旧票务分享二维码契约保持不变。

## Task 1: 公共阵容、点赞、排序与迁移

**Owner:** backend implementer. **Files:** `football_insight_service_backend_rs/src/lineup/**`、`src/app.rs`、新 migration、`tests/lineup_postgres.rs` 或独立 public 测试。

- [x] 按 spec 写公开条件、排序分页、点赞与授权失败测试，先验证失败。
- [x] 实现 domain read model/error、consumer ports、public application service、Postgres 查询/事务、独立 Web DTO/路由；既有我的阵容接口保持兼容。
- [x] 定义 `LineupCodePort::image(&str)->Result<LineupCodeImage,LineupPublicError>`；bootstrap 使用 Task 2 的 `WechatLineupCode::new(app_id,secret)`，`build_lineup_routes` 接收 config，app.rs 更新装配。
- [x] 新迁移创建点赞表与索引，不在真实库运行；只对触碰的 Rust 文件格式化，不全仓 cargo fmt。
- [x] 运行 lineup 单测与一次性 PG 测试，报告结果与架构边界；由根 agent 统一提交/推送。

## Task 2: 微信二维码通用能力与回归

**Owner:** root. **Files:** `src/share_code/adapters/wechat_code.rs`、同目录 tests，必要时其 domain/ports 只做兼容扩展。

- [x] 阅读现有票务二维码测试，增加 lineup 场景、缓存隔离和旧票务行为回归。
- [x] 提取 scene/page 目标参数，提供 `WechatLineupCode` 实现 Task 1 定义的 port；稳定 token 不强刷，成功缓存 86400 秒、失败缓存 30 秒，多码缓存有界。
- [x] 单测必须验证真实请求负载与缓存行为；不要让 lineup Web 调用 share_code Web。

## Task 3: 小程序分享、详情与球队列表

**Owner:** mini implementer. **Files:** `src/api/lineup.ts`、`src/types/lineup.ts`、`src/utils/lineupLinks*`、`src/pages/lineup/**`、`src/pages.json`，必要的 canvas 共享原语仅增量复用。

- [x] 按 spec 契约增加 DTO 与原子 API，新增 view/list 页面及状态 composable、展示组件；禁止把新流程继续堆进 index.vue。
- [x] 保留重置按钮红底白字与 FiBottomSheet 二次确认；编辑页实现分享门禁、保存后分享码、球队列表入口及公开提示。
- [x] 实现只读详情与点赞/取消、游客登录回跳；球队列表实现最新/点赞排序和分页，旧请求隔离、点赞防重与错误恢复。
- [x] 实现好友卡片与 canvas 分享图（头像占位、二维码失败阻断、相册授权），所有来源/path/scene 按 spec。
- [x] 必要纯函数/关键状态测试、类型/边界检查；根 agent 统一构建和提交，agent 不改版本号、不上传。

## Task 4: 整体验证与复核

- [x] 独立 reviewer 核对任务 1–3 与 spec，修复实质问题。
- [x] 全量 Bun / cargo test、7+新增独立 PG 测试、类型/边界/微信构建与组件注册；记录既有全仓 Clippy 限制。
- [x] 重启 local233 后端，健康与新增未迁移接口只读失败路径验证；生产迁移在独立验证、提交推送后按用户授权执行。
- [ ] 更新 FEATURES/CHANGELOG 与验证文档，提交推送当前分支，刷新项目记忆页面，在 peiqian 应用新迁移、切换 local233 构建的镜像，验证后上传 robot 2，并记录真实发布与真机验证范围。
