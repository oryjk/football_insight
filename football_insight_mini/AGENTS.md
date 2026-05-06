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
