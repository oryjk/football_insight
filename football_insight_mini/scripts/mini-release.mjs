#!/usr/bin/env bun
// 微信小程序唯一构建/发布入口：清理继承变量 → 生产构建 → 校验实际产物 → 上传。
// 用法：
//   bun run build:mp-weixin            # build：生产构建 + 校验（MINI_REVIEW_SKIP=1 时不登记版本）
//   bun run mp:release -- [--robot 2] [--desc 文案]   # upload：Git 门禁 + 登记版本号 + 构建 + 上传
//   bun run mp:preview -- [--robot 2] [--desc 文案]   # preview：只上传已有生产产物为预览
//   bun run verify:mp-release          # verify：只校验现有产物，无远程调用
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import {
  loadProductionEnvironment,
  parseMiniCiArgs,
  verifyUploadGitState,
} from "./mini-release-guards.mjs";
import { verifyMiniProgramApiBase } from "./verify-mp-api-base.mjs";

const defaultProjectRoot = fileURLToPath(new URL("..", import.meta.url));

function validateArguments(command, args) {
  if (!["build", "upload", "preview", "verify"].includes(command)) {
    throw new Error("用法: bun scripts/mini-release.mjs build|upload|preview|verify [--robot N] [--desc 文案]");
  }
  const forwarded = args[0] === "--" ? args.slice(1) : [...args];
  if ((command === "build" || command === "verify") && forwarded.length) {
    throw new Error(`${command} 不接受额外参数，生产 mode 与 API 由统一入口管理。`);
  }
  if (command === "upload" || command === "preview") {
    parseMiniCiArgs(forwarded);
  }
  return forwarded;
}

async function executeStep(cmd, options) {
  const child = Bun.spawn(cmd, { ...options, stdin: "inherit", stdout: "inherit", stderr: "inherit" });
  return await child.exited;
}

export async function runMiniRelease(command, args = [], {
  projectRoot = defaultProjectRoot,
  env = process.env,
  runStep = executeStep,
  checkGitState = verifyUploadGitState,
} = {}) {
  const forwarded = validateArguments(command, args);
  const production = loadProductionEnvironment(projectRoot, env);
  // 上传必须走登记库，不能被上一次离线验证留下的 MINI_REVIEW_SKIP=1 意外跳过。
  if (command === "upload") production.MINI_REVIEW_SKIP = "0";
  console.log(`[mini-release] Bun ${Bun.version}; mode=production; API=${production.VITE_API_BASE_URL}`);

  if (command === "upload") {
    console.log("[mini-release] 检查 Git：main / clean / origin/main 已推送");
    await checkGitState(projectRoot, { fetch: true });
  }

  // 临时空 envDir：阻止 Vite 自动读取 .env.local / .env.production.local 等本地覆盖层，
  // 客户端变量只来自上面显式解析的 .env.production；finally 里清理。
  const releaseEnvDir = mkdtempSync(path.join(tmpdir(), "football-release-env-"));
  try {
    production.FOOTBALL_RELEASE_ENV_DIR = releaseEnvDir;

    const run = async (label, script, scriptArgs = []) => {
      console.log(`[mini-release] ${label}`);
      const exitCode = await runStep([process.execPath, "--no-env-file", "--bun", path.join(projectRoot, script), ...scriptArgs], {
        cwd: projectRoot, env: production,
      });
      if (exitCode !== 0) throw new Error(`${label}失败（退出码 ${exitCode}），后续步骤未执行。`);
    };

    if (command === "build" || command === "upload") {
      await run("检查 import 边界", "scripts/check-import-boundaries.mjs");
      await run("同步审核版本", "scripts/sync-manifest-version.mjs");
      if (command === "upload") {
        // 新分配版本会修改 manifest/version 常量：必须先提交并推送，再重新执行发布，
        // 保证最终上传到微信的源码与 origin/main 精确对应。
        await checkGitState(projectRoot, { fetch: false });
      }
      await run("构建生产小程序", "node_modules/@dcloudio/vite-plugin-uni/bin/uni.js", ["build", "-p", "mp-weixin", "--mode", "production"]);
      await run("检查小程序组件注册", "scripts/verify-mp-component-registrations.mjs");
    }
    const actualApi = verifyMiniProgramApiBase(projectRoot);
    console.log(`[mini-release] 编译包 API 校验通过: ${actualApi}`);
    if (command === "upload" || command === "preview") {
      if (command === "upload") {
        // 构建后再次确认源代码没有被任何步骤改动，且远端 main 仍与当前 HEAD 一致。
        await checkGitState(projectRoot, { fetch: true });
      }
      await run(command === "upload" ? "上传微信开发版本" : "生成微信预览", "scripts/mini-ci.mjs", [command, ...forwarded]);
    }
  } finally {
    rmSync(releaseEnvDir, { recursive: true, force: true });
  }
}

if (import.meta.main) {
  runMiniRelease(process.argv[2], process.argv.slice(3)).catch((error) => {
    console.error(`[mini-release] ${error instanceof Error ? error.message : String(error)}`);
    process.exitCode = 1;
  });
}
