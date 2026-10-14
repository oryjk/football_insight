#!/usr/bin/env bun
import { spawn } from "node:child_process";
import { existsSync, realpathSync } from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { setTimeout as delay } from "node:timers/promises";

const defaultProjectRoot = fileURLToPath(new URL("..", import.meta.url));
const prefix = "[mini-sync]";

export function parseInterval(args, env = process.env) {
  const options = args[0] === "--" ? args.slice(1) : args;
  if (options.length && (options.length !== 2 || options[0] !== "--interval")) {
    throw new Error("用法：bun run dev:mp-weixin:sync [--interval 秒数]");
  }
  const value = options[1] ?? env.MINI_SYNC_INTERVAL_SECONDS ?? "10";
  const seconds = Number(value);
  if (!/^\d+$/.test(String(value)) || !Number.isSafeInteger(seconds) || seconds < 1 || seconds > 86400) {
    throw new Error("同步间隔必须为 1–86400 的整数秒数。");
  }
  return seconds;
}

export class ContextChangedError extends Error {}

// Git 与编译生命周期分开，便于用真实的临时仓库验证保护规则。
export async function createSyncController({ projectRoot, git, stopDev, startDev, installDependencies, log }) {
  const repoRoot = realpathSync((await git(["rev-parse", "--show-toplevel"])).trim());
  const projectPath = path.relative(repoRoot, realpathSync(projectRoot)).split(path.sep).join("/");
  if (!projectPath || projectPath.startsWith("../") || path.isAbsolute(projectPath)) {
    throw new Error("小程序目录必须位于 Git 仓库内。");
  }
  const readContext = async () => {
    const branch = (await git(["branch", "--show-current"])).trim();
    if (!branch) throw new ContextChangedError("当前为 detached HEAD，请切换到配置了远端上游的分支。");
    const upstream = (await git(["rev-parse", "--symbolic-full-name", "@{upstream}"])).trim();
    const remote = (await git(["config", "--get", `branch.${branch}.remote`])).trim();
    if (!upstream.startsWith("refs/remotes/") || !remote || remote === ".") {
      throw new Error("当前分支需要配置远端上游（例如 git branch --set-upstream-to=origin/main）。");
    }
    return { branch, upstream, remote };
  };
  const context = await readContext();
  let needsInstall = false;
  let needsRestart = false;
  let previousNotice;
  const notice = (message) => {
    if (message !== previousNotice) log(message);
    previousNotice = message;
  };
  const checkContext = async () => {
    let current;
    try {
      current = await readContext();
    } catch (error) {
      throw new ContextChangedError(`分支或上游已变更，请重新启动同步命令。${error.message}`);
    }
    if (Object.keys(context).some((key) => context[key] !== current[key])) {
      throw new ContextChangedError("分支或上游已变更，请重新启动同步命令。");
    }
  };
  const isDirty = async () => Boolean((await git(["status", "--porcelain", "--untracked-files=normal"])).trim());
  const resumeDev = async () => {
    if (needsInstall) {
      log("依赖已变化，执行 bun install --frozen-lockfile；失败后下一轮重试。");
      await installDependencies();
      needsInstall = false;
    }
    if (needsRestart) {
      await startDev();
      needsRestart = false;
    }
  };
  return {
    context,
    async tick() {
      await checkContext();
      await resumeDev();
      if (await isDirty()) {
        notice("工作区存在未提交修改或未跟踪文件，暂缓同步；清理后自动恢复。");
        return;
      }
      await git(["fetch", "--quiet", context.remote]);
      await checkContext();
      if (await isDirty()) {
        notice("拉取检查期间工作区发生变化，暂缓同步。");
        return;
      }
      const before = (await git(["rev-parse", "HEAD"])).trim();
      const target = (await git(["rev-parse", context.upstream])).trim();
      if (before === target) {
        notice(`已与 ${context.upstream.replace("refs/remotes/", "")} 同步，等待新提交。`);
        return;
      }
      const counts = (await git(["rev-list", "--left-right", "--count", `${before}...${target}`])).trim().split(/\s+/).map(Number);
      if (counts[0] !== 0) {
        notice("本地有领先提交或与远端分叉，暂缓同步；请手动处理 Git 状态。");
        return;
      }
      const changed = (await git(["diff", "--no-relative", "--name-only", "-z", before, target, "--", `:(top,literal)${projectPath}`])).split("\0").filter(Boolean);
      const dependencyChanged = changed.some((file) => ["package.json", "bun.lock", "bun.lockb"].includes(file.slice(projectPath.length + 1)));
      if (changed.length) {
        needsRestart = true;
        await stopDev();
      }
      try {
        // 停编译后再次检查；不使用 stash/reset，也不允许用户配置自动 stash。
        await checkContext();
        if (await isDirty() || (await git(["rev-parse", "HEAD"])).trim() !== before) {
          notice("更新前本地代码发生变化，暂缓同步。");
          await resumeDev();
          return;
        }
        await git(["-c", "merge.autoStash=false", "merge", "--ff-only", "--no-edit", target]);
      } catch (error) {
        if (!(error instanceof ContextChangedError)) await resumeDev();
        throw error;
      }
      log(`已快进更新 ${before.slice(0, 7)} → ${target.slice(0, 7)}${changed.length ? "，重新编译小程序。" : "，小程序目录无变化。"}`);
      previousNotice = undefined;
      needsInstall = dependencyChanged;
      await resumeDev();
    },
  };
}

function signalTree(child, signal) {
  if (!child.pid) return;
  if (process.platform === "win32") {
    if (child.exitCode !== null || child.signalCode !== null) return;
    // Windows 没有 POSIX 进程组；连同 bun run 启动的编译器子进程一起退出。
    return new Promise((resolve) => {
      const killer = spawn("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" });
      killer.once("error", () => { child.kill(); resolve(); });
      killer.once("exit", resolve);
    });
  }
  try { process.kill(-child.pid, signal); } catch (error) {
    if (error.code !== "ESRCH") throw error;
  }
}

async function stopProcess(child) {
  if (!child?.pid) return;
  const exited = child.exitCode !== null || child.signalCode !== null
    ? Promise.resolve()
    : new Promise((resolve) => child.once("exit", resolve));
  await signalTree(child, "SIGTERM");
  if (process.platform !== "win32") {
    const groupAlive = () => {
      try { process.kill(-child.pid, 0); return true; } catch (error) {
        if (error.code === "ESRCH") return false;
        throw error;
      }
    };
    // bun run 的主进程可能先退出，仍要给忽略 SIGTERM 的后代升级为 SIGKILL。
    const deadline = Date.now() + 3000;
    while (groupAlive() && Date.now() < deadline) await delay(50);
    if (groupAlive()) await signalTree(child, "SIGKILL");
  }
  await exited;
}

export async function runMiniDevSync({ projectRoot = defaultProjectRoot, args = process.argv.slice(2), env = process.env } = {}) {
  const seconds = parseInterval(args, env);
  const log = (message) => console.log(`${prefix} ${message}`);
  const abort = new AbortController();
  const commands = new Set();
  let watcher;
  let stopping = false;
  let watcherFailed = false;
  const shutdown = () => {
    if (stopping) return;
    stopping = true;
    abort.abort();
    for (const child of commands) void stopProcess(child).catch((error) => log(`停止子进程失败：${error.message}`));
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
  const launch = (cmd, stdio) => {
    if (stopping) throw new Error("同步已停止。");
    return spawn(cmd[0], cmd.slice(1), { cwd: projectRoot, env, stdio, detached: process.platform !== "win32" });
  };
  const execute = (cmd, { capture = false, timeout = 30000 } = {}) => new Promise((resolve, reject) => {
    const child = launch(cmd, capture ? ["ignore", "pipe", "pipe"] : ["ignore", "inherit", "inherit"]);
    commands.add(child);
    let stdout = "";
    let stderr = "";
    let timedOut = false;
    child.stdout?.on("data", (chunk) => { stdout += chunk; });
    child.stderr?.on("data", (chunk) => { stderr += chunk; });
    const timer = setTimeout(() => { timedOut = true; void signalTree(child, "SIGKILL"); }, timeout);
    const cleanup = () => { clearTimeout(timer); commands.delete(child); };
    child.once("error", (error) => { cleanup(); reject(error); });
    child.once("close", (code) => {
      cleanup();
      if (code === 0 && !timedOut) resolve(stdout);
      else reject(new Error(`${cmd[0]} ${cmd.slice(1).join(" ")} ${timedOut ? "超时" : `失败 (${code})`}${stderr.trim() ? `：${stderr.trim()}` : ""}`));
    });
  });
  const git = (args) => execute(["git", ...args], { capture: true });
  const stopDev = async () => {
    const current = watcher;
    watcher = undefined; // 主动停止不算编译器异常退出。
    await stopProcess(current);
  };
  const startDev = async () => {
    log("启动 bun run dev:mp-weixin；微信开发者工具打开 dist/dev/mp-weixin。");
    const child = launch([process.execPath, "run", "dev:mp-weixin"], "inherit");
    watcher = child;
    const failed = (message) => {
      if (watcher === child && !stopping) {
        log(message);
        watcherFailed = true;
        shutdown();
      }
    };
    child.once("error", (error) => failed(`开发编译启动失败：${error.message}`));
    child.once("exit", (code, signal) => failed(`开发编译已退出 (${code ?? signal})，同步一并停止。`));
  };
  const installDependencies = () => execute([process.execPath, "install", "--frozen-lockfile"], { timeout: 120000 });
  try {
    const controller = await createSyncController({ projectRoot, git, stopDev, startDev, installDependencies, log });
    log(`跟踪 ${controller.context.branch} → ${controller.context.upstream.replace("refs/remotes/", "")}；检查间隔 ${seconds}s；Ctrl+C 停止。`);
    if (!existsSync(path.join(projectRoot, "node_modules"))) await installDependencies();
    await startDev();
    while (!stopping) {
      try {
        await controller.tick();
      } catch (error) {
        if (stopping) break;
        if (error instanceof ContextChangedError) throw error;
        log(`${error.message}；${seconds}s 后重试。`);
      }
      if (!stopping) await delay(seconds * 1000, undefined, { signal: abort.signal }).catch((error) => {
        if (error.name !== "AbortError") throw error;
      });
    }
    if (watcherFailed) throw new Error("开发编译异常退出，请修复后重新启动。");
  } finally {
    shutdown();
    await Promise.all([stopDev(), ...[...commands].map(stopProcess)]);
    process.off("SIGINT", shutdown);
    process.off("SIGTERM", shutdown);
  }
}

if (import.meta.main) {
  if (process.argv.slice(2).includes("--help")) {
    console.log("用法：bun run dev:mp-weixin:sync [--interval 秒数]\n默认每 10 秒同步当前分支的远端上游；也可设置 MINI_SYNC_INTERVAL_SECONDS。");
  } else {
    // 禁止 Git 交互输入；未配置认证时让轮询失败并提示，不阻塞开发编译。
    runMiniDevSync({ env: { ...process.env, GIT_TERMINAL_PROMPT: "0" } }).catch((error) => {
      console.error(`${prefix} ${error.message}`);
      process.exitCode = 1;
    });
  }
}
