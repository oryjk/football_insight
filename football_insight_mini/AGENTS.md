<!-- From: /Users/carlwang/football_insight/football_insight_mini/AGENTS.md -->
# football_insight_mini 前端约定

## 审核版本号

当用户说"改版本号"或"改审核版本号"时，指的是**提交微信小程序审核时传给后端的版本号**。

这个值在三个地方使用，改版本号时必须**同时修改**：

1. `.env.production` → `VITE_MINI_PROGRAM_VERSION`
2. `.env.development` → `VITE_MINI_PROGRAM_VERSION`
3. `src/api/system.ts` → `DEFAULT_MINI_PROGRAM_VERSION` fallback 默认值

`src/manifest.json` 里的 `versionName` 只是微信小程序平台上**展示给用户看**的版本号，不影响 API 请求。

**总结**：审核版本号 = `.env` 里的 `VITE_MINI_PROGRAM_VERSION`，不是 `manifest.json` 里的 `versionName`。

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
