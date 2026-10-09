#!/usr/bin/env bun
// 小程序发布的纯校验共享模块：统一入口（mini-release.mjs）与底层 CI（mini-ci.mjs）都从这里复用，
// 避免两个脚本互相 import 形成循环。import 本模块没有任何上传/登记副作用。
import { existsSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { PRODUCTION_API_BASE_URL } from "./verify-mp-api-base.mjs";

const defaultProjectRoot = fileURLToPath(new URL("..", import.meta.url));

function requireBun() {
  if (!process.versions.bun) throw new Error("小程序构建和上传必须使用 Bun，请运行 bun run mp:release。");
}

// 生产 API 必须精确为 https://match.oryjk.cn/api/v1，不允许本地地址、其他域名或带尾斜杠的变体。
export function normalizeReleaseApiBase(value) {
  if (typeof value !== "string" || !value.trim()) {
    throw new Error(".env.production 必须显式配置 VITE_API_BASE_URL，禁止回退到本地地址。");
  }
  const base = value.trim().replace(/\/+$/, "");
  if (base !== PRODUCTION_API_BASE_URL) {
    throw new Error(`生产 VITE_API_BASE_URL 必须精确为 ${PRODUCTION_API_BASE_URL}，实际为 ${value}。`);
  }
  return base;
}

// 清理继承的 VITE_* / UNI_*（外层 `bun run` 可能提前注入 .env.development，--no-env-file 不会清除已继承的值），
// 只读取显式的 .env.production 重建客户端变量，固定 NODE_ENV=production。
// 保留 PATH、代理和 MINI_CI_* / MINI_REVIEW_* 等非客户端配置；客户端变量不含私钥/API key。
export function loadProductionEnvironment(projectRoot = defaultProjectRoot, inherited = process.env) {
  requireBun();
  const envFile = path.join(projectRoot, ".env.production");
  if (!existsSync(envFile)) throw new Error("缺少 .env.production，已阻止构建/上传。");

  const clean = Object.fromEntries(Object.entries(inherited)
    .filter(([key, value]) => value !== undefined && !key.startsWith("VITE_") && !key.startsWith("UNI_")));
  clean.NODE_ENV = "production";
  const result = Bun.spawnSync([
    process.execPath, "--no-env-file", `--env-file=${envFile}`, "--eval",
    'console.log(JSON.stringify(Object.fromEntries(Object.entries(process.env).filter(([key]) => key.startsWith("VITE_")))))',
  ], { cwd: projectRoot, env: clean, stdout: "pipe", stderr: "pipe" });
  if (result.exitCode !== 0) throw new Error("使用 Bun 读取 .env.production 失败，已阻止构建/上传。");
  let production;
  try {
    production = JSON.parse(result.stdout.toString());
  } catch {
    throw new Error("无法解析生产构建环境，已阻止构建/上传。");
  }
  const apiBase = normalizeReleaseApiBase(production.VITE_API_BASE_URL);
  return { ...clean, ...production, VITE_API_BASE_URL: apiBase };
}

function executeGit(args, cwd) {
  const result = Bun.spawnSync(["git", ...args], { cwd, stdout: "pipe", stderr: "pipe" });
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString(),
    stderr: result.stderr.toString(),
  };
}

function gitValue(runGit, args, cwd, label) {
  const result = runGit(args, cwd);
  if (result.exitCode !== 0) {
    const detail = String(result.stderr || result.stdout || "").trim();
    throw new Error(`${label}失败${detail ? `：${detail}` : ""}，已阻止小程序上传。`);
  }
  return String(result.stdout || "").trim();
}

// 正式上传的 Git 门禁：main 分支、工作区干净、（可选 fetch 成功后）HEAD 与 origin/main 完全一致。
export function verifyUploadGitState(projectRoot = defaultProjectRoot, {
  runGit = executeGit,
  fetch = true,
} = {}) {
  const repoRoot = gitValue(runGit, ["rev-parse", "--show-toplevel"], projectRoot, "读取 Git 仓库");
  const branch = gitValue(runGit, ["branch", "--show-current"], repoRoot, "读取当前分支");
  if (branch !== "main") {
    throw new Error(`小程序上传只允许从 main 分支执行；当前分支为 ${branch || "detached HEAD"}。`);
  }

  const status = gitValue(runGit, ["status", "--porcelain", "--untracked-files=normal"], repoRoot, "检查 Git 工作区");
  if (status) {
    throw new Error("检测到工作区不干净，已阻止小程序上传；请先 commit 全部改动并 push 到 origin/main 后重新执行 mp:release。");
  }

  if (fetch) {
    gitValue(runGit, ["fetch", "origin", "main", "--quiet"], repoRoot, "刷新 origin/main");
  }
  const head = gitValue(runGit, ["rev-parse", "HEAD"], repoRoot, "读取本地 HEAD");
  const originMain = gitValue(runGit, ["rev-parse", "origin/main"], repoRoot, "读取 origin/main");
  if (head !== originMain) {
    throw new Error("小程序上传要求本地 HEAD 与 origin/main 完全一致；请先 commit + push（或同步远端 main）后重新执行 mp:release。");
  }
  return { repoRoot, head };
}

// upload/preview 只接受 --robot（1 日常开发版 / 2 体验版专用线，默认 1）和 --desc 文案。
export function parseMiniCiArgs(args) {
  const forwarded = args[0] === "--" ? args.slice(1) : [...args];
  const seen = new Set();
  let robot = 1;
  let desc;
  for (let i = 0; i < forwarded.length; i += 2) {
    const name = forwarded[i];
    const value = forwarded[i + 1];
    if (!["--robot", "--desc"].includes(name) || seen.has(name) || !value || value.startsWith("--")) {
      throw new Error("上传参数无效，只支持 --robot N 和 --desc 文案，且不能重复。");
    }
    if (name === "--robot") {
      if (!/^\d+$/.test(value) || ![1, 2].includes(Number(value))) {
        throw new Error("--robot 只支持 1（日常开发版，默认）或 2（体验版专用线）。");
      }
      robot = Number(value);
    } else {
      desc = value;
    }
    seen.add(name);
  }
  return { robot, desc };
}
