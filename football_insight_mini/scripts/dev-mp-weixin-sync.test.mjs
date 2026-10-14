import { afterEach, describe, expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { ContextChangedError, createSyncController, parseInterval } from "./dev-mp-weixin-sync.mjs";

const roots = [];
const processes = [];
const descendants = new Set();
const sourceScript = new URL("./dev-mp-weixin-sync.mjs", import.meta.url);

function git(cwd, args) {
  const result = Bun.spawnSync(["git", ...args], { cwd, env: { ...process.env, GIT_TERMINAL_PROMPT: "0" } });
  if (result.exitCode !== 0) throw new Error(result.stderr.toString());
  return result.stdout.toString();
}

function write(root, name, contents) {
  const target = path.join(root, name);
  mkdirSync(path.dirname(target), { recursive: true });
  writeFileSync(target, contents);
}

function commit(root, message = "fixture") {
  git(root, ["add", "."]);
  git(root, ["-c", "user.name=Mini Sync Test", "-c", "user.email=mini-sync@example.invalid", "-c", "commit.gpgsign=false", "commit", "-qm", message]);
}

function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), "football-mini-sync-"));
  roots.push(root);
  const remote = path.join(root, "remote.git");
  const writer = path.join(root, "local233");
  const reader = path.join(root, "preview-computer");
  git(root, ["init", "--bare", "--initial-branch=main", remote]);
  git(root, ["clone", remote, writer]);
  write(writer, ".gitignore", "node_modules/\ndist/\n");
  write(writer, "football_insight_mini/package.json", JSON.stringify({ scripts: { "dev:mp-weixin": "bun fake-dev.mjs" } }));
  write(writer, "football_insight_mini/src/page.txt", "initial");
  write(writer, "football_insight_mini/scripts/dev-mp-weixin-sync.mjs", readFileSync(sourceScript));
  write(writer, "backend.txt", "initial backend");
  commit(writer);
  git(writer, ["push", "-qu", "origin", "main"]);
  git(root, ["clone", remote, reader]);
  const projectRoot = path.join(reader, "football_insight_mini");
  const events = [];
  const logs = [];
  const hooks = {
    projectRoot,
    git: async (args) => git(projectRoot, args),
    stopDev: async () => { events.push("stop"); },
    startDev: async () => { events.push("start"); },
    installDependencies: async () => { events.push("install"); },
    log: (message) => logs.push(message),
  };
  const push = (name = "football_insight_mini/src/page.txt", contents = "updated") => {
    write(writer, name, contents);
    commit(writer, contents);
    git(writer, ["push", "-q"]);
  };
  return { root, remote, writer, reader, projectRoot, events, logs, hooks, push };
}

afterEach(async () => {
  for (const proc of processes.splice(0)) {
    if (proc.exitCode === null) proc.kill("SIGTERM");
    await proc.exited;
  }
  for (const pid of descendants) {
    try { process.kill(pid, "SIGKILL"); } catch {}
  }
  descendants.clear();
  for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true });
});

describe("sync interval", () => {
  test("defaults to 10 seconds and supports args and environment", () => {
    expect(parseInterval([], {})).toBe(10);
    expect(parseInterval([], { MINI_SYNC_INTERVAL_SECONDS: "20" })).toBe(20);
    expect(parseInterval(["--", "--interval", "5"], { MINI_SYNC_INTERVAL_SECONDS: "20" })).toBe(5);
    for (const args of [["--interval", "0"], ["--interval", "-1"], ["--interval", "1.5"], ["--interval", ""], ["--interval", "86401"], ["--bad"], ["--interval"]]) {
      expect(() => parseInterval(args, {})).toThrow();
    }
  });
});

describe("real Git synchronization", () => {
  test("fast forwards pushed mini changes, restarts once and ignores unchanged polls", async () => {
    const f = fixture();
    const controller = await createSyncController(f.hooks);
    await controller.tick();
    expect(f.events).toEqual([]);
    f.push();
    await controller.tick();
    expect(readFileSync(path.join(f.projectRoot, "src/page.txt"), "utf8")).toBe("updated");
    expect(git(f.reader, ["rev-parse", "HEAD"])).toBe(git(f.writer, ["rev-parse", "HEAD"]));
    expect(f.events).toEqual(["stop", "start"]);
    await controller.tick();
    expect(f.events).toEqual(["stop", "start"]);
  });

  test("updates other monorepo files without restarting mini compilation", async () => {
    const f = fixture();
    const controller = await createSyncController(f.hooks);
    f.push("backend.txt", "updated backend");
    await controller.tick();
    expect(readFileSync(path.join(f.reader, "backend.txt"), "utf8")).toBe("updated backend");
    expect(f.events).toEqual([]);
  });

  for (const localFile of ["football_insight_mini/src/page.txt", "backend.txt", "untracked.txt"]) {
    test(`protects local ${localFile} and resumes after it is cleared`, async () => {
      const f = fixture();
      const controller = await createSyncController(f.hooks);
      const before = git(f.reader, ["rev-parse", "HEAD"]);
      f.push();
      write(f.reader, localFile, "local work");
      await controller.tick();
      expect(git(f.reader, ["rev-parse", "HEAD"])).toBe(before);
      expect(readFileSync(path.join(f.reader, localFile), "utf8")).toBe("local work");
      expect(f.events).toEqual([]);
      if (localFile === "untracked.txt") rmSync(path.join(f.reader, localFile));
      else git(f.reader, ["restore", localFile]);
      await controller.tick();
      expect(f.events).toEqual(["stop", "start"]);
    });
  }

  test("ignored local configuration and build outputs do not block synchronization", async () => {
    const f = fixture();
    git(f.reader, ["config", "core.excludesFile", path.join(f.root, "excludes")]);
    write(f.root, "excludes", ".env.local\n");
    write(f.reader, "football_insight_mini/.env.local", "LOCAL_ONLY=yes");
    write(f.reader, "football_insight_mini/dist/dev/mp-weixin/app.js", "old output");
    const controller = await createSyncController(f.hooks);
    f.push();
    await controller.tick();
    expect(f.events).toEqual(["stop", "start"]);
    expect(readFileSync(path.join(f.projectRoot, ".env.local"), "utf8")).toBe("LOCAL_ONLY=yes");
  });

  test("protects staged changes even when merge.autoStash is enabled", async () => {
    const f = fixture();
    git(f.reader, ["config", "merge.autoStash", "true"]);
    write(f.reader, "backend.txt", "staged local work");
    git(f.reader, ["add", "backend.txt"]);
    f.push();
    const controller = await createSyncController(f.hooks);
    await controller.tick();
    expect(git(f.reader, ["diff", "--cached"])).toContain("staged local work");
    expect(git(f.reader, ["stash", "list"])).toBe("");
    expect(f.events).toEqual([]);
  });

  for (const diverged of [false, true]) {
    test(`protects ${diverged ? "diverged" : "ahead"} local commits`, async () => {
      const f = fixture();
      write(f.reader, "backend.txt", "local commit");
      commit(f.reader);
      const before = git(f.reader, ["rev-parse", "HEAD"]);
      if (diverged) f.push();
      const controller = await createSyncController(f.hooks);
      await controller.tick();
      expect(git(f.reader, ["rev-parse", "HEAD"])).toBe(before);
      expect(f.events).toEqual([]);
      expect(f.logs.join("\n")).toContain("领先提交或与远端分叉");
    });
  }

  test("rechecks local work after stopping compilation and keeps it intact", async () => {
    const f = fixture();
    const controller = await createSyncController({ ...f.hooks, stopDev: async () => {
      f.events.push("stop");
      write(f.reader, "backend.txt", "edit during sync");
    } });
    f.push();
    const before = git(f.reader, ["rev-parse", "HEAD"]);
    await controller.tick();
    expect(git(f.reader, ["rev-parse", "HEAD"])).toBe(before);
    expect(readFileSync(path.join(f.reader, "backend.txt"), "utf8")).toBe("edit during sync");
    expect(f.events).toEqual(["stop", "start"]);
  });

  test("dependency failure pauses compilation and retries before restarting", async () => {
    const f = fixture();
    let failing = true;
    const controller = await createSyncController({ ...f.hooks, installDependencies: async () => {
      f.events.push("install");
      if (failing) throw new Error("offline install");
    } });
    f.push("football_insight_mini/bun.lock", "fixture lock");
    await expect(controller.tick()).rejects.toThrow("offline install");
    expect(f.events).toEqual(["stop", "install"]);
    failing = false;
    await controller.tick();
    expect(f.events).toEqual(["stop", "install", "install", "start"]);
  });

  test("network failure leaves compilation running and the next poll recovers", async () => {
    const f = fixture();
    const controller = await createSyncController(f.hooks);
    git(f.reader, ["remote", "set-url", "origin", path.join(f.root, "missing.git")]);
    await expect(controller.tick()).rejects.toThrow();
    expect(f.events).toEqual([]);
    git(f.reader, ["remote", "set-url", "origin", f.remote]);
    f.push();
    await controller.tick();
    expect(f.events).toEqual(["stop", "start"]);
  });

  test("stops when the branch or upstream changes", async () => {
    const f = fixture();
    const controller = await createSyncController(f.hooks);
    git(f.reader, ["checkout", "-qb", "other", "--track", "origin/main"]);
    await expect(controller.tick()).rejects.toBeInstanceOf(ContextChangedError);
    expect(f.events).toEqual([]);
  });

  test("rejects missing upstream and detached HEAD before starting compilation", async () => {
    const f = fixture();
    git(f.reader, ["branch", "--unset-upstream"]);
    await expect(createSyncController(f.hooks)).rejects.toThrow();
    git(f.reader, ["checkout", "--detach", "-q"]);
    await expect(createSyncController(f.hooks)).rejects.toBeInstanceOf(ContextChangedError);
  });
});

async function waitFor(predicate, timeout = 10000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (predicate()) return;
    await delay(30);
  }
  throw new Error("Timed out waiting for process state");
}

function alive(pid) {
  try { process.kill(pid, 0); return true; } catch { return false; }
}

for (const ignoreTerm of [false, true]) {
test.skipIf(process.platform === "win32")(`CLI polls pushed code and Ctrl+C cleans up compiler descendants${ignoreTerm ? " even when a child ignores SIGTERM" : ""}`, async () => {
  const f = fixture();
  const pidLog = path.join(f.root, "compiler-pids.txt");
  write(f.writer, "football_insight_mini/fake-dev.mjs", `
import { spawn } from 'node:child_process';
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
const child = spawn(process.execPath, ['-e', ${JSON.stringify(`${ignoreTerm ? "process.on('SIGTERM', () => {});" : ""} process.stdout.write('ready'); setInterval(() => {}, 1000);`)}], { stdio: ['ignore', 'pipe', 'ignore'] });
child.stdout.once('data', () => {
appendFileSync(process.env.MINI_SYNC_TEST_LOG, process.pid + ',' + child.pid + '\\n');
mkdirSync('dist/dev/mp-weixin', { recursive: true });
writeFileSync('dist/dev/mp-weixin/page.txt', readFileSync('src/page.txt'));
});
setInterval(() => {}, 1000);
`);
  commit(f.writer);
  git(f.writer, ["push", "-q"]);
  git(f.reader, ["pull", "--ff-only", "-q"]);
  mkdirSync(path.join(f.projectRoot, "node_modules"));
  const proc = Bun.spawn([process.execPath, "scripts/dev-mp-weixin-sync.mjs", "--interval", "1"], {
    cwd: f.projectRoot, env: { ...process.env, MINI_SYNC_TEST_LOG: pidLog }, stdout: "pipe", stderr: "pipe",
  });
  processes.push(proc);
  const output = new Response(proc.stdout).text();
  const errors = new Response(proc.stderr).text();
  const dist = path.join(f.projectRoot, "dist/dev/mp-weixin/page.txt");
  await waitFor(() => existsSync(dist) && readFileSync(dist, "utf8") === "initial");
  f.push();
  await waitFor(() => readFileSync(dist, "utf8") === "updated");
  const pids = () => {
    const values = readFileSync(pidLog, "utf8").trim().split(/[\n,]/).map(Number);
    for (const pid of values) descendants.add(pid);
    return values;
  };
  await waitFor(() => pids().slice(0, 2).every((pid) => !alive(pid)));
  proc.kill("SIGINT");
  expect(await proc.exited).toBe(0);
  await waitFor(() => pids().every((pid) => !alive(pid)));
  expect(await output).toContain("已快进更新");
  expect(await errors).not.toContain("Error:");
}, 20000);
}
