# 赛季总结页重设计（小程序）

日期：2026-10-20
状态：待实现
范围：只改小程序 `football_insight_mini/src/pages/season-recap/` 的页面层与局部组件。**不改后端、API、类型、海报画布内容（`SeasonRecapPoster.vue` 绘制逻辑与 `poster.ts` 不动）**。

## 现状问题

1. 页面开头是一张 `FiPanel + FiSectionHeading`（kicker + 大标题）卡片，和其他二级页（`topics`、`transfer/board`、`lineup/list`）「`FiSecondaryNav` + 页面内纯文字标题 + 一行说明」的通用头部不一致。
2. 页面太长：比赛列表是一场一行的整宽按钮（一个赛季约 30 场），距离输入和「生成海报」按钮在列表下面，生成后海报再追加到页面最底部。用户要来回滚动才能完成操作、看到结果。
3. 大段口径说明直接平铺；已保存的里程结果在页面上看不到，只能生成海报才知道。
4. 没有批量操作：季票球迷要逐场点十几次主场。

## 设计目标

- 一屏内看到「我的里程」和比赛选择；主操作按钮始终可点到，不需要滚到底。
- 选比赛的点击次数少：紧凑网格 + 主/客筛选 + 一键全选当前筛选。
- 生成结果就地弹出，不追加到页面底部。
- 头部与其他二级页一致；样式全部走 token，沿用红色主色、卡片层级。

## 页面结构（自上而下）

```text
FiSecondaryNav（保持不变，share 行为不变）
┌ 页面标题区（纯文字，不包卡片）
│  recap-title   「2026 我的赛季足迹」
│  recap-caption 「勾选去过的蓉城比赛，生成你的里程海报」
├ SeasonRecapSummary  里程总览卡（已保存结果）
│  大数字：总里程 N 公里
│  三列：客场 X 公里 | 主场 Y 公里（未填写显示「—」）| Z 场 · W 城
│  无已保存记录（summary.matches === 0）时：大数字处显示「0」，副文案「还没有记录，勾选比赛后生成海报」
│  选择或距离已修改但未保存时，卡片右上角小标签「未保存」
├ SeasonRecapMatches  比赛选择卡
│  头部行：「我去过的比赛」 + 右侧「已选 n / 总数」
│  FiSegmented compact：全部 n / 主场 n / 客场 n（显示的是各筛选下的场次总数）
│  批量按钮（文字按钮，靠右）：当前筛选下未全选 →「全选主场 / 全选客场 / 全选」；已全选 →「清空」
│  三列网格的比赛小卡（按日期顺序）：
│    第一行：主/客小标签（主=红底浅色系，客=中性灰） + MM-DD
│    第二行：对手名（单行省略）
│    选中：主色边框 + 浅红底 + 右上角 ✓ 角标；按压 hover-class
│  空列表：「本赛季暂时没有已结束的蓉城比赛」
├ 主场距离行（仅当已选 ≥1 场主场时出现，放在比赛卡片内部底部或紧随其后一张小卡）
│  「去主场单边距离 [ 输入框 ] 公里」+ 一行小字「填一次即可，会记住」
├ 「里程怎么算 ›」文字行（页面自建可点击行，箭头展开时旋转）+ FiCollapse（只有 `open` prop，无内置标题；默认收起），内容为原口径说明：
│  客场：成都至对手主场城市的大圆直线距离 × 2，每场取整后相加；
│  主场：单边距离 × 2 × 场次，合计取整；取消或未结束的比赛不计入。
└ 页面底部留出 dock 高度 + 安全区的 padding
SeasonRecapDock  固定底部操作条（position: fixed，带安全区）
   左：「已选 n 场」+ 小字「主 a · 客 b」
   右主按钮：
     generating      →「正在生成…」（disabled）
     有 posterPath   →「查看海报」（打开弹层，不重新生成）
     其他            →「生成海报」
SeasonRecapPosterSheet  海报结果弹层（基于 FiBottomSheet）
   title「我的赛季海报」，图片 widthFix，点击图片 previewImage
   footer：MP-WEIXIN「保存到相册」+「分享给好友」(open-type=share)；
           H5「查看大图，长按保存」+「复制分享链接」
```

未登录：标题区下面一张卡片「登录后记录观赛足迹，换设备也能继续查看」+「登录并制作」按钮；不显示 dock。
加载中：`FiLoading`（文案沿用现有）。不显示 dock。
错误：标题区下方一行红色错误文本；无数据时带「重新加载」按钮。生成时的校验错误（如未填距离）也在这里显示，同时用 `uni.showToast({icon:'none'})` 提示，避免用户在底部 dock 点按钮却看不到顶部的错误。

## 交互细节

- 海报 `posterReady` 后自动打开弹层；关闭弹层后 dock 显示「查看海报」，可再次打开。
- 任何勾选或距离修改都会作废海报（flow 已有 `invalidatePoster`），dock 回到「生成海报」，总览卡显示「未保存」。
- 弹层打开时用 `page-meta` 锁滚动（页面已有 `<page-meta>`，拼接 `pageLockStyle`，参考 `membership-purchase/index.vue` 与 `AGENTS.md`「小程序弹框滚动锁」）。
- 生成中所有比赛卡、筛选批量按钮、输入框禁用。
- 分享（`onShareAppMessage` / `onShareTimeline`）、登录跳转、`resolveRecapSeason` 逻辑保持不变。

## 代码组织

遵循 `football_insight_mini/AGENTS.md` 页面拆分模式；局部组件只用 props/emits，不 import `api/**`。

- `index.vue`：只做编排（生命周期、flow wiring、弹层开关、scroll lock、toast）。
- `helpers.ts` 新增纯函数（并在 `helpers.test.ts` 补测试）：
  - `type RecapFilter = 'all' | 'home' | 'away'`
  - `filterRecapMatches(matches, filter)`
  - `recapSelectionStats(matches, selected)` → `{ total, home, away }`（只统计 matches 中存在的 id）
  - `recapFilterCounts(matches)` → `{ all, home, away }`
  - `isFilterFullySelected(matches, selected, filter)`
  - `toggleFilterSelection(matches, selected, filter)` → 新 id 数组：当前筛选已全选则移除这些 id，否则补齐；保持去重，不影响其他筛选下的已选项
  - `formatRecapDate('2026-03-01')` → `'03-01'`（格式异常时原样返回）
- `flow.ts`：
  - 新增 `setSelected(ids: number[])`：守卫同 `toggle`（generating 时忽略；只保留 view.matches 中存在的 id；去重），然后 `invalidatePoster()`；`flow.test.ts` 补用例。
  - 新增 `dirty` 计算：当前 selected（排序去重）/ distance 与最近一次加载或保存返回的 `selected_match_ids` / `home_one_way_km` 不一致即为 true；补测试。
  - 现有行为与测试不得回退。
- 组件（`src/pages/season-recap/components/`）：
  - `SeasonRecapSummary.vue`（props: summary, season, dirty）
  - `SeasonRecapMatches.vue` 重写为网格 + 筛选 + 批量（props: matches, selected, disabled；emits: toggle(id), select(ids)）；筛选状态放组件内部即可。
  - `SeasonRecapDock.vue`（props: stats, generating, hasPoster；emits: generate, open）
  - `SeasonRecapPosterSheet.vue`（props: visible, posterPath；emits: update:visible, save, preview, copy）。分享按钮直接 `open-type="share"`。
  - `SeasonRecapPoster.vue` 保持不变。
- 距离输入可留在 `index.vue` 或单独小组件，自行判断。

## 样式约束

- 颜色、间距、字号、行高、圆角、边框、阴影、字重全部用 `fi-tokens.css` 的 token；`bun test` 里的 `scripts/style-token-audit.test.mjs` 必须通过，**不要往 `LEGACY_OFF_SCALE` 加名额**。缺 token 先在 `src/styles/fi-tokens.css` 按三层规则补。
- 标题区字号/间距参考 `pages/topics/index.vue` 的 `topics-title` / `topics-caption`，保持二级页一致。
- 总览卡可用主色强调大数字；如用渐变，用已有 token（如 `--fi-gradient-*`）。
- mp-weixin 陷阱（`AGENTS.md`）：不要在自定义组件标签上写 class 改外观；flex 容器里子组件撑满由父容器 `align-items: stretch` 解决；选择器只用 class，不用 `[disabled]` 属性选择器（改成状态 class）。
- 网格用 `display: grid; grid-template-columns: repeat(3, minmax(0, 1fr))` 或 flex-wrap，两端都要正常；对手名单行省略。
- 不用 `:hover`，用 `hover-class`；单位只用 rpx/token。

## 验收

1. 头部：`FiSecondaryNav` + 纯文字标题，与 `topics` 页视觉一致。
2. 一屏（iPhone 12 尺寸）内可见：标题、总览卡、比赛网格前几行、底部 dock。
3. 主场筛选 →「全选主场」一次选中全部主场；再点变「清空」。
4. 选了主场但没填距离点生成：有 toast 和错误文本，不发保存请求。
5. 生成成功自动弹出海报弹层；关闭后 dock 为「查看海报」；改勾选后变回「生成海报」。
6. `bun run type-check`、`bun test`、`bun run check:boundaries`、`MINI_REVIEW_SKIP=1 bun run build:mp-weixin`、`bun run verify:mp-components` 全部通过。
