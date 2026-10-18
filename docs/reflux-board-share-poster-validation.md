# 回流看板分享图验证记录

日期：2026-10-19。

## 实现与内容边界

- 当前比赛与历史复盘页提供「生成分享图」入口。空数据、缺少/无效 `sale_start_at`、审核态、原有权益锁定状态不显示入口；加载中禁止生成。历史选择尚未发布成功时不使用上一场数据生成。
- 750 × 1080 海报只显示基础库存格子的公开字段：球队文字、比赛时间、累计回流张数、区域数、回流较多的五个区域及票价。没有队徽、用户信息、关注/兴趣数据、分区精确回流时间、会员最近回流速览或最佳盯盘时段。
- 依据现有 `TicketInventoryPanel.vue`：基础区域库存不按会员档位隐藏；V6–V8 的最近回流速览是另一套权益数据，不进入海报模型。`buildTicketPosterContent` 明确白名单构造模型，`viewer` 的任何会员档位不会改变输出。
- 计数沿用已有页面的「张」单位。核实上游 `ticket-monitor-axum/src/region/adapter/persistence/mysql_repository.rs` 的 `occurrences = SUM(inventory)`；云南玉昆 `region/application/use_case_impl.rs` 同样累计 `record.inventory`。这是累计观察到的回流张数，不能标为独立回流事件次数，也不代表当前可购买余票或去重票数。
- 页面原有数据请求均以 `requireInventorySince(sale_start_at)`（开售后 10 分钟）起算，不新增数据读取接口。纯函数再次拒绝缺失起算时间，并排除最后观察时间早于起算点的行；累计数来自已按起算时间过滤的页面 API 结果。
- 当前图写明秒级「数据读取截止」及北京时间，使用 `currentBoardLoadedAtMs`（本次读取完成时间，不冒充服务端最后一次采集时间），另标「实时库存变化很快，请以小程序为准」。历史图标明历史记录不代表当前余票。
- 生成后页内预览、点击系统图片预览及保存到相册。未登录用户生成时引导登录。已完成的图片在系统原生预览可能触发页面隐藏时保留，在途生成在隐藏后作废；更换比赛/页签/球队/账号、进入审核态及卸载会清理旧图。
- 所有 canvas 绘制串行执行；异步取码、绘制、导出均核对生成版本。以球队、页签、比赛、账号、权限与数据合法性决定失效，同场轮询不取消在途任务或清理已生成图；生成时冻结数据快照，保留当时的秒级截止时间。保存相册权限查询期间若比赛/页签已改变，也不会继续保存旧图。
- 相册拒绝授权逻辑抽至 `utils/canvasPoster.ts`，赛季海报保留原导出名称并复用。长文本的 Unicode 安全折行抽至 `utils/shareCardText.ts`，话题分享图保持原导出。

## 小程序码与失败策略

- `POST /api/v1/share-codes/ticket-watch`，Bearer JWT 鉴权；无请求体，返回 `{ image_base64, mime_type }`。
- 固定微信参数：`scene=s=share&c=ticket_poster`、`page=pages/ticket-watch/index`、`check_path=false`、`env_version=release`、`width=280`。场景不含用户或比赛信息，扫码进入看板默认视图。
- 非强刷 `stable_token`，成功缓存 24 小时、失败缓存 30 秒，并发请求合并；缺凭据及微信失败返回 503。
- 取码失败时**阻止生成**，按钮解除加载并显示「二维码暂时获取失败，请重试」；不会输出缺少入口的图片。绘制/导出失败另显示重试提示。
- 注册来源沿用 `App.vue` 的 scene 捕获：`src=share`、`cid=ticket_poster`。普通对话/朋友圈分享仍用 `ticket_current` / `ticket_history`。
- 未添加「生成分享图」行为计数：现有 `reportPageActivity` 仅支持预定义页面键；按设计不为此扩大后端范围。

## 自动化与本地检查

- 新增前端关键测试及既有赛季海报回归：`bun test src/pages/ticket-watch/sharePoster.test.ts src/pages/ticket-watch/ticketPosterFlow.test.ts src/utils/canvasPoster.test.ts src/pages/season-recap/poster.test.ts`，33 通过、0 失败。覆盖当前/历史图在 guest、V1–V9 下输出一致、字段白名单、开售时间缺失及起算前数据、秒级截止、旧取码/旧导出竞态、实际页面选择/页签/审核变化、当前/历史同场刷新保持在途与已完成快照、系统预览返回刷新、重复点击、失败反馈、相册授权查询期间失效。
- 后端：`cargo test -- --skip test_three_rounds`，552 通过、73 ignored、1 filtered；该过滤项读取 `.env` 调用真实远程 AI，未执行。8 项新增测试覆盖固定请求字段、非强刷 token、未配置凭据、成功/失败缓存及过期、并发合并、鉴权。赛季海报原有测试通过。
- `cargo check`、限定改动文件的 `rustfmt --check`、`git diff --check` 通过。
- `cargo clippy -- -D warnings` 被既有模块的 17 个 lint 阻塞（`collapsible_if`、`needless_lifetimes`、`too_many_arguments`、`new_without_default`、`should_implement_trait`），新模块无命中。全仓架构扫描仍有两类既有问题；对新模块内层与 Web 层执行同等边界扫描通过。
- `./deploy/local-test/up.sh` 已重新编译并重启本地 18092 后端，systemd active、health 为 `status=ok`；对新路由无 JWT / invalid JWT 请求均返回 401；重启后 `cargo test share_code` 8 通过。没有调用真实微信生成接口或写生产数据库。
- 最终小程序全套 `bun test`：968 通过、0 失败，90 个文件，13.76 秒；`bun run type-check`、`bun run check:boundaries` 通过。
- 最终 `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` 与 `bun run build:h5` 均通过；小程序构建尾部 `verify:mp-components` 检查确认全部自定义组件已注册，产物 API 校验通过。仅本地离线构建，版本保持 1.0.99，未登记新审核版本、未上传。
- 独立复审确认同场轮询保留图与异场切换失效逻辑通过，无新增阻塞。Chromium 真实 canvas 的合成比赛数据与**占位二维码**视觉检查覆盖长队名/emoji、长区域名、五区域、秒级时间，未见画布溢出或文字/二维码重叠；不算微信真机取码或扫码通过。

## 仍需真机与上线验证

1. 用微信真机分别生成当前/历史图，检查长队名、区域、秒级截止及图形导出；快速切换比赛、页签、球队后确认旧图不会出现。
2. 验证系统预览长按发送，以及相册首次授权、拒绝后设置引导、保存成功和普通失败反馈。
3. 部署后端后发布小程序正式版，最后扫描**真实**小程序码：进入回流看板默认视图，以测试新用户验证注册来源 `share/ticket_poster`。`check_path=false` 只保证取码阶段，不能代替正式版页面可用性。
4. 视觉评估中如使用占位二维码，仅用于布局检查，不算真实扫码通过；微信开发者工具不能替代上述真机步骤。

当前状态：**本地后端已重启；生产后端未部署、未重启；小程序未上传、未发布。** 无数据库迁移。
