# football_insight_admin_web

先阅读上级 AGENTS.md、README.md、`src/styles/README.md` 和根目录商品 Web 设计/计划。
本项目只负责纯前端，不直连数据库，不添加服务端、SQL、微信密钥或假数据。

- 使用 Bun；修改后运行 `bun test` 和 `bun run build`。API、模型、会话修改先编写能失败的业务测试。
- React 19 + Vite + TypeScript + Tailwind v4；基础控件统一复用 `src/components/ui` shadcn。
- 与 betalpha-admin 保持 deep blue / teal 深浅主题、36px 密集控件、48px 登录主操作、6/8/10px 语义圆角。
- 颜色、字号、行高、间距、圆角引用 Token；不引用停用的 font-size-2xs / font-size-sm。新布局常量先加入 foundation。
- 页面用语义类名，Tailwind 工具类只放基础组件；Button 外观统一 data-slot/data-variant 协议。
- 保持 Sidebar、Header、Main Content Card 三 Surface；业务筛选/操作/表格均留在 Main Card。
- React 组件 PascalCase，配套纯逻辑 camelCase `*Model.ts`；文件基名不得仅大小写不同。
- 请求必须真实消费共享 API 契约。PUT 用原键，保存后使用服务端返回的新键；金额输入用字符串精确换分。
- 打开后台先获取真实初始化状态，不提供默认初始密码；空库管理员自行设置 `superadmin` 的密码，有管理员（含停用）则保留现有账号。检查失败只重试，初始化 409 重新检查，成功不自动登录。
- 商品和凭据写入期间均锁定导航/退出，列表 GET 不上报写入锁；页面卸载释放锁并取消请求。
- 设置页只修改当前管理员；成功 204 必须清除当前 token 并要求重新登录，不自动登录，不持久化密码；新密码保留空格，长度按 Unicode 字符计数。凭据写入期间锁定导航和退出，错误保留输入。
- 购买记录固定打开时 Product.catalog_product_id，每个返回页面先校验同名字段；缺失/变化均拒绝展示并允许关闭刷新，避免同键删除重建后串单。catalog_product_id 不提交到商品写入 DTO。
- 商品购买记录汇总读取 API 全量 summary，不由当前分页推导；金额是原支付金额，包含退款的成功支付历史。支付/到期时间按北京时间显示，历史估算标明来源；快照 expires_at=null 为长期有效，unrecorded 为未记录。头像只接收 http/https、加载失败兜底；只读查询不占写入锁，关闭/翻页/退出取消请求。
- 购买范围明确选择公开/指定账号；指定账号为空不能退化为公开。搜索与显示信息来自真实 users API，已有 UUID 无论是否可查询均保留，元数据请求最多 4 并发、可取消，不因缺失显示信息阻止保存。不写死账号或套餐专用范围逻辑。
- V3 不能公开；UUID/长度/金额边界与后端同步。新建默认停用。
- 控制 401、AbortError、快速筛选、卸载/退出的迟到响应；失败保留输入，写入防重复，写入中不能关闭丢失输入。
- 不提交 .env、token、密钥和真实用户数据，不自行部署生产。写操作联调只能用隔离测试数据库。
- 新 API 的后端变更后必须先重启本地后端，最终报告区分本地重启和生产未部署/未重启。

参考 UI 文档：`/home/betalpha/projects/betalpha-admin/docs/ui-design-guidelines.md`。
