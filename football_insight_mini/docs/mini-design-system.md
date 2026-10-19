# football_insight_mini 设计系统（mini-design-system）

本文档是足球小程序 H5 / mp-weixin 两端的**现行视觉规范**。视觉身份由本项目决定：红（`#dc2626`）主色、墨色文字、浅色画布、卡片式信息层级。与报名系统的对齐仅限工程方法（三层 token 组织、组件契约、布局责任划分），**不引入其主题皮肤，不把足球改成其他风格**。结构约定见 [`mini-architecture.md`](./mini-architecture.md)。

`src/styles/fi-tokens.css` 是 token 的唯一事实源（由 `App.vue` 全局导入，兼容 `uni.css` 旧变量）；本文件解释组织规则与组件契约，具体取值以 token 文件为准。

## Token 三层

```text
primitive（--fi-primitive-*）   原始值，不直接在页面引用
   ↓
semantic（语义别名）            结构 UI 统一引用这一层
   ↓
component（--fi-component-*）   组件级复合契约（卡片、kicker/标题、关闭按钮、加载卡等）
```

覆盖八个维度，引用规则：

- **颜色**：`var(--fi-color-*)`。新增页面、组件、共享壳层必须引用 token，不新增散落 hex；需要新色值时先加 primitive，再建 semantic 别名。
- **间距**：页面同级卡片和卡片列表统一用 `var(--fi-space-card-gap)`，默认 16rpx；卡片内部文字、按钮、指标等布局继续使用 `var(--fi-space-<rpx>)` 数字刻度。
- **字号**：`var(--fi-font-<rpx>)` 数字刻度。
- **行高**：`var(--fi-leading-*)`（none/tight/snug/normal/loose/relaxed）。
- **圆角**：`var(--fi-radius-*)`（sm/md/lg/xl/round）。
- **边框/分割线**：`var(--fi-border-*)`、`var(--fi-color-divider*)`。
- **阴影**：`var(--fi-shadow-*)`（card/card-strong/soft/floating）。
- **字重**：`var(--fi-weight-*)`（bold/extrabold/black）。

存量约 900 处散落 hex 按「触碰哪个文件就顺手迁移哪个」增量替换，不要求一次性重写；刻度外的值先评估就近归档（视觉差 ≤2rpx 可并入相邻刻度），确需新值再进 token 文件。

**颜色例外**：球队/球衣等业务数据值、插画装饰色（球场图、海报绘制色）不是视觉 token，保留原样，可在相邻注释标注 decorative。

### 动效 token

新增或修改动效优先引用 `--fi-motion-*`，取值收拢自 `FiBottomSheet` / `FiSegmented` 现有动画：

| token | 值 | 用途 |
|---|---|---|
| `--fi-motion-duration-fast` | 120ms | 按压反馈 |
| `--fi-motion-duration-base` | 220ms | 列表入场、文案切换、箭头旋转 |
| `--fi-motion-duration-slow` | 280ms | 浮层（如浮窗按钮）入场 |
| `--fi-motion-ease-out` | `cubic-bezier(0.22, 1, 0.36, 1)` | 入场 |
| `--fi-motion-ease-in` | `cubic-bezier(0.4, 0, 1, 1)` | 退场 |
| `--fi-motion-ease-spring` | `cubic-bezier(0.3, 0.9, 0.3, 1)` | 滑块类位移 |

动画只动 `transform` 与 `opacity`，避免小程序重排卡顿；存量页面的动画不强制迁移。

## 布局责任

跨组件布局由**父级 wrapper** 负责，子组件不自我布局。小程序端 uni-app 组件编译为微信自定义组件，存在宿主节点与样式隔离，以下写法 H5 正常、小程序静默失效（勿再犯）：

卡片之间的距离只由父容器的 `gap: var(--fi-space-card-gap)` 提供，卡片根节点不再叠加外部 margin。一个组件内部包含多张卡片时，也应使用真实 `view` 包裹节点统一排列。固定弹层放在卡片容器外；避免其空宿主节点参与小程序的 flex 间距。跨组件的外部布局类写在父页面的真实 `view` 上，例如会员权益组件外部的包裹节点；换座地图的 sticky 和横向扩展也由页面包裹节点承担。

1. 不要用 `custom-class` + 父级 scoped 样式给子组件根节点做布局——作用域 id + 样式隔离会让整条规则失效。布局类加在自己模板内的包裹 view 上。
2. flex 行容器里子组件宿主节点收缩为内容宽，子组件内部 `width: 100%` 撑不满。由 flex 容器侧解决（`flex-direction: column` + `align-items: stretch`）。
3. 通用原则：**H5 显示正常不代表小程序正常**，跨组件宽度/布局必须在 mp-weixin 端实际验证。

## 组件契约

- 跨页面通用组件命名带 `Fi` 前缀（`FiBottomSheet`、`FiBrandNav`、`FiLoading`、`FiLoginFloat`），放 `src/components/`；页面专属组件放 `src/pages/<domain>/components/`。
- **运行时 Vue 组件必须直接从 `.vue` 文件导入**（如 `import FiLoading from "@/components/FiLoading.vue"`），不走 barrel 二次导出——小程序编译器可能不追踪二次导出，导致 WXML 有标签但 JSON 缺 `usingComponents`，构建成功却渲染不出。类型和纯函数可以走 barrel。
- 展示组件只通过 `props` 接收数据、`emits` 发出意图，不 import api / session / 认证存储（边界检查机器强制）。
- 新增页面/组件后执行 `bun run build:mp-weixin`；构建尾部的组件注册检查报 `Unregistered mini-program components` 时先核对 `.vue` 直接导入。

品牌导航 `FiBrandNav` 使用 64rpx 透明足球主体图标，副标题位于主标题右侧，两段文字底部对齐，整体文字组合与图标居中。品牌行与返回按钮独立使用 `--fi-space-brand-nav-row-size`（52rpx），避免放大 logo 后撑高布局或改变文字位置。内容区高度由 token `--fi-brand-nav-bar-height`（76rpx）控制，页面顶部占位统一使用 `--fi-brand-nav-height`（状态栏 + 内容区），不要在页面里另写旧高度。小程序品牌行与微信胶囊居中对齐，右侧预留胶囊空间；H5 使用流式布局，长副标题省略尾部，二级页保留返回按钮。

### FiSecondaryNav（二级导航）

二级页面统一使用 `FiSecondaryNav`：中间为「足球洞察」，左侧为圆角返回/分享按钮组，右侧预留微信胶囊空间，内容区仍为 76rpx。设置、账号及支付页面传 `shareable=false`，只保留返回按钮。分享按钮在微信端使用原生 `open-type="share"`，H5 通过 `share` 事件由页面复制公开链接；展示组件不读取业务数据或拼接分享内容。导航图标保留 SVG 源图，运行时使用透明 PNG，避免跨端图片格式差异。

### FiPanel / FiSectionHeading（内容卡片与区块标题）

内容卡片与区块标题统一用这两个组件，不再在页面/局部组件里重复定义 `.panel`、`.section-heading`、`.section-kicker`、`.section-title`。

```vue
<FiPanel>
  <FiSectionHeading kicker="站队状态" :title="stanceTitle">
    <template #meta><text class="meta-pill">已助力</text></template>
  </FiSectionHeading>
  <!-- 卡片内容 -->
</FiPanel>
```

- `FiPanel` 只管卡片外观（`--fi-component-panel-*`：背景、圆角 36rpx、卡片边框/阴影、内边距 20rpx）；`flush` 去内边距，`clip` 裁剪溢出（卡内绝对定位遮罩）。
- `FiSectionHeading` 管标题区（`--fi-component-kicker-*` / `--fi-component-title-*`）：kicker 22rpx 加粗灰色，主标题 `size` 三档 `md` 44rpx（默认）/ `lg` 50rpx / `xl` 56rpx（页面 hero）；`align="center"` 用于带关闭按钮的标题行；`meta` 插槽放徽标/按钮，富文本标题用 `heading` 插槽整体替换。
- 小程序组件样式隔离：父级 scoped 样式到不了组件内部，也不要给组件标签挂布局/修饰 class。需要定制的内容（遮罩、模糊、点击区域）放进插槽——插槽内容由父组件编译，父级样式生效。
- 已迁移：洞察页 6 个面板、首页面板（原 `HomePanelCard` 已删除）与主队助力面板、助力页。其余页面的 `.panel` / `.section-*` 按「触碰哪个文件就顺手迁移」增量替换，迁移即统一到上述标准外观。

### 洞察页联动内容与背景

- 回流看板独立置顶；球队选择、数据洞察/战术胶囊和对应内容统一放在一个 FiPanel 内，默认数据洞察。内容区以留白和细分隔线区分赛前倾向、球队概览和贡献分析，避免重复的大圆角卡片和阴影；指标与我的战术保留浅底区分。
- 内层 FiPanel 的背景、圆角、边框、阴影与内边距由页面内容区继承的 `--fi-component-panel-*` 调整；布局与分隔线放在父页面真实 view，不通过父级 scoped 样式穿透子组件。骨架屏和空态同样适配统一底板。
- 球场背景保留，首页和洞察共用 `--fi-color-page-backdrop-fade`：透明 45% → 半透明 78% → 页面浅底色 100%，背景高度 600rpx；洞察页底色与渐变终点一致。

### FiLoading（token 化示范组件）

加载态卡片，用于各页面数据加载中（如支持页、榜赛、洞察）。

- Props（均必填）：`title: string`、`caption: string`。
- 无 emits；不发 API、不碰 store。
- 宿主是一个纵向 flex 卡片（`gap`/`padding`/圆角/磨砂白底/浮起阴影），宽度由父级 wrapper 决定；长 title/caption 自然换行，组件不做截断。
- 样式全部来自 component token：`--fi-component-loading-*`（bg/radius/shadow/gap/padding/title-*/caption-*），取值提取自组件原实现，token 化前后视觉不变。

### 首页话题入口

- `HomeDailyTopicCard` 使用 `--fi-component-topic-hero-*` 契约：深色红晕渐变、大圆角、红色类别标签、白色标题和浅灰摘要；热度、阅读与收听在同一 footer 行中。继续保留下一张卡片的露出、滑动与圆点导航。
- 文章标签包含成都蓉城稳定 ID 时，卡片右侧展示现有队徽的放大半透明水印，顺时针倾斜 12°；尺寸和透明度归入 `--fi-component-topic-crest-*`。`FiTopicCrest` 由首页与全部话题列表共用：保持浅色列表布局，`history` 变体使用更淡的透明度。图片加载失败隐藏，装饰层不拦截点击。
- `FiTopicHeat`、`FiTopicReadEntry`、`FiTopicAudioEntry` 提供可选 `variant="hero"`，默认 `default` 保持列表/详情页现有外观；各组件自行持有样式，不依赖父级 scoped 穿透。首页阅读与收听均为纯图标：文档、播放/暂停及准备/加载中的省略点；两个按钮共用 action-width/height token（72rpx × 52rpx），使用块级 flex 避免图标基线引发上下偏移。保留按钮无障碍名称和未就绪禁用，状态变化不挤压相邻热度条。
- 热度读数仍为原计数及既有万/亿格式；装饰条只表示强弱，0–100 填充、超过 100 填满，不作为百分比或榜单指标。

### FiBottomSheet（新增均为向后兼容的可选 prop）

底部弹层，承载各页面的 sheet 交互。

- Props：`visible: boolean`（必填，配合 `v-model:visible`）、`title?: string`、`eyebrow?: string`（默认 `''`）、`height?: string`（默认 `'88vh'`）、`closeOnMask?: boolean`（默认 `true`）、`compactFooter?: boolean`（默认 `false`）。
- `fitContent?: boolean`（默认 `false`）：卡片随内容自适应高度，`height` 作为最大高度；内容区使用普通 `view` + `overflow-y: auto`，避免自适应 flex 下 `scroll-view` 内层高度大于可见区域而裁掉底部内容。默认模式保留固定高度和原结构。
- `zIndex?: number`（默认 `40`）：设置遮罩层级，需要覆盖页面浮层的表单弹层可显式传入。
- `respectWindowBottom?: boolean`（默认 `false`）：遮罩底部使用 `var(--window-bottom, 0px)`，用于 H5 底栏避让。
- Emits：`update:visible`、`close`。
- Slots：默认（body，默认 `scroll-view` 滚动，自适应时 `view` 滚动）、`footer`。
- mask 已处理 `@touchmove.stop.prevent` 与内容区 `@touchmove.stop` 防穿透。
- 关闭仍保留 220ms 动画。表单保存中需设置 `closeOnMask=false`，并在 `update:visible` 处理函数中拦截关闭（头部 `✕` 同样走该事件）；组件不自行修改 `visible`。由外部控制显隐，关闭动画期间需保留表单展示数据。

**弹框滚动锁约定**：小程序里 sheet 打开后要锁底层页面滚动时，由真正的页面入口（`src/pages/**/index.vue`）用 `page-meta` 做页面级锁：

```vue
<page-meta :page-style="pageLockStyle" />
```

```ts
const pageScrollLocked = ref(false)
const pageLockStyle = computed(() => pageScrollLocked.value ? 'overflow: hidden;' : '')
```

子组件不自己写 `page-meta`，通过 `emit` 通知父页面切换锁状态；普通 `view` 的 `height: 100vh; overflow: hidden` 锁不住页面级滚动，只能作辅助。参考实现：`rankings/index.vue` 的球队赛季弹框，`seat-swap/index.vue` 的发布、管理与地图弹层。

### 换座页

- 座位语义色：`--fi-color-seat-current(-bg)` 表示"当前座位"（红），`--fi-color-seat-desired(-bg)` 表示"目标座位"（绿），地图图例、候选卡、Dock、发布弹层共用。
- 票价分区配色（蓝、绿、紫、黄、藏青、红、VIP）是业务数据色，用 `/* decorative:start … */ … /* decorative:end */` 包裹保留原值。`scripts/seat-swap-style-audit.test.mjs` 会扫描 `src/pages/seat-swap` 下全部 `.vue`，decorative 块之外出现渐变、hex 或数字字重即测试失败。
- 地图不吸顶：首屏展示，划出视口后由 `SeatSwapMapFab` 浮窗唤起 `SeatSwapMapSheet` 弹层；点击规则由 `resolveSeatSwapMapTap` 统一。设计见 `docs/superpowers/specs/2026-10-19-seat-swap-page-refactor-design.md`。

## 动效与 reduced-motion

- 存量动效：FiBottomSheet 开合过渡、洞察数字动画（`utils/animatedInteger.ts` 的 `useAnimatedInteger`）。本次不迁移或替换动效实现。
- **reduced-motion 验证约定**：新增或修改动效（transition/keyframes/JS 数字动画）时，必须在系统「减弱动态效果」（reduced motion）设置下验证两端表现；对用户任务无必要的位移/缩放动效应提供降级（H5 可用 `prefers-reduced-motion` 媒体查询，MP 端以缩短时长/取消位移为准）。纯视觉验证以模拟器/截图为准，不新增快照断言。

## 跨端约束

- 统一 `uni.*` API，不调 `wx.*`；禁止浏览器 DOM API（`document.*`、`window.*`、`localStorage`），存储用 `uni.getStorageSync` / `uni.setStorageSync`。
- 微信专属能力（支付、订阅消息、`open-type` 等）用 `<!-- #ifdef MP-WEIXIN -->` 条件编译隔离，并提供 H5 降级路径。
- 样式统一 `rpx`，不混用 `px`/`vw`/`rem`；不用 `:hover`（用 `hover-class`）。

## 后续项目（不在当前基线内）

以下项目按设计文档第 8 节单独排期，达到进入条件后另行设计与验收：

- **Wot UI**：是否采用由独立迁移试点决定（选一个页面做 H5/MP 对照 + 体积变化，review 后再扩展），当前不作为依赖，不批量改皮肤。
- **H5 一次性 code 桥接**：独立 `football_insight_h5` 已存在（URL token 透传）；mini 内 web-view 嵌页与一次性 code 兑换协议待后端签发/兑换接口确认后单独设计。
- **普通分包**：当前 `pages.json` 无 subPackages；先测主包实际体积、列候选页面，五个 Tab 留在主包，无体积证据不机械分包。
