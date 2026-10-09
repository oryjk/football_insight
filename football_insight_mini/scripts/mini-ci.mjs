#!/usr/bin/env bun
// 小程序 CI 上传/预览：基于官方 miniprogram-ci。
// 通常不直接调用，统一走 scripts/mini-release.mjs（bun run mp:release / mp:preview）；
// 即便直接调用，本脚本也会在加载微信 SDK 前重复执行产物 API 校验与上传 Git 门禁。
// 私钥默认放在项目根 private.<appid>.key，可用 MINI_CI_PRIVATE_KEY_PATH 覆盖。
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";

import JSON5 from "json5";

import { loadProductionEnvironment, parseMiniCiArgs, verifyUploadGitState } from "./mini-release-guards.mjs";
import { verifyMiniProgramApiBase } from "./verify-mp-api-base.mjs";

const projectRoot = path.resolve(new URL("..", import.meta.url).pathname);

async function main() {
  const command = process.argv[2];
  if (command !== "upload" && command !== "preview") {
    console.error("用法: bun run mp:release [-- --robot 2 --desc 文案]");
    console.error("   或: bun run mp:preview [-- --robot 2 --desc 文案]");
    console.error("（mp:release 内部先执行完整构建，再以 upload 方式调用本脚本）");
    process.exit(1);
  }
  const { robot, desc: descArg } = parseMiniCiArgs(process.argv.slice(3));

  const manifest = JSON5.parse(readFileSync(path.join(projectRoot, "src", "manifest.json"), "utf8"));
  const appid = manifest["mp-weixin"]?.appid;
  const privateKeyPath =
    process.env.MINI_CI_PRIVATE_KEY_PATH || path.join(projectRoot, `private.${appid}.key`);

  if (!appid) {
    console.error("[mini-ci] manifest.json 缺少 mp-weixin.appid");
    process.exit(1);
  }
  if (!existsSync(privateKeyPath)) {
    console.error(`[mini-ci] 未找到上传私钥: ${privateKeyPath}`);
    console.error("       请从微信公众平台（开发管理 → 开发设置 → 小程序代码上传）下载私钥，");
    console.error("       放到项目根目录 private.<appid>.key，或用 MINI_CI_PRIVATE_KEY_PATH 指定路径。");
    process.exit(1);
  }

  const distPath = path.join(projectRoot, "dist", "build", "mp-weixin");
  if (!existsSync(path.join(distPath, "app.json"))) {
    console.error(`[mini-ci] 构建产物缺失: ${distPath}/app.json`);
    console.error("       先运行 bun run build:mp-weixin（或直接用 mp:release 一条龙）。");
    process.exit(1);
  }

  // 即便直接调用本上传脚本，也必须先确认产物实际连接生产 API。
  // 在加载微信 SDK 前失败，避免错误构建触发任何上传副作用。
  loadProductionEnvironment(projectRoot);
  const apiBase = verifyMiniProgramApiBase(projectRoot);
  console.log(`[mini-ci] 上传前 API 校验通过: ${apiBase}`);
  if (command === "upload") {
    verifyUploadGitState(projectRoot);
    console.log("[mini-ci] Git 校验通过: main / clean / origin/main 已推送");
  }

  const { default: ci } = await import("miniprogram-ci");
  const project = new ci.Project({
    appid,
    type: "miniProgram",
    projectPath: distPath,
    privateKeyPath,
    ignores: ["node_modules/**/*"],
  });

  const desc = descArg || `v${manifest.versionName} CI 上传`;
  const setting = { es6: false, minifyJS: false, minifyWXML: false, minifyWXSS: false };

  if (command === "upload") {
    const result = await ci.upload({ project, version: manifest.versionName, desc, robot, setting });
    console.log(`[mini-ci] 上传完成: 版本 ${manifest.versionName} robot=${robot}`);
    if (result?.subPackageInfo) {
      console.log("[mini-ci] 分包信息:", JSON.stringify(result.subPackageInfo));
    }
    return;
  }

  const qrcodeOutputDest = path.join(projectRoot, "dist", "preview-qrcode.jpg");
  await ci.preview({ project, desc, robot, setting, qrcodeFormat: "image", qrcodeOutputDest });
  console.log(`[mini-ci] 预览已生成: ${qrcodeOutputDest}（robot=${robot}）`);
}

if (import.meta.main) {
  main().catch((error) => {
    console.error(`[mini-ci] ${process.argv[2]} 失败:`, error instanceof Error ? error.message : error);
    process.exit(1);
  });
}
