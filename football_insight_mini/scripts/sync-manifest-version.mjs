#!/usr/bin/env node
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";

import JSON5 from "json5";

const projectRoot = path.resolve(new URL("..", import.meta.url).pathname);
const envPath = path.join(projectRoot, ".env.ci.local");
const manifestPath = path.join(projectRoot, "src", "manifest.json");
const generatedVersionPath = path.join(projectRoot, "src", "config", "generatedMiniProgramVersion.ts");

// 审核版本登记走本项目 Rust 后端（match.oryjk.cn）的 mini-review 登记库，密钥放在 .env.ci.local（不入 git）。
const DEFAULT_MINI_REVIEW_API_URL = "https://match.oryjk.cn/api/v1";
const DEFAULT_MINI_REVIEW_PROJECT_CODE = "football_insight_mini";

function parseEnvFile(filePath) {
  if (!existsSync(filePath)) return {};

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
}

export function miniReviewConfig(env = parseEnvFile(envPath)) {
  return {
    apiKey: process.env.MINI_REVIEW_API_KEY || env.MINI_REVIEW_API_KEY || "",
    baseUrl: (process.env.MINI_REVIEW_API_URL || env.MINI_REVIEW_API_URL || DEFAULT_MINI_REVIEW_API_URL).replace(/\/+$/, ""),
    projectCode: process.env.MINI_REVIEW_PROJECT_CODE || env.MINI_REVIEW_PROJECT_CODE || DEFAULT_MINI_REVIEW_PROJECT_CODE,
    skip: /^(1|true)$/i.test(String(process.env.MINI_REVIEW_SKIP ?? env.MINI_REVIEW_SKIP ?? "")),
  };
}

// 向 Go 后端登记并取得本次构建应使用的版本号（登记库是唯一权威，多台构建机结果一致）：
// 库内最新版本仍在审核中时复用；已出审核则在库内最大版本基础上 +0.0.1 新建并标记审核中；
// 仅当库内无任何记录时才以本地 manifest 为起点。删库重置后版本号随库回落。
// MINI_REVIEW_SKIP=1 时返回 null（离线本地构建，不登记）。
export async function allocateReviewVersion({ explicitVersion = "", currentVersion = "" } = {}) {
  const config = miniReviewConfig();
  if (config.skip) return null;
  if (!config.apiKey) {
    throw new Error(
      "生产构建需要在 .env.ci.local（或环境变量）配置 MINI_REVIEW_API_KEY 以登记审核版本；纯本地离线构建可设 MINI_REVIEW_SKIP=1",
    );
  }

  const response = await fetch(`${config.baseUrl}/mini-review/allocate`, {
    method: "POST",
    headers: { "Content-Type": "application/json", "X-Api-Key": config.apiKey },
    body: JSON.stringify({
      project_code: config.projectCode,
      current_version: String(currentVersion || ""),
      ...(explicitVersion ? { version: String(explicitVersion) } : {}),
    }),
  });
  if (!response.ok) {
    throw new Error(`审核版本登记接口请求失败: HTTP ${response.status} ${await response.text().catch(() => "")}`);
  }
  const payload = await response.json();
  if (!payload?.version) {
    throw new Error(`审核版本登记接口响应无效: ${JSON.stringify(payload).slice(0, 200)}`);
  }
  return payload.version;
}

function parseVersionCode(versionName) {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(String(versionName || "").trim());
  if (!match) {
    throw new Error(`MINI_PROGRAM_VERSION 格式必须是 x.y.z，实际收到: ${versionName}`);
  }
  const [, major, minor, patch] = match;
  return String(Number(major) * 10000 + Number(minor) * 100 + Number(patch));
}

export async function syncManifestVersion(options = {}) {
  const env = parseEnvFile(envPath);
  const manifest = JSON5.parse(readFileSync(manifestPath, "utf8"));
  const explicitVersion = options.versionName || process.env.MINI_PROGRAM_VERSION || env.MINI_PROGRAM_VERSION || "";

  // 版本号统一由审核登记接口决定（显式 MINI_PROGRAM_VERSION 作为指定版本传入）；
  // 跳过登记的离线构建退回 manifest 当前值。
  const versionName =
    (await allocateReviewVersion({ explicitVersion, currentVersion: String(manifest.versionName || "") })) ||
    explicitVersion ||
    manifest.versionName;
  if (!versionName) {
    return null;
  }

  const versionCode = parseVersionCode(versionName);

  const manifestChanged = !(manifest.versionName === versionName && String(manifest.versionCode) === versionCode);
  if (manifestChanged) {
    manifest.versionName = versionName;
    manifest.versionCode = versionCode;
    writeFileSync(`${manifestPath}`, `${JSON.stringify(manifest, null, 2)}\n`, "utf-8");
  }

  writeFileSync(
    generatedVersionPath,
    `export const MINI_PROGRAM_VERSION = ${JSON.stringify(versionName)};\nexport const MINI_PROGRAM_VERSION_CODE = ${JSON.stringify(versionCode)};\n`,
    "utf-8",
  );

  return { versionName, versionCode, changed: manifestChanged };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const result = await syncManifestVersion();
    if (result) {
      console.log(
        `[sync-manifest-version] ${result.changed ? "updated" : "kept"} manifest versionName=${result.versionName} versionCode=${result.versionCode}`,
      );
    }
  } catch (error) {
    console.error(`[sync-manifest-version] ${error instanceof Error ? error.message : error}`);
    process.exit(1);
  }
}
