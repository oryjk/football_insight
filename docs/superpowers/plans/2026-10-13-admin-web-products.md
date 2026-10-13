# Football Insight Web 商品管理实施计划

> **For agentic workers:** Use the parallel-agents skill for independently owned backend/frontend tasks, then integrate and review. Steps use checkbox syntax for tracking.

**Goal:** 独立 Web 后台通过真实 Admin API 管理现有会员微信道具。
**Architecture:** React 前端复用 Admin JWT；现有 Rust admin 域新增 repository 和带审计的商品 CRUD。前后端按以下固定契约独立实现，由主代理联调。
**Tech Stack:** React 19、Vite、TypeScript、Bun、Tailwind v4、shadcn/ui；Rust 2024、Axum 0.8、SQLx、PostgreSQL。
**Spec:** `docs/superpowers/specs/2026-10-13-admin-web-products-design.md`

## Global Constraints

- 保留全部既有未提交修改；不提交环境文件、密钥或真实用户数据。
- 项目 `football_insight_admin_web` 为纯前端；所有 SQL 在现有 Rust persistence adapter。
- UI 遵循参考项目 Token、shadcn、语义类名、文件命名和深浅主题规范。
- V3 必须指定购买账号；V6–V9 支持公开或指定账号；价格单位为整数分。
- 无模拟保存，无生产写操作验证，不部署生产或修改 Nginx。
- 后端修改后重启本地进程再进行接口联调。

## Shared API Contract

Product JSON: `product_key:string, membership_tier:string, name:string, product_id:string,
price_cents:number, membership_duration_months:number, enabled:boolean, remark:string, allowed_user_ids:string[], created_at:string,
updated_at:string`。

Create/PUT body: `membership_tier, name, product_id, price_cents, membership_duration_months, enabled, remark, allowed_user_ids`。
有效期默认 12 月，限定 1–1200 整数；兼容旧请求省略时使用 12。
业务键由服务端生成 `membership:<tier>@<price_cents>`；PUT URL 使用旧键，响应返回新键。

- GET `/api/v1/admin/products?query=&membership_tier=&enabled=&page=1&page_size=20`
  返回 `{items:Product[],total:number,page:number,page_size:number}`；默认仅会员映射。
- GET `/api/v1/admin/products/{encoded_key}` 返回 Product。
- POST `/api/v1/admin/products`、PUT `/api/v1/admin/products/{encoded_key}` 返回 Product。
- DELETE `/api/v1/admin/products/{encoded_key}` 返回 204。
- 请求统一 Bearer Admin JWT；错误为 HTTP 400/401/403/404/409/500 和安全的文字正文。
- 登录使用现有 POST `/api/v1/admin/auth/login`，响应 `{access_token,expires_at,admin:{id,username,display_name,role}}`；GET me 返回 admin，POST logout 返回 204。

## Review Focus

- 价格变化导致业务键变化：原记录原子更新，重复新键返回 409，不覆盖他人记录。
- V3 缺少 UUID 或 UUID 格式错误：前后端拒绝且保留输入。
- token 过期、快速筛选或退出：返回登录态，过期响应不能覆盖当前数据。
- 数据库商品被其他管理员删除：详情/编辑/删除返回 404，并可刷新恢复。
- 网络失败和重复点击：不报假成功，提交期间不能重复写入或关闭丢失输入。

### Task 1: Rust 商品 API

**Files:** admin/domain/admin_product.rs、ports/admin_product_repository.rs、application/admin_product_service.rs、adapters/persistence/postgres_admin_product_repository.rs、adapters/web/product_{dto,handlers,routes}.rs；修改各 mod.rs、admin/bootstrap.rs、AdminPermission 和 role 授权；新增 tests/admin_products_postgres.rs。
**Interfaces:** 消费现有 AdminAuthService、PgPool 和审计表，提供 Shared API Contract 的全部端点。

- [x] 先写商品领域校验和角色授权失败测试并观察失败。
- [x] 实现 typed error、规范化输入、生成业务键、V3 限制、分页和检索。
- [x] 实现端口和 SQLx repository；每次变更与 before/after 审计同事务。
- [x] 实现 DTO、JWT 授权 handler 和路由装配；内部错误只记日志，不向前端暴露 SQL。
- [x] 在隔离数据库测试创建、编辑改键、重复键、筛选、删除、不存在、审计回滚及权限。
- [x] 运行 cargo test、cargo check、改动 Rust 文件格式检查；报告准确结果，不提交已有改动。

### Task 2: React 管理前端

**Files:** 新建 football_insight_admin_web 项目配置、README、AGENTS、src API/auth/theme、layout、features/products、styles 与 shadcn UI。
**Interfaces:** 严格消费 Shared API Contract，不改后端或根文档。

- [x] 先写 productsModel 测试：金额精确换分、限制 UUID、默认停用、key 编码；观察失败。
- [x] 复用参考项目所需 shadcn 组件、Token 和控件样式；建立 Vite/Bun 项目与可配置代理。
- [x] 实现真实登录、me 会话验证、logout、401 处理和主题持久化。
- [x] 实现 Sidebar/Header/Main Card 布局与窄屏抽屉。
- [x] 实现分页、查询、筛选、查看、新建、编辑、启停和危险删除确认；失败保留输入。
- [x] 验证 API 错误、安全消息、204 响应、快速筛选取消与过期会话；运行 bun test 和 bun run build。

### Task 3: 联调、审查和交付

**Files:** 根 README/FEATURES/CHANGELOG、部署文档；必要接口一致性修正；浏览器验证记录。

- [x] 核对前后端接口、阅读全部新增代码并独立审查。
- [x] 重启含新代码的本地后端，配置隔离测试库和测试管理员，不操作生产商品。
- [x] 实际浏览器验证登录、CRUD、筛选、错误、主题、窄屏；保存截图。
- [x] 完成后端边界检查与 clippy，区分既有失败与新增失败。
- [x] 更新项目记忆及启动说明；最终报告实际测试与本地/生产重启状态。

## 实施期兼容调整

实施期间另一个工作区任务新增 `20261013230000_add_membership_duration.sql`。
管理端接入该字段读写、有效期校验及订单快照兼容，复用已有迁移，不改其支付实现。
表格操作列固定在右侧，以便横向滚动时仍可执行管理操作。
