# 洞察页统一球队切换与展示精简

日期：2026-10-18。后续修正赛前倾向展示，只改小程序与文档。

最新指令：已移除公开赛季战绩整块及其请求，后台计分保留；下文初次验证中“保留战绩”属于当时状态，最新验证见末尾追加记录。

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

## 追加：切回球队后比赛双高亮

2026-10-18，负责人在微信端反馈“成都蓉城 → 大连英博 → 成都蓉城”后两场比赛同时有深色边框。

### 排查与修改

`selectedMatchId` 始终是单个标量，H5 按同一顺序切换时也只有一个 active；不是后端返回双选。

检查微信编译产物：父级 `scroll-view :key` 被编译成普通 `key` 属性，不能隔离内部 `wx:for` 的节点复用；按钮的 `wx:key` 原来只含 match_id。蓉城列表 `[A, B, C]` 中 B 是蓉城对大连，大连列表 `[B, D]` 中 B 是第一场。返回蓉城时，真实 uni 差量算法按索引发送比赛文字/ID/key 的变更，但原索引 0 的选中标志与索引 1 的未选中标志都没变，因此不发送这两个标志；原生按比赛 key 复用时，B 的旧高亮可能留下。

修复仅修改 `InsightsMatchOutlook.vue`：把现有比赛列表标识提为 computed，并加入每个比赛按钮的 key，使跨列表共享比赛不复用原生节点。同一列表中选择比赛仍只更新原按钮；外层卡片、缓存、会话、会员与付费度量不改。样式、接口、token 和版本号均不变。同步 FEATURES/CHANGELOG；没有新增生产文件或删除文件。

### 验证证据与限制

- 先用现有微信编译产物的实际 render 函数与 `@dcloudio/uni-mp-vue` 的真实 diff 算法生成 setData 补丁，再用 keyed 节点/属性依赖模型重放。修复前：Chengdu `[101]` → Dalian `[102]` → Chengdu-return `[101,102]`，断言失败；修复后每步只有期望的一个 active。20 轮、每轮 6 种状态，共 120 次，包含共享比赛换位、列表长度变化、同长度列表切换、手动改选比赛。
- 这个重放使用真实编译 render 和差量算法，**原生节点更新是模型，不是微信 SDK/真机运行**。H5 无法重现微信的原生复用故障。未把模拟结果写成真机已验收，仍需上传后验证负责人原步骤。
- H5 全部 API mock、外部流量拦截，在 320/390px 各重复 8 次成都↔大连切换，逐步断言 active 数量和比赛 ID；另验证慢请求时布局不收缩、缓存不重发、卡片 DOM 不重建。
- UI 修复未新增机械模板源码断言；实际编译产物重放脚本与日志放忽略目录 `.superpowers/sdd/2026-10-18-match-outlook/fixture-selection-*`，没有生产数据库操作。

| 命令 | 实际输出结论 |
|---|---|
| `bun test` | 912 pass / 0 fail，5955 expect，85 文件，13.30s |
| `bun run type-check` | vue-tsc --noEmit，exit 0 |
| `bun run check:boundaries` | 依赖边界检查通过 |
| `bun run build:h5` | DONE Build complete |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | DONE；组件注册及生产 API 产物检查通过 |
| `bun run verify:mp-components` | All mini-program custom components are registered |

微信/H5 仍使用原有 `@tap`、`hover-class` 和 `scroll-view`，没有浏览器专属 API 进入产品代码。没有上传小程序，没有重新部署或重启后端。

## 追加：移除公开赛季战绩展示与请求

2026-10-18，负责人确认此区块当前价值有限、文字过多，因此移除公开赛季战绩标题、空状态、命中率/RPS/基线、赛季初样本说明和分隔线。保留比赛选择、主胜/平/客胜概率及原来的一句参考说明；仅异常改期时提示赛程变化。

`InsightsMatchOutlook.vue` 去掉战绩 props/模板/样式，`index.vue` 去掉接线，`useMatchOutlook.ts` 停止请求战绩并删除相关状态与场次汇总，`matchOutlook.ts` 清理无用格式化函数和文案。原有 10 项 loader 测试保留，移除一个已删除功能的格式化测试及无用假数据。后端 API、数据库与赛后计分不改；客户端 API 模块/DTO 保留，避免扩大接口清理范围。

### 验证

- 修改前先在 H5 交互脚本断言区块不存在，实际失败：`removed season record is still visible`。修改后，320/390px 两种宽度均通过区块不存在和零次 `/match-outlook/record` 请求检查；保持单一球队选择，逐步切换 8 轮成都↔大连只选中一场，慢请求不折叠布局，卡片 DOM 保留。所有 API mock、拦截外部流量，没有生产写入。
- 微信编译产物的原生 key 更新重放模型 120 次切换仍通过；这不替代微信真机，仍需负责人上传后验收。没有为纯删除 UI 新增机械源码测试。

| 命令 | 真实输出结论 |
|---|---|
| `bun test` | 911 pass / 0 fail，5948 expect，85 文件，13.36s；相比上次少 1 项为删除的战绩格式化测试 |
| `bun run type-check` | vue-tsc --noEmit，exit 0 |
| `bun run check:boundaries` | 依赖边界检查通过，未改 ALLOWLIST |
| `bun run build:h5` | DONE Build complete |
| `MINI_REVIEW_SKIP=1 bun run build:mp-weixin` | DONE；组件注册与生产 API 产物校验通过 |
| `bun run verify:mp-components` | All mini-program custom components are registered |

展示改动之外只有上述无用读取/状态清理；未修改 useInsightsData、sessionEffects、付费度量、接口、版本号或 token。同步设计/FEATURES/CHANGELOG。未新增或删除生产文件，未上传小程序，未发布 H5，未重启本地后端，生产后端未重新部署或重启。
