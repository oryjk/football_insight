#!/usr/bin/env bun
// 向后端 mini-review 登记库设置当前 manifest 版本的审核状态（静态 X-Api-Key 通道）。
// 由统一入口 scripts/mini-release.mjs 在上传成功后调用；import 本模块无副作用。
import { readFileSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import JSON5 from "json5";

import { miniReviewConfig, parseVersionCode } from "./sync-manifest-version.mjs";

const defaultProjectRoot = fileURLToPath(new URL("..", import.meta.url));

export async function setReviewStatus({
  config = miniReviewConfig(),
  version,
  isReviewing,
  fetchImpl = fetch,
} = {}) {
  if (!config.apiKey) {
    throw new Error("设置审核状态需要在 .env.ci.local（或环境变量）配置 MINI_REVIEW_API_KEY");
  }
  // 复用版本号格式校验：非 x.y.z 一律拒绝，不向登记库发请求。
  parseVersionCode(version);

  const response = await fetchImpl(`${config.baseUrl}/mini-review/review-status`, {
    method: "PUT",
    headers: { "Content-Type": "application/json", "X-Api-Key": config.apiKey },
    body: JSON.stringify({ project_code: config.projectCode, version: String(version), is_reviewing: Boolean(isReviewing) }),
  });
  if (!response.ok) {
    throw new Error(
      `版本 ${version} 设置审核状态失败: HTTP ${response.status} ${await response.text().catch(() => "")}。` +
        "可在小程序「我的 → 设置」里手动切换。",
    );
  }
  return response.json().catch(() => ({}));
}

if (import.meta.main) {
  try {
    const target = process.argv[2];
    if (!["approved", "reviewing"].includes(target)) {
      throw new Error("用法: bun scripts/set-review-status.mjs approved|reviewing");
    }
    const manifest = JSON5.parse(readFileSync(path.join(defaultProjectRoot, "src", "manifest.json"), "utf8"));
    const isReviewing = target === "reviewing";
    await setReviewStatus({ version: manifest.versionName, isReviewing });
    console.log(`[set-review-status] 版本 ${manifest.versionName} 已标记为${isReviewing ? "审核中" : "已审核通过"}`);
  } catch (error) {
    console.error(`[set-review-status] ${error instanceof Error ? error.message : error}`);
    process.exit(1);
  }
}
