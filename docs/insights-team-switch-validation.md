# 洞察页统一球队切换与展示精简

日期：2026-10-18。后续修正赛前倾向展示，只改小程序与文档。

## 原因与结果

- 原倾向卡与进失球卡都内嵌球队选择，绑定同一 selectedTeamId，形成重复控制；现只在页面顶部保留一处选择，统一控制概率、进失球和贡献内容。
- 切队本身没有重新请求贡献，但预测先清空再显示一行 loading，竖排比赛数量也因球队而变，导致卡片收缩和下方内容移动。现比赛为固定单行横向滚动，预测区保留尺寸，只有内部内容淡入，不重建整张卡片。
- 预测按比赛缓存 60 秒（包括明确暂无响应），命中时同步切换、不进入 loading；同一场在途请求合并，后台逐一预取各球队下一场。过期重新读取；审核/会话切换/卸载清空缓存并失效旧请求。不保留其他球队的概率冒充新球队。
- 进失球直接显示当前球队数据，取消逐整数计数、每个数字重新 key 和 pop 动画；只在当前内容区做 260ms 透明度淡入。组件局部禁用全局 briefing-card 上浮，减弱动态效果时取消新动效。
- 按负责人截图指令删除“本赛季联赛较均衡，预测把握度偏低”与常规同步/冻结状态解释，并删除前端无调用的 confidenceText；后台 τ 校准、接口字段不变。保留比赛信息、概率、赛程变更提示、战绩和固定模型说明。

“公开赛季战绩”统计真实赛前发布后、完赛计分的命中和 RPS。当前公网查询 matches=0，各 RPS=null；2802 场历史只用于训练/回测，没有伪造为公开战绩。这个区块及后台计分功能保留，未来已预测比赛结束并同步后累积。

## 验证

先补三项失败测试，再实现缓存/预取：命中缓存仍进入 loading、缺少预取、未过期时再次请求均先失败。补充开赛边界失败测试，修复缓存未来比赛列表造成的时间判断滞后。修复后，10 项 loader 测试全通过；原账号/审核/迟到请求用例保持通过。

| 命令/检查 | 实际结论 |
|---|---|
| `bun test` | 912 pass / 0 fail，5955 expect，85 文件，13.40s |
| `bun run type-check` | vue-tsc --noEmit，exit0 |
| `bun run check:boundaries` | 通过，未改 ALLOWLIST，无跨页导入，组件无 api 导入 |
| `bun run build:h5` | DONE Build complete |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | 构建、组件注册及产物 API 检查通过 |
| `bun run verify:mp-components` | All mini-program custom components are registered |
| H5 Playwright 320px / 390px | 单一选择器、缓存即时切队、总进失球立即替换、贡献仅请求一次、原倾向卡 DOM 保留、无横向整页溢出；后台预取和 550ms 慢请求期间布局稳定；reduced-motion 取消新动效 |
| 洞察目录 gradient 搜索 | 无结果，未新增 token、散落颜色或渐变 |
| `git diff --check` | 通过 |

H5 最初实测发现局部首元素 margin 折叠使慢请求期间下方位置移动，已用独立 flex 内容区消除。320px 预测区高度始终 51.1875px、比赛行 30.71875px；390px 分别为 62.390625px、37.4375px，等待与返回数据保持一致。首屏已有入场动画的亚像素采样差小于 1px；卡片没有折叠/移除。另实测修复了切回球队时比赛横向滚动停在旧位置的问题：只按比赛列表重建局部 scroll-view，整张卡片保留。截图与运行脚本位于忽略的 `.superpowers/sdd/2026-10-18-match-outlook/selector-*`。

浏览器 API 全部 mock，拦截其他外部请求，没有生产写入。微信真机的横向滚动与观感仍需负责人验收，编译不代替真机验收。没有上传小程序，没有发布 H5；本轮无后端改动，没有重启/重新部署后端，生产仍为此前部署的 2e1b840。

## 文件与跨端

- 修改 `src/pages/insights/index.vue`、`components/InsightsTeamPanel.vue`、`components/InsightsMatchOutlook.vue`、`components/InsightsSummaryPanel.vue`；控制器只在页面，展示组件只接 props/emits。
- 修改 `useMatchOutlook.ts` / `.test.ts`、`matchOutlook.ts` / `.test.ts`；不改 useInsightsData、sessionEffects 或付费度量。
- 同步设计、原验证报告、FEATURES/CHANGELOG；无新增生产组件或后端接口，无删除文件。
- 球队选择沿用原 scroll-view / @click；比赛按钮用 @tap / hover-class，scroll-into-view 对齐当前比赛，比赛列表变化时重置局部滚动位置。使用 uni-app 跨端能力，没有浏览器 DOM API 写入产品代码；布局/颜色/字体/间距均复用现有 token。
