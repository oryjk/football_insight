#!/usr/bin/env node
// 小程序 CI 上传/预览：基于官方 miniprogram-ci。
// 用法：
//   bun run mp:release  -- [--robot 2] [--desc 文案]   # 构建 + 分配版本号 + 上传
//   bun run mp:preview  -- [--robot 2] [--desc 文案]   # 上传当前构建产物为预览
// 私钥默认放在项目根 private.<appid>.key，可用 MINI_CI_PRIVATE_KEY_PATH 覆盖。
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";

import JSON5 from "json5";
import ci from "miniprogram-ci";

const projectRoot = path.resolve(new URL("..", import.meta.url).pathname);
const command = process.argv[2];
const extraArgs = process.argv.slice(3);

if (command !== "upload" && command !== "preview") {
  console.error("用法: bun run mp:release [-- --robot 2 --desc 文案]");
  console.error("   或: bun run mp:preview [-- --robot 2 --desc 文案]");
  console.error("（mp:release 内部先执行完整构建，再以 upload 方式调用本脚本）");
  process.exit(1);
}

function argValue(name) {
  const index = extraArgs.indexOf(name);
  return index >= 0 && extraArgs[index + 1] ? extraArgs[index + 1] : undefined;
}

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

const robot = Number(argValue("--robot")) || 1;
const desc = argValue("--desc") || `v${manifest.versionName} CI 上传`;
const setting = { es6: false, minifyJS: false, minifyWXML: false, minifyWXSS: false };

async function main() {
  const project = new ci.Project({
    appid,
    type: "miniProgram",
    projectPath: distPath,
    privateKeyPath,
    ignores: ["node_modules/**/*"],
  });

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

main().catch((error) => {
  console.error(`[mini-ci] ${command} 失败:`, error instanceof Error ? error.message : error);
  process.exit(1);
});
