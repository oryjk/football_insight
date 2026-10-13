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
- **间距**：`var(--fi-space-<rpx>)` 数字刻度（如 `--fi-space-18` = 18rpx）。
- **字号**：`var(--fi-font-<rpx>)` 数字刻度。
- **行高**：`var(--fi-leading-*)`（none/tight/snug/normal/loose/relaxed）。
- **圆角**：`var(--fi-radius-*)`（sm/md/lg/xl/round）。
- **边框/分割线**：`var(--fi-border-*)`、`var(--fi-color-divider*)`。
- **阴影**：`var(--fi-shadow-*)`（card/card-strong/soft/floating）。
- **字重**：`var(--fi-weight-*)`（bold/extrabold/black）。

存量约 900 处散落 hex 按「触碰哪个文件就顺手迁移哪个」增量替换，不要求一次性重写；刻度外的值先评估就近归档（视觉差 ≤2rpx 可并入相邻刻度），确需新值再进 token 文件。

**颜色例外**：球队/球衣等业务数据值、插画装饰色（球场图、海报绘制色）不是视觉 token，保留原样，可在相邻注释标注 decorative。

## 布局责任

跨组件布局由**父级 wrapper** 负责，子组件不自我布局。小程序端 uni-app 组件编译为微信自定义组件，存在宿主节点与样式隔离，以下写法 H5 正常、小程序静默失效（勿再犯）：

1. 不要用 `custom-class` + 父级 scoped 样式给子组件根节点做布局——作用域 id + 样式隔离会让整条规则失效。布局类加在自己模板内的包裹 view 上。
2. flex 行容器里子组件宿主节点收缩为内容宽，子组件内部 `width: 100%` 撑不满。由 flex 容器侧解决（`flex-direction: column` + `align-items: stretch`）。
3. 通用原则：**H5 显示正常不代表小程序正常**，跨组件宽度/布局必须在 mp-weixin 端实际验证。

## 组件契约

- 跨页面通用组件命名带 `Fi` 前缀（`FiBottomSheet`、`FiBrandNav`、`FiLoading`、`FiLoginFloat`），放 `src/components/`；页面专属组件放 `src/pages/<domain>/components/`。
- **运行时 Vue 组件必须直接从 `.vue` 文件导入**（如 `import FiLoading from "@/components/FiLoading.vue"`），不走 barrel 二次导出——小程序编译器可能不追踪二次导出，导致 WXML 有标签但 JSON 缺 `usingComponents`，构建成功却渲染不出。类型和纯函数可以走 barrel。
- 展示组件只通过 `props` 接收数据、`emits` 发出意图，不 import api / session / 认证存储（边界检查机器强制）。
- 新增页面/组件后执行 `bun run build:mp-weixin`；构建尾部的组件注册检查报 `Unregistered mini-program components` 时先核对 `.vue` 直接导入。

品牌导航 `FiBrandNav` 使用透明足球主体图标，标题与副标题横向排成一行。内容区高度由 token `--fi-brand-nav-bar-height`（88rpx）控制，页面顶部占位统一使用 `--fi-brand-nav-height`（状态栏 + 内容区），不要在页面里另写旧高度。小程序品牌行与微信胶囊居中对齐，右侧预留胶囊空间；H5 使用流式布局，长副标题省略尾部，二级页保留返回按钮。

### FiLoading（token 化示范组件）

加载态卡片，用于各页面数据加载中（如支持页、榜赛、洞察）。

- Props（均必填）：`title: string`、`caption: string`。
- 无 emits；不发 API、不碰 store。
- 宿主是一个纵向 flex 卡片（`gap`/`padding`/圆角/磨砂白底/浮起阴影），宽度由父级 wrapper 决定；长 title/caption 自然换行，组件不做截断。
- 样式全部来自 component token：`--fi-component-loading-*`（bg/radius/shadow/gap/padding/title-*/caption-*），取值提取自组件原实现，token 化前后视觉不变。

### FiBottomSheet（既有契约，记录不改实现）

底部弹层，承载各页面的 sheet 交互。

- Props：`visible: boolean`（必填，配合 `v-model:visible`）、`title?: string`、`eyebrow?: string`（默认 `''`）、`height?: string`（默认 `'88vh'`）、`closeOnMask?: boolean`（默认 `true`）、`compactFooter?: boolean`（默认 `false`）。
- Emits：`update:visible`、`close`。
- Slots：默认（body，scroll-view 滚动）、`footer`。
- mask 已处理 `@touchmove.stop.prevent` 与内容区 `@touchmove.stop` 防穿透。

**弹框滚动锁约定**：小程序里 sheet 打开后要锁底层页面滚动时，由真正的页面入口（`src/pages/**/index.vue`）用 `page-meta` 做页面级锁：

```vue
<page-meta :page-style="pageLockStyle" />
```

```ts
const pageScrollLocked = ref(false)
const pageLockStyle = computed(() => pageScrollLocked.value ? 'overflow: hidden;' : '')
```

子组件不自己写 `page-meta`，通过 `emit` 通知父页面切换锁状态；普通 `view` 的 `height: 100vh; overflow: hidden` 锁不住页面级滚动，只能作辅助。参考实现：`rankings/index.vue`、`seat-swap/index.vue` 的球队赛季弹框。

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
