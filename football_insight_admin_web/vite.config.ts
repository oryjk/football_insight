import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath, URL } from "node:url";
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  return {
    plugins: [react(), tailwindcss()],
    base: env.ADMIN_WEB_BASE_PATH || "/",
    resolve: {
      alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
    },
    server: {
      strictPort: true,
      proxy: {
        "/api": {
          target: env.ADMIN_API_PROXY_TARGET || "http://127.0.0.1:18093",
          changeOrigin: true,
        },
      },
    },
  };
});
