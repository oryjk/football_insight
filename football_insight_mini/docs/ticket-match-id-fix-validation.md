# 获取比赛 ID 展示与弹层稳定性验证

2026-10-16，用户反馈获取比赛 ID 时页面跳动，显示数据库唯一 ID 而不是业务 ID。本修复独立于之前的展示重构，仅修改小程序；未上传、未部署后端、未修改接口或执行数据库写入。

## 根因与改动

- Rust 现有 map_match_summary 明确把上游 item.id 映射为 match_id、item.match_id 映射为 external_match_id。只读 GET current-match 核实当前二者分别为 584 与字符串 100；业务 ID 不应固定限定两位。
- 页面原来直接传 currentMatch.match_id 给 TicketMatchIdSheet。新增状态计算 matchIdDisplayValue，仅 unlocked 时返回 external_match_id.trim()，缺失返回 null；页面展示改接此值，字符串保留前导零。权益/下单的请求参数仍为内部 match_id，owner/revision/generation、支付和付费度量代码均未改。
- 原组件把 loading 当作未解锁购买态，成功后再换为高度不同的 ID 面板；居中位置随高度变化，且原入场动画含位移/缩放。新增独立查询态，不显示 ID 或购买入口；正文使用现有 token 计算最小高度，移除位移/缩放，只保留遮罩淡入。
- 缺失业务 ID 提示“业务 ID 暂不可用”，复制按钮禁用，不用数据库主键兜底。已解锁正常 ID 原样复制，会员和单场来源文案保留。
- page-meta 在原导航变量后合并弹层可见时 overflow:hidden，关闭恢复；mask touchmove.stop.prevent、card touchmove.stop，tap/close/pay/upgrade 对外事件和 @upgrade 页面绑定保持原样。

## TDD 与复审

- 新增 5 项状态/真实权益流程回归：业务 ID 78、100、08，查询仍传 584；查询中不暴露、解锁后显示；业务缺失不回退主键；locked/loading/paying 都不提供展示值。
- 新增 4 项真实 SFC 渲染/事件测试：查询中不闪购买入口且不显示 ID、复制保留 08、缺失业务 ID 提示并禁复制、locked/paying 保留原购买与禁用状态。测试 renderer 只提供原生节点宿主，执行真实组件模板及复制事件；不发网络或真实支付。
- RED：2 pass/7 fail（新 computed 尚不存在，加载态/缺失反馈断言失败）；GREEN：定向 9 pass/0 fail。首次全量类型检查发现仓库的 Bun 类型声明不支持 .not matcher，删除与正向 ID 断言重复的三项否定断言后重新完整验证。
- 独立只读复审通过，无阻塞/重要/次要问题，确认 API 字段映射、显示接线、会话竞态与小程序滚动锁。

## 真实验证输出

各命令退出码均为 0，原始日志位于会话 scratch `.tmp/ticket-match-id-fix/`。

| 命令 | 最终真实输出 |
|---|---|
| bun test | 811 pass / 0 fail / 5592 expect() calls，69 files，13.09s |
| bun run type-check | vue-tsc --noEmit，退出 0 |
| bun run check:boundaries | 依赖边界检查通过（组件↛api/页面、页面不互引、api 依赖受限、uni.request 收口） |
| bun run build:h5 | DONE Build complete. |
| MINI_REVIEW_SKIP=1 bun run build:mp-weixin | DONE Build complete.，组件全部注册、生产 API 校验通过；没有版本登记或上传 |
| bun run verify:mp-components | All mini-program custom components are registered. |
| git diff --check | 无输出，退出 0 |

构建保留原有 uni-app 更新与 Browserslist 旧数据提示，非构建失败。ticket-watch 目录 gradient/hex 搜索无结果。manifest、token 文件、边界 ALLOWLIST 未改。

## 布局与跨端验证

会话夹具 check-layout.cjs 从真实 SFC 生成 SSR 结构、使用真实组件样式和全局 token，在 Chromium 320×568、375×812、430×932 三尺寸，各测试普通/减弱动态效果以及 loading/unlocked/locked/paying 四态。原版居中 top 变化约 4.39–6.31px；修复后六组 top 变化均为 0，弹层不超出屏幕，背景原滚动位置保持 300。脚本和实际输出 layout.log 均在上述 scratch，不进入仓库。

新增样式使用 space-8/18/56/64/84、font-26、leading-relaxed/none 及 text-primary/surface 等现有语义 token；没有新增 token、hex 或渐变。正文 min-height 足够容纳原购买态，查询和解锁态垂直居中；卡片继续固定遮罩居中，淡入不含位移。

H5/微信均使用 @tap，不双绑 @click；保留原生按钮和禁用行为。此弹层无 scroll-view、无计时器；页面级滚动锁依照项目 page-meta 约定，导航 CSS 变量继续透传，组件宿主不承担布局。浏览器夹具不代表完整 H5 页面或微信真机验证；微信真机跳动、滑动防穿透与页面关闭后滚动恢复仍待用户体验版复测。未连生产数据库、未真实购买、未向公众号发消息。

## 文件清单

- 新增：scripts/ticket-match-id-sheet.test.ts、docs/ticket-match-id-fix-validation.md。
- 修改：src/pages/ticket-watch/index.vue、useTicketWatchState.ts、effects.test.ts、components/TicketMatchIdSheet.vue；根 CHANGELOG.md、FEATURES.md。
- 无删除文件；后端不变，本地/生产后端均未因本次改动重启或部署。
