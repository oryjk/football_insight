# 小程序分包设计（第一期：盯盘页）

> 状态：待实现。设计者：Claude。实现：其他 agent。评审：Claude。
> 给 agent 的提示词在第 0 节，直接读本文件即可。

## 0. 给实现 agent 的提示词

你要在 `football_insight_mini` 里做小程序分包，范围只有第一期：把盯盘页 `pages/ticket-watch` 拆成一个分包。请先读完本文件，再读 `football_insight_mini/AGENTS.md`。

硬性约束：

1. 只做第一期，不要顺手拆别的页面。尤其**不要**拆 `membership-purchase`（付款路径，见第 4 节）。
2. 页面路径必须保持 `/pages/ticket-watch/index` 不变，不能改任何 `navigateTo`、分享路径、跳转常量。做法见第 3 节（`root` 用现有目录）。
3. 不要上传小程序、不要部署、不要提交 `manifest.json` 里由发版脚本生成的版本号。`bun run mp:release` 不要跑。只构建和本地验证。
4. 不要改 `scripts/check-import-boundaries.mjs` 的 ALLOWLIST，不要新增例外。
5. 不要碰 `football_insight_admin_android`。
6. 每一步都要记录主包体积的前后对比（第 6 节），写进 `football_insight_mini/docs/subpackage-validation.md`。
7. 做完后提交并推送，subject 用英文，例如 `feat(mini): split ticket-watch into a subpackage`。

交付物见第 7 节。完成后只汇报结果和数字，评审由我做。

## 1. 结论：该开始分包，但只做一期

| 事实 | 数据 |
|---|---|
| 主包体积 | 1737KB / 2048KB（85%） |
| 已经超限过一次 | 1.0.86 上传时 2058KB，被微信拒绝，靠删无用图片救回 |
| 单次功能增量 | 盯盘页最佳时段面板约 +20KB；一个中等功能 +30～60KB |
| `optimization.subPackages` | 现在是 `false`，没有任何分包 |
| 最大的页面目录 | `pages/ticket-watch` 205KB，其余都在 100KB 以下 |

不分包的话，再做 5 到 8 个功能就会再次超限，每次都要临时找东西删。分包的另一个好处是首次打开更快（主包更小）。

但不用全拆。只拆盯盘页就够：

- 它是体积最大的单页，拆出去预计主包降到约 1.5MB，余量约 500KB，够用很久。
- 它只有一个入口（洞察页的入口卡片），容易做预加载。
- 其余页面都很小（team-board 41KB、topic-detail 24KB、support 20KB、topics 9KB），收益小，不值得引入更多风险。

## 2. 现状要点

- `src/pages.json` 没有 `subPackages`，12 个页面全在主包，5 个 tab（home、rankings、insights、seat-swap、user）。
- 盯盘页只被 `pages/insights/index.vue` 跳转：`url: '/pages/ticket-watch/index'`。
- `src/static/ticket-watch/*` 这些图标被主包的 `components/topics/FiTopicHeat.vue` 引用，所以静态资源**保持在主包**，不动。
- 项目的边界检查已经保证页面之间不互相 import，这是分包的前提，已满足。
- `scripts/mini-ci.mjs` 上传后会打印 `分包信息`，可直接用来看每个包的大小。

## 3. 技术方案

### 3.1 `root` 用现有目录，路径零变化

在 `pages.json` 加：

```json
"subPackages": [
  { "root": "pages/ticket-watch", "pages": [ { "path": "index", "style": { ...原来的 style... } } ] }
]
```

并把 `pages` 里原来的 `pages/ticket-watch/index` 条目**移到** `subPackages` 里（保留它原有的 `style` 配置，不要丢）。

这样页面的完整路径仍是 `pages/ticket-watch/index`，源码目录不用搬，所有跳转、分享、测试里的路径都不用改。该目录下的 `components/`、`*.ts` 辅助模块会一起进入分包。

### 3.2 打开自动分包优化

`src/manifest.json` 的 `mp-weixin.optimization.subPackages` 改成 `true`。含义：只被分包页面使用的模块（例如 `api/ticketWatch.ts`、盯盘专用的 helpers）会自动打进分包，不留在主包的 `common/vendor.js` 里。被主包和分包共用的模块仍留在主包。

这一项是真正减小主包的关键，只拆目录不开它，体积降不了多少。

### 3.3 预加载

在 `pages.json` 加 `preloadRule`，用户进入洞察页时就在后台下载盯盘分包，点进去时几乎无等待：

```json
"preloadRule": {
  "pages/insights/index": { "network": "all", "packages": ["pages/ticket-watch"] }
}
```

分包名就是 `root`。盯盘页是核心功能，不能让用户点进去后等下载。

### 3.4 不能出现的引用

微信规定：**主包代码不能 require 分包里的文件**。已知需要确认的点：

- 主包里有没有文件 import `pages/ticket-watch/` 目录下的任何东西（包括 `.css`、`.ts`、组件）。如果有，要么把那部分挪到 `src/utils` 或 `src/components` 共用，要么证明它实际并不需要。
- 构建后检查：主包的所有 `.js` 里没有 `require('../pages/ticket-watch/...')` 这样的引用。

## 4. 为什么不拆 `membership-purchase`

它是付款页。分包首次进入需要下载，弱网下失败会直接丢订单。它只有 29KB，拆出去收益很小，风险在收入路径上，不划算。以后即使要拆，也要单独设计并保证付款路径可靠。

## 5. 要同步检查的脚本和流程

分包会改变构建产物的目录结构，下面这些可能要跟着适配，agent 要逐个看，不用的就说明不用：

- `scripts/verify-mp-component-registrations.mjs`：检查自定义组件是否注册，要确认它能扫到分包里的页面和组件。
- `scripts/dev-mp-weixin-sync.mjs`、`scripts/sync-mp-weixin-dist.sh`：开发时同步产物，确认复制逻辑不假设单包结构。
- `scripts/mini-ci.mjs`、`scripts/mini-release.mjs`：上传流程，确认不依赖 `__FULL__` 这个单包名（现在只是打印）。
- H5 构建 `bun run build:h5`：`subPackages` 在 H5 下应无影响，但要实际构建验证。
- 所有 `bun test` 里涉及盯盘页路径的测试。

## 6. 验证与度量

必须给出下面这些数字，写进 `football_insight_mini/docs/subpackage-validation.md`：

1. 改动前：`bun run build:mp-weixin` 后的主包体积（用 `dist/build/mp-weixin` 目录算，不含分包）。
2. 只拆目录、未开 `optimization.subPackages`：主包体积。
3. 开启 `optimization.subPackages` 后：主包体积、盯盘分包体积。
4. `common/vendor.js` 前后大小。
5. 通过 `miniprogram-ci` 的 `getCompiledResult` 或现有上传脚本的 `分包信息` 输出各包大小（**不要真的上传**，如果拿不到就说明原因，用 dist 目录统计代替）。

验收标准：

- 主包 ≤ 1600KB（目标约 1500KB）。
- 盯盘分包 ≤ 400KB。
- `bun run type-check`、`bun test`、`bun run check:boundaries`、`bun run verify:mp-components`、`bun run build:mp-weixin`、`bun run build:h5` 全部通过。
- 构建产物里主包不引用分包文件（第 3.4 节）。

## 7. 交付物

1. `pages.json`（subPackages + preloadRule）、`manifest.json`（optimization）的改动。
2. 因第 3.4、第 5 节发现的必要适配。
3. `football_insight_mini/docs/subpackage-validation.md`：第 6 节的数字、做了哪些适配、没做什么。
4. `CHANGELOG.md` 一条记录，说明原因（主包逼近 2MB 上限）。
5. 提交并推送，不上传、不部署。

## 8. 上线前我会在真机上确认（你们不用做，写在这里让大家知道）

- 冷启动直接进入盯盘页（分享链接场景），分包能正常下载并显示。
- 从洞察页点进去：已预加载，无明显等待。
- 弱网或飞行模式下首次进入的失败表现（应有可理解的提示，而不是白屏卡死）。
- 盯盘页内全部功能：下拉刷新、获取比赛 ID、最佳时段面板、折叠动画、付费位点击跳转会员购买页。
- 用体验版（robot 2）验证，通过后再决定是否提审。

## 9. 回滚

分包改动集中在 `pages.json` 和 `manifest.json` 两个文件，回滚就是还原这两处，不涉及目录搬迁，风险低。
