<!-- From: /Users/carlwang/football_insight/football_insight_mini/AGENTS.md -->
# football_insight_mini 前端约定

## 审核版本号

审核版本号由本项目后端的 mini-review 登记库统一分配（`f_i_mini_review_statuses` 表），**不再手动维护**。

- 发版用 `bun run mp:release`：构建前 `scripts/sync-manifest-version.mjs` 会调 `POST /api/v1/mini-review/allocate` 申请版本号，并同步写入 `src/manifest.json` 和 `src/config/generatedMiniProgramVersion.ts`。
- 登记库是唯一权威：最新版本仍在审核中则复用，已出审核则 `+0.0.1`；不要手动改 `manifest.json` 的 `versionName`。
- 如果用户说"改版本号"或"指定版本号"，用显式覆盖：`MINI_PROGRAM_VERSION=x.y.z bun run build:mp-weixin`（作为指定版本传给登记接口）。
- 纯本地离线构建（不登记、不递增版本）用 `MINI_REVIEW_SKIP=1 bun run build:mp-weixin`。
- 微信审核结束后要调 `PUT /api/v1/mini-review/review-status` 把当前版本标记 `is_reviewing=false`，否则后续构建会一直复用审核中的版本号。
- 若构建分配了新版本号，`src/manifest.json` 与 `src/config/generatedMiniProgramVersion.ts` 的变更需要单独提交。

运行时来源：`src/api/system.ts` 的 `MINI_PROGRAM_VERSION` 优先读 `src/config/generatedMiniProgramVersion.ts`，构建时生成。

## 小程序弹框滚动锁

微信小程序里如果弹框、底部 sheet、全屏 mask 打开后需要禁止底层页面滚动，优先使用真正页面入口里的 `page-meta` 做页面级锁：

```vue
<page-meta :page-style="pageLockStyle" />
```

```ts
const pageScrollLocked = ref(false)
const pageLockStyle = computed(() => pageScrollLocked.value ? 'overflow: hidden;' : '')
```

注意：

1. `page-meta` 必须放在 `src/pages/**/index.vue` 这类真正的小程序页面文件里。
2. 如果弹框逻辑在子组件中，子组件不要自己写 `page-meta`；应通过 `emit` 通知父页面切换 `pageScrollLocked`。
3. 只给普通 `view` 加 `height: 100vh; overflow: hidden` 通常锁不住小程序页面级滚动，只能作为辅助。
4. mask 仍然需要配合 `@touchmove.stop.prevent`，弹框内容区可用 `@touchmove.stop`，防止触摸事件穿透到底层。

这个项目里球队赛季弹框的滚动穿透问题，最终就是通过父页面 `page-meta` + 子组件 emit 锁状态解决的。后续遇到同类问题，优先按这个模式处理。
