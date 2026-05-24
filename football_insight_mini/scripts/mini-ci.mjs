#!/usr/bin/env node
import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import JSON5 from "json5";
import { syncManifestVersion } from "./sync-manifest-version.mjs";

const command = process.argv[2];
const extraArgs = process.argv.slice(3);
const cliPath = "/Users/carlwang/.local/share/mini-program-ci-cli/index.mjs";
const projectRoot = path.resolve(new URL("..", import.meta.url).pathname);
const ciEnvPath = path.join(projectRoot, ".env.ci.local");
const ciConfig = parseEnvFile(ciEnvPath);
const reviewInternalToken = "football_insight_mini";
const reviewApiBaseUrl =
  process.env.MINI_REVIEW_API_BASE_URL || ciConfig.MINI_REVIEW_API_BASE_URL || "http://127.0.0.1:3003";
const reviewProjectCode =
  process.env.MINI_REVIEW_PROJECT_CODE || ciConfig.MINI_REVIEW_PROJECT_CODE || "football_insight_mini";

if (!command) {
  console.error("用法: bun run mp:preview [-- --robot 2 --desc 文案]");
  console.error("   或: bun run mp:upload [-- --robot 2 --version 1.0.1 --desc 文案]");
  process.exit(1);
}

function incrementPatchVersion(version) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(String(version || "").trim());
  if (!match) {
    throw new Error(`无法按 +1 规则计算版本号，当前版本格式必须是 x.y.z，实际收到: ${version}`);
  }
  const [, major, minor, patch] = match;
  return `${major}.${minor}.${Number(patch) + 1}`;
}

function parseEnvFile(filePath) {
  try {
    const content = readFileSync(filePath, "utf8");
    const env = {};
    for (const rawLine of content.split(/\r?\n/)) {
      const line = rawLine.trim();
      if (!line || line.startsWith("#")) continue;
      const index = line.indexOf("=");
      if (index < 0) continue;
      const key = line.slice(0, index).trim();
      const value = line.slice(index + 1).trim();
      env[key] = value;
    }
    return env;
  } catch {
    return {};
  }
}

function resolveUploadVersion() {
  const manifest = JSON5.parse(readFileSync(path.join(projectRoot, "src", "manifest.json"), "utf8"));
  const versionArgIndex = extraArgs.indexOf("--version");
  if (versionArgIndex >= 0 && extraArgs[versionArgIndex + 1]) {
    return extraArgs[versionArgIndex + 1];
  }

  const envPath = path.join(projectRoot, ".env.ci.local");
  const content = readFileSync(envPath, "utf8");
  const parsed = content ? content.split(/\r?\n/) : [];
  const versionLine = parsed.find((line) => line.startsWith("MINI_PROGRAM_VERSION="));
  const baseline = versionLine ? versionLine.slice("MINI_PROGRAM_VERSION=".length) : manifest.versionName;
  return incrementPatchVersion(baseline);
}

async function postReviewRecord(version) {
  const response = await fetch(`${reviewApiBaseUrl.replace(/\/$/, "")}/api/admin/reviews`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Internal-Token": reviewInternalToken,
    },
    body: JSON.stringify({
      project_code: reviewProjectCode,
      version,
      is_reviewing: true,
      status_text: "正在审核",
      submitted_at: new Date().toISOString(),
      reviewed_at: new Date().toISOString(),
      remark: "mp:upload 自动创建",
    }),
  });

  if (!response.ok) {
    throw new Error(`review api failed: ${response.status}`);
  }
}

async function main() {
  const uploadVersion = command === "upload" ? resolveUploadVersion() : null;
  const versionArgIndex = extraArgs.indexOf("--version");
  const forwardedArgs =
    command === "upload" && uploadVersion && versionArgIndex < 0 ? [...extraArgs, "--version", uploadVersion] : extraArgs;

  syncManifestVersion();

  const child = spawn("node", [cliPath, command, projectRoot, ...forwardedArgs], {
    stdio: "inherit",
  });

  child.on("error", (error) => {
    console.error(error);
    process.exit(1);
  });

  const exitCode = await new Promise((resolve) => {
    child.on("exit", (code) => resolve(code ?? 1));
  });

  if (exitCode !== 0) {
    process.exit(exitCode);
  }

  if (command === "upload") {
    syncManifestVersion();
    await postReviewRecord(uploadVersion);
  }

  process.exit(0);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
