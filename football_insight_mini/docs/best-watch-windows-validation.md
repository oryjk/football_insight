# 最佳盯盘时段：小程序验证记录

日期：2026-10-16。范围仅为小程序前端，后端和 monitor 的统计/对账验证见各自报告。术语服从设计第 00 节：座位就是 `block_name` 号码，无额外的座位身份前置条件。

## 实现与边界

- `src/api/ticketWatch.ts` 新增 `GET /ticket-watch/best-windows`，使用已有 request，携带可选 Bearer，`silentNetworkError: true`。
- DTO 在 `src/types/ticketWatch.ts`；`bestWindowsHelpers.ts` 完成柱高、最小非零高度、前三高峰、当前小时、文案和本地权益裁剪。
- `useTicketWatchBestWindows.ts` 只管理这一个接口。成功响应按会话缓存 10 分钟；`show()` 才做过期刷新，账号 revision 同步清理缓存，凭据的同步修改完成后再发起读取。比赛切换、历史选择和轮询不关联此请求。隐藏/销毁作废 owner，旧成功、失败和 finally 均不能覆盖新状态。
- `TicketBestWindowsPanel.vue` 用 `FiPanel` / `FiSectionHeading` 和现有 token。full 展示 24 柱、前三高峰和可单选小时说明；北京时间“现在”只在可见 full 下每分钟刷新，隐藏/切换 preview/卸载清理时钟。preview 只有第一个高峰、样本说明和升级入口，不生成柱状图。loading 使用全局骨架类。
- 页面在当前 tab 的最近回流速览之后、库存之前接入。`insufficient`、`unavailable`、失败和审核状态隐藏。会员权益暂停继续使用页面已有整页锁；纯函数还会把 full 裁成 preview，避免只按等级误解锁。
- 仅新增 `ticket_watch_best_window_lock` 曝光/点击，沿用已有购买跳转和同访问去重。已有两个付费位的 handler/watch、`beginPaywallVisit()`、订阅、比赛 ID、轮询、历史文件和用户水印保持原样。
- 未新增 token、渐变、hex 或依赖边界例外。

## 自动化结果

均在 `football_insight_mini` 执行，shell 命令前缀 `rtk`。

| 命令 | 实际结果 |
| --- | --- |
| `bun test` | 852 pass，0 fail，73 files |
| `bun run type-check` | exit 0 |
| `bun run check:boundaries` | 通过 |
| `bun run build:h5` | Build complete |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | Build complete；组件注册和编译包 API 校验通过 |
| `bun run verify:mp-components` | All mini-program custom components are registered |

新增测试：`bestWindowsHelpers.test.ts` 5 个；`useTicketWatchBestWindows.test.ts` 9 个；`scripts/ticket-best-windows-panel.test.ts` 4 个。组件测试沿用仓库的 SFC 编译 + Vue renderer，未全局 mock 模块。覆盖 0/全 0/少于 3 个高峰、preview 无柱、权益暂停裁剪、静默失败、10 分钟缓存、revision 同步清理、迟到响应、隐藏/销毁、单小时选择和时钟清理。

TDD 实际观察：helpers 初始空实现 4 fail；composable 初始空实现 4 fail；组件初始空实现 4 fail。随后各自转绿。同步抛错导致 pending 不释放的回归先观察到 1 fail，修复后 composable 6 pass。

构建提示仅为现有 uni-app 和 Browserslist 更新提醒。离线构建保留 `versionName=1.0.85` / `versionCode=10085`，未申请版本号、未上传。

## H5 320 窄屏与付费度量

使用本地实际 H5 构建、Playwright 和已安装 Chromium 145，全部 `/api/v1/**` 请求由 fixture 拦截，不写生产数据。

| 情形 | 请求次数 | 柱数 | 页面宽 / 文档宽 | 结果 |
| --- | --- | --- | --- | --- |
| full | 1 | 24 | 320 / 320 | 3 根高峰柱；图宽 283.875；24 柱全在图内；现在标记在视口内 |
| preview | 1 | 0 | 320 / 320 | 仅第一个高峰、锁定文案和升级按钮 |
| V6 且权益暂停，接口仍返回 full | 1 | 0 | 320 / 320 | 面板隐藏，沿用整页会员锁 |
| 审核 | 0 | 0 | 320 / 320 | 面板隐藏 |
| unavailable | 1 | 0 | 320 / 320 | 面板隐藏 |
| HTTP 503 | 1 | 0 | 320 / 320 | 面板隐藏 |

已打开 full/preview 截图目视检查，无横向溢出。full 点击 12 点显示对应样本说明。临时脚本、截图位于 `/tmp/football-best-windows-20261016/`，不提交。

登录 V1 fixture 在当前 → 历史 → 当前切换后，本接口仍只读取 1 次。新 surface 只产生 1 次 impression；点升级后产生 1 次 tap，两者都携带 `match_id: 584`，并跳转 `/pages/membership-purchase/index`。请求确认发送可选 Bearer。其他付费位由原逻辑继续处理。

## 验证限制与发布状态

H5 的真实构建和小程序组件注册已验证；微信开发者工具/真机中的布局尚未验收。没有伪称已完成 MP 真机检查。

本子任务以独立feat(mini)提交推送main；没有部署、迁移、写生产库或上传小程序，未改版本。后端重启与生产状态见主报告，生产部署另按用户指令执行。

## 复审修复：退出/失效时的凭据读取竞态

复审发现 `createAppSession.logoutSession()` 和 `invalidateSession()` 都先发布 revision，再同步清除 token。原 composable 的同步 revision watcher 当场发起 HTTP，request 读取到旧 Bearer；旧账号的 full 结果因此会落入新 revision 的缓存。

用真实 `createAppSession`、默认 `getTicketWatchBestWindows` 和实际 request/存储路径复现，不模拟 revision/凭据的先后顺序。logout 与 invalidate 两个测试分别观察到第二次请求仍有 `Bearer old-full-token`（RED），加上隐藏前取消尚未发起的请求，共 3 个失败。修复保留同步 invalidate/清空，在读取接口前等待一个微任务，使同步 token 修改先完成，并在发请求前及响应发布前均检查 owner、可见身份和销毁状态。

修复后两个路径的第二次请求均没有 Authorization，结果为 preview 且 `hours=[]`；隐藏后失效 owner 不再真正发起读取。composable 9 pass；全量 852 pass / 0 fail。修复后重新执行 type-check、boundaries 和 verify:mp-components，通过结果以本报告表格为准。未修改共享 appSession、旧页面逻辑或任何后端。

## 组件结构与样式对应

|部分|结构与token|现有样式出处|
|---|---|---|
|外壳/标题|FiPanel、FiSectionHeading；space-16、font-22、text-muted、leading-normal|既有FiPanel/FiSectionHeading与回流卡片统一标题|
|24柱|flex等分/min-width:0、space-4间隙、space-100组合图高、radius-sm；divider常规、text-primary高峰|既有回流趋势使用设计token的纯色图表风格|
|小时/现在/详情|font-18、font-26、space-8/24/30、leading-none/normal、text-primary/muted|回流看板的正文/辅助信息token|
|锁定与按钮|space-16/24/72、radius-md/round、border-card、color-page/bg/text-primary、weight-extrabold|TicketRecentRefluxLock的纯色锁定提示与购买入口|
|加载|fi-skeleton-panel/block/title及space-24/100、radius-sm|全局骨架类，未新增跨页面import|

交互使用`@tap`由uni编译到H5/微信事件；面板不新增scroll-view，不影响原列表滚动。分钟计时器仅组件可见full运行、隐藏/卸载清理，不驱动页面轮询。CSS变量复用全局token，无新增hex或渐变；已验证双端编译，真实微信设备效果仍待验收。
