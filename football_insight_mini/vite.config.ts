import process from "node:process";
import { defineConfig } from "vite";
import uni from "@dcloudio/vite-plugin-uni";

// 发布入口（scripts/mini-release.mjs）会创建临时空目录并经 FOOTBALL_RELEASE_ENV_DIR 传入，
// 作为 Vite envDir，阻止自动读取 .env.local / .env.production.local 等本地覆盖层；
// 未设置时保持现有行为，日常 dev / H5 构建不受影响。
const releaseEnvDir = process.env.FOOTBALL_RELEASE_ENV_DIR;

// https://vitejs.dev/config/
export default defineConfig({
  ...(releaseEnvDir ? { envDir: releaseEnvDir } : {}),
  plugins: [uni()],
});
