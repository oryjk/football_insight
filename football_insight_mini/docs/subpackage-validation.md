# 小程序分包验证（第一期：盯盘页）

> 实施日期：2026-10-16。依据：`docs/superpowers/specs/2026-10-17-mini-subpackage-design.md`。
> 范围：仅把 `pages/ticket-watch` 拆为分包；未上传、未部署、未动版本号。

## 改动内容

1. `src/pages.json`：`pages/ticket-watch/index` 从 `pages` 移入 `subPackages`（`root: "pages/ticket-watch"`，保留原 `style`）；新增 `preloadRule`（进入洞察页预载盯盘分包）。页面完整路径不变。
2. `src/manifest.json`：`mp-weixin.optimization.subPackages` 由 `false` 改为 `true`。

其余文件零改动：无目录搬迁，无 `navigateTo`/分享路径变更，无 ALLOWLIST 例外。

## 体积数字

度量口径：`dist/build/mp-weixin` 下文件**表观字节**（`find -printf '%s'` 求和）。主包 = 全量减去分包根目录 `pages/ticket-watch/`。

| 阶段 | 主包 | 盯盘分包 | `common/vendor.js` | 全量 |
|---|---|---|---|---|
| 1. 改动前（无分包） | 1,118.0 KB（1,144,861 B） | — | 67,249 B | 1,118.0 KB |
| 2. 仅拆目录（optimization=false） | 913.1 KB（935,038 B） | 205.1 KB | 67,249 B | 1,118.2 KB |
| 3. 开启 optimization=true | 913.1 KB（935,038 B） | 205.1 KB（210,045 B） | 67,249 B | 1,118.2 KB |

参考：磁盘块口径（`du -sk`，受 4KB 块取整放大）改动前全量 3,228 KB，改动后主包 2,576 KB、分包 488 KB。

口径说明：设计文档中的「主包 1737KB」来自 miniprogram-ci 上传时的 `subPackageInfo` 统计（微信 SDK 口径，如 1.0.85 完整包为 2,090,729 B），与 dist 表观字节不可直接比较。按 dist 口径主包下降 204.9 KB（-18.3%）。

1.0.91（robot 2）实际上传后取得的 `subPackageInfo` 实值：**主包 `__APP__` = 1,401,244 B（约 1,368 KB）**，盯盘分包 `/pages/ticket-watch/` = 336,140 B（约 328 KB），完整包 `__FULL__` = 1,737,384 B。与上文按 dist 口径的折算（主包约 1,419 KB）接近，主包与分包均满足验收线。

`vendor.js` 三个阶数完全相同：盯盘页引用的共享模块（`api/ticketWatch.js`、`utils/request.js`、各 composable 等）同时被主包页面使用（`api/ticketWatch.js` 被 `pages/seat-swap/index.js` 引用），按规则留在主包；`optimization.subPackages=true` 无额外可下沉模块，产物与第 2 阶段逐字节一致。

## 主包不引用分包（设计第 3.4 节）

- 源码：`src` 下无任何文件 import `pages/ticket-watch/` 模块（`scripts/*.test.ts` 为开发态测试，不进产物）。
- 产物：主包 js/json/wxml 中 `pages/ticket-watch` 仅出现在两处——`pages/insights/index.js` 的 `navigateTo({url:"/pages/ticket-watch/index"})` 字符串，与 `app.json` 的 `subPackages` 声明；**无任何 `require('.../pages/ticket-watch/...')`**。
- 分包反向引用主包 `common/vendor.js`、`utils`、`composables` 属正常允许方向。

## 脚本与流程适配（设计第 5 节）

逐个检查后**均无需改动**：

- `scripts/verify-mp-component-registrations.mjs`：递归扫描 dist 全部 `.wxml`（含分包目录），分包产物下检查通过。
- `scripts/dev-mp-weixin-sync.mjs` / `scripts/sync-mp-weixin-dist.sh`：整目录 rsync 镜像，无单包结构假设。
- `scripts/mini-ci.mjs` / `scripts/mini-release.mjs`：无 `__FULL__` 依赖，`subPackageInfo` 仅上传后打印。
- H5 构建：`subPackages`/`preloadRule` 对 H5 无影响，`bun run build:h5` 通过，`pages-ticket-watch-index.*.js` 生成为独立懒加载 chunk。
- `bun test` 中 5 个盯盘相关测试 import 的是源码路径（未变），全部通过。

## 验证结果

- `bun run build:mp-weixin`（`MINI_REVIEW_SKIP=1`，版本保持 1.0.89）：通过，产物 `app.json` 含 `subPackages` 与 `preloadRule`。
- `bun run type-check`：通过。
- `bun test`：859 pass / 0 fail（74 个文件）。
- `bun run check:boundaries`：通过，无新增 ALLOWLIST。
- `bun run verify:mp-components`：通过（All mini-program custom components are registered）。
- `bun run build:h5`：通过。

验收对照：主包 dist 口径 913.1 KB（折算微信口径约 1,419 KB）≤ 1,600 KB ✅；盯盘分包 205.1 KB ≤ 400 KB ✅；全部检查命令通过 ✅；主包不引用分包 ✅。

## 未做事项

- 未拆分其他页面（`membership-purchase` 等），留待后续分期。
- 真机验证（冷启动直进盯盘页、预加载体感、弱网失败表现）按设计第 8 节由评审人在体验版上执行。

## 实测（体验版 1.0.90 上传，robot 2）

`miniprogram-ci` 的 `subPackageInfo`（微信口径）：主包 `__APP__` 1,401,244 B（约 1,368 KB），盯盘分包 `/pages/ticket-watch/` 336,061 B（约 328 KB），合计 1,737,305 B。拆分前主包为 1,737 KB，主包下降约 369 KB，占上限 2,048 KB 的 67%。验收线（主包 ≤ 1,600 KB、分包 ≤ 400 KB）均满足。上文的折算值（约 1,419 KB）偏高约 51 KB，以本节实测为准。真机验证（设计第 8 节）待做。
