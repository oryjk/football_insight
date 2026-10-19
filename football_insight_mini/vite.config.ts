import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import uni from "@dcloudio/vite-plugin-uni";

// 发布入口（scripts/mini-release.mjs）会创建临时空目录并经 FOOTBALL_RELEASE_ENV_DIR 传入，
// 作为 Vite envDir，阻止自动读取 .env.local / .env.production.local 等本地覆盖层；
// 未设置时保持现有行为，日常 dev / H5 构建不受影响。
const releaseEnvDir = process.env.FOOTBALL_RELEASE_ENV_DIR;
let revision = "unknown";
try {
  revision = execFileSync("git", ["rev-parse", "--short=12", "HEAD"], {
    cwd: fileURLToPath(new URL(".", import.meta.url)), encoding: "utf8", stdio: ["ignore", "pipe", "ignore"],
  }).trim();
} catch { /* Source archives may not contain .git. */ }

// https://vitejs.dev/config/
export default defineConfig({
  ...(releaseEnvDir ? { envDir: releaseEnvDir } : {}),
  define: {
    __FOOTBALL_BUILD_INFO__: JSON.stringify({ revision, builtAt: new Date().toISOString() }),
  },
  plugins: [uni()],
});
