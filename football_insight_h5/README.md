# football_insight_h5

Football Insight 的独立 H5 项目，面向小程序 web-view 嵌入场景，当前包含换座位页（`src/pages/seat-swap/`）。不依赖 uni-app，后端 API 与小程序共用同一套 `football_insight_service_backend_rs`。

## 技术栈

- Vite
- Vue 3
- TypeScript
- Bun

## 常用命令

```bash
cd football_insight_h5
bun install
bun run dev          # vite 开发服务器
bun run build        # 先 vue-tsc 类型检查，再 vite build
bun run type-check   # 仅类型检查
bun run preview      # 预览构建产物
```

`build` 前置 `vue-tsc --noEmit`，类型不过会直接失败。

## 配置

后端 API 基地址通过 `.env` 的 `VITE_API_BASE_URL` 配置（参考 `.env.example`，本地开发默认连本机 8092 的后端）。

## 鉴权

两种入口，token 统一存 localStorage（`src/lib/auth.ts`）：

- 小程序 web-view 嵌入：由小程序在 URL 上透传 `?token=`，页面启动时读取并写入 localStorage，随后从 URL 中摘除
- 测试环境：后端 H5 测试登录白名单（`H5_TEST_LOGIN_USER_IDS`），前端通过 `TestLoginPanel` 调 `/auth/h5-test-login` 换取 token；后端未配置白名单时接口 403

## 代码来源

纯函数与类型从 `football_insight_mini` 移植，后端 API 不变：

- `src/utils/seatSwap.ts`：换座相关表单/校验纯函数，移植自 mini 的 `src/pages/seat-swap/helpers.ts`
- `src/utils/stadiumRegions.ts`：看台区域布局与配色，移植自 mini 的 `src/utils/stadiumRegions.ts`
