import { afterEach, describe, expect, test } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import {
  loadProductionEnvironment,
  normalizeReleaseApiBase,
  verifyUploadGitState,
} from "./mini-release-guards.mjs";
import { runMiniRelease } from "./mini-release.mjs";

const productionApi = "https://match.oryjk.cn/api/v1";
const localApi = "http://127.0.0.1:8092/api/v1";
const roots = [];

function fixture(api = productionApi) {
  const root = mkdtempSync(path.join(tmpdir(), "football-mini-release-test-"));
  roots.push(root);
  writeFileSync(path.join(root, ".env.production"), `VITE_API_BASE_URL=${productionApi}\nVITE_PRODUCTION_ONLY=yes\n`);
  writeFileSync(path.join(root, ".env.development"), `VITE_API_BASE_URL=${localApi}\n`);
  writeFileSync(path.join(root, ".env.local"), `VITE_API_BASE_URL=${localApi}\nVITE_LOCAL_ONLY=pollution\n`);
  const dist = path.join(root, "dist/build/mp-weixin");
  mkdirSync(path.join(dist, "config"), { recursive: true });
  writeFileSync(path.join(dist, "app.json"), "{}");
  writeFileSync(path.join(dist, "config/apiBase.js"), `exports.API_BASE_URL = ${JSON.stringify(api)};`);
  return root;
}

function recordingRunner(failScript) {
  const steps = [];
  return {
    steps,
    runStep: async (cmd, options) => {
      steps.push({ cmd, env: options.env });
      return cmd.some((arg) => arg.endsWith(failScript || "<no failure>")) ? 1 : 0;
    },
  };
}

function scriptNames(steps) {
  return steps.map(({ cmd }) => path.basename(cmd[3]));
}

function gitRunner(responses) {
  const calls = [];
  return {
    calls,
    runGit: (args) => {
      calls.push(args);
      const key = args.join(" ");
      const value = responses[key];
      if (!value) return { exitCode: 1, stdout: "", stderr: `unexpected git command: ${key}` };
      return { exitCode: value.exitCode ?? 0, stdout: value.stdout ?? "", stderr: value.stderr ?? "" };
    },
  };
}

afterEach(() => {
  for (const root of roots.splice(0)) rmSync(root, { recursive: true, force: true });
});

describe("production release environment", () => {
  test("replaces inherited development variables with the explicit production file", () => {
    const root = fixture();
    const inherited = {
      ...process.env,
      NODE_ENV: "development",
      VITE_API_BASE_URL: localApi,
      VITE_DEV_ONLY: "must-not-leak",
      UNI_INPUT_DIR: "/wrong/source",
      MINI_CI_PRIVATE_KEY_PATH: "/test/private.key",
    };
    const env = loadProductionEnvironment(root, inherited);
    expect(env.NODE_ENV).toBe("production");
    expect(env.VITE_API_BASE_URL).toBe(productionApi);
    expect(env.VITE_PRODUCTION_ONLY).toBe("yes");
    expect(env.VITE_DEV_ONLY).toBeUndefined();
    expect(env.UNI_INPUT_DIR).toBeUndefined();
    expect(env.MINI_CI_PRIVATE_KEY_PATH).toBe("/test/private.key");
    expect(inherited.VITE_API_BASE_URL).toBe(localApi);
  });

  test("ignores .env.local / .env.development overlays and keeps only the production file", () => {
    const root = fixture();
    const env = loadProductionEnvironment(root, { PATH: process.env.PATH });
    expect(env.VITE_API_BASE_URL).toBe(productionApi);
    expect(env.VITE_LOCAL_ONLY).toBeUndefined();
  });

  test("fails without an explicit production file instead of falling back to localhost", () => {
    const root = fixture();
    rmSync(path.join(root, ".env.production"));
    expect(() => loadProductionEnvironment(root)).toThrow(".env.production");
  });

  test("fails when the production file omits the API or uses a non-production endpoint", () => {
    const root = fixture();
    writeFileSync(path.join(root, ".env.production"), "VITE_OTHER=value\n");
    expect(() => loadProductionEnvironment(root)).toThrow("VITE_API_BASE_URL");
    writeFileSync(path.join(root, ".env.production"), `VITE_API_BASE_URL=${localApi}\n`);
    expect(() => loadProductionEnvironment(root)).toThrow(productionApi);
  });

  test("accepts only the exact production endpoint (trim and trailing slash tolerated)", () => {
    expect(normalizeReleaseApiBase(` ${productionApi}/ `)).toBe(productionApi);
    for (const api of [
      localApi,
      "http://match.oryjk.cn/api/v1",
      "https://example.com/api/v1",
      "https://match.oryjk.cn/api/v1?debug=1",
      "https://match.oryjk.cn/api",
      "",
      undefined,
    ]) {
      expect(() => normalizeReleaseApiBase(api)).toThrow();
    }
  });
});

describe("upload git guard", () => {
  const baseResponses = {
    "rev-parse --show-toplevel": { stdout: "/repo\n" },
    "branch --show-current": { stdout: "main\n" },
    "status --porcelain --untracked-files=normal": { stdout: "" },
    "fetch origin main --quiet": {},
    "rev-parse HEAD": { stdout: "abc123\n" },
    "rev-parse origin/main": { stdout: "abc123\n" },
  };

  test("accepts only a clean main checkout whose HEAD equals fetched origin/main", () => {
    const runner = gitRunner(baseResponses);
    const state = verifyUploadGitState("/repo/football_insight_mini", { runGit: runner.runGit });
    expect(state).toEqual({ repoRoot: "/repo", head: "abc123" });
    expect(runner.calls.map((args) => args.join(" "))).toContain("fetch origin main --quiet");
  });

  test("rejects feature branches and detached HEAD", () => {
    const feature = gitRunner({ ...baseResponses, "branch --show-current": { stdout: "feat/x\n" } });
    expect(() => verifyUploadGitState("/repo/football_insight_mini", { runGit: feature.runGit })).toThrow("main 分支");

    const detached = gitRunner({ ...baseResponses, "branch --show-current": { stdout: "" } });
    expect(() => verifyUploadGitState("/repo/football_insight_mini", { runGit: detached.runGit })).toThrow("detached HEAD");
  });

  test("rejects dirty worktrees, unpushed commits and fetch failures", () => {
    const dirty = gitRunner({
      ...baseResponses,
      "status --porcelain --untracked-files=normal": { stdout: " M football_insight_mini/src/a.ts\n" },
    });
    expect(() => verifyUploadGitState("/repo/football_insight_mini", { runGit: dirty.runGit })).toThrow("工作区不干净");

    const unpushed = gitRunner({ ...baseResponses, "rev-parse origin/main": { stdout: "remote456\n" } });
    expect(() => verifyUploadGitState("/repo/football_insight_mini", { runGit: unpushed.runGit })).toThrow("origin/main");

    const fetchFailed = gitRunner({ ...baseResponses, "fetch origin main --quiet": { exitCode: 1, stderr: "network down" } });
    expect(() => verifyUploadGitState("/repo/football_insight_mini", { runGit: fetchFailed.runGit })).toThrow("刷新 origin/main");
  });

  test("skips fetch when disabled but still compares HEAD with the recorded origin/main", () => {
    const runner = gitRunner(baseResponses);
    verifyUploadGitState("/repo/football_insight_mini", { runGit: runner.runGit, fetch: false });
    expect(runner.calls.map((args) => args.join(" "))).not.toContain("fetch origin main --quiet");
  });
});

describe("Bun release pipeline", () => {
  test("upload: boundaries → allocate once → git recheck → build → components → api → git/fetch recheck → sdk upload", async () => {
    const root = fixture();
    const runner = recordingRunner();
    const gitChecks = [];
    await runMiniRelease("upload", ["--", "--robot", "2", "--desc", "发布说明 with spaces"], {
      projectRoot: root,
      env: { ...process.env, MINI_REVIEW_SKIP: "1", VITE_API_BASE_URL: localApi },
      checkGitState: (_root, options) => gitChecks.push(options ?? {}),
      ...runner,
    });
    expect(scriptNames(runner.steps)).toEqual([
      "check-import-boundaries.mjs",
      "sync-manifest-version.mjs",
      "uni.js",
      "verify-mp-component-registrations.mjs",
      "mini-ci.mjs",
      "set-review-status.mjs",
    ]);
    for (const { cmd, env } of runner.steps) {
      expect(cmd.slice(0, 3)).toEqual([process.execPath, "--no-env-file", "--bun"]);
      expect(env.NODE_ENV).toBe("production");
      expect(env.VITE_API_BASE_URL).toBe(productionApi);
      // 外层 MINI_REVIEW_SKIP=1 的 upload 仍必须登记。
      expect(env.MINI_REVIEW_SKIP).toBe("0");
      expect(env.VITE_DEV_ONLY).toBeUndefined();
    }
    expect(runner.steps[2].cmd.slice(4)).toEqual(["build", "-p", "mp-weixin", "--mode", "production"]);
    expect(runner.steps[4].cmd.slice(4)).toEqual(["upload", "--robot", "2", "--desc", "发布说明 with spaces"]);
    // 默认上传成功后标记为已审核通过，不再需要手动切换。
    expect(runner.steps[5].cmd.slice(4)).toEqual(["approved"]);
    expect(gitChecks).toEqual([{ fetch: true }, { fetch: false }, { fetch: true }]);
  });

  test("upload --reviewing keeps the version in review: no approval step and the flag never reaches mini-ci", async () => {
    const root = fixture();
    const runner = recordingRunner();
    await runMiniRelease("upload", ["--robot", "1", "--reviewing", "--desc", "提审版"], {
      projectRoot: root,
      checkGitState: () => undefined,
      ...runner,
    });
    expect(scriptNames(runner.steps)).not.toContain("set-review-status.mjs");
    expect(runner.steps.at(-1).cmd.slice(4)).toEqual(["upload", "--robot", "1", "--desc", "提审版"]);
  });

  test("a failed sdk upload never marks the version approved, so a rerun reuses the in-review version", async () => {
    const root = fixture();
    const runner = recordingRunner("mini-ci.mjs");
    await expect(
      runMiniRelease("upload", [], { projectRoot: root, checkGitState: () => undefined, ...runner }),
    ).rejects.toThrow();
    expect(scriptNames(runner.steps).at(-1)).toBe("mini-ci.mjs");
    expect(scriptNames(runner.steps)).not.toContain("set-review-status.mjs");
  });

  test("a failed approval step is reported after the upload instead of being swallowed", async () => {
    const root = fixture();
    const runner = recordingRunner("set-review-status.mjs");
    await expect(
      runMiniRelease("upload", [], { projectRoot: root, checkGitState: () => undefined, ...runner }),
    ).rejects.toThrow("标记版本为已审核通过");
  });

  test("preview and build never touch the review status", async () => {
    const root = fixture();
    const preview = recordingRunner();
    await runMiniRelease("preview", [], { projectRoot: root, ...preview });
    expect(scriptNames(preview.steps)).not.toContain("set-review-status.mjs");
    const build = recordingRunner();
    await runMiniRelease("build", [], { projectRoot: root, env: { ...process.env, MINI_REVIEW_SKIP: "1" }, ...build });
    expect(scriptNames(build.steps)).not.toContain("set-review-status.mjs");
  });

  test("points Vite at an isolated empty envDir during the run and cleans it up afterwards", async () => {
    const root = fixture();
    const runner = recordingRunner();
    const envDirs = new Set();
    const runStep = async (cmd, options) => {
      const dir = options.env.FOOTBALL_RELEASE_ENV_DIR;
      expect(dir).toBeString();
      expect(existsSync(dir)).toBe(true);
      envDirs.add(dir);
      return runner.runStep(cmd, options);
    };
    await runMiniRelease("build", [], {
      projectRoot: root,
      env: { ...process.env, MINI_REVIEW_SKIP: "1" },
      runStep,
      checkGitState: () => {
        throw new Error("build 不应触发上传 Git 门禁");
      },
    });
    expect(envDirs.size).toBe(1);
    for (const dir of envDirs) expect(existsSync(dir)).toBe(false);
  });

  test("stops after version allocation when generated version files still need commit and push", async () => {
    const root = fixture();
    const runner = recordingRunner();
    let checks = 0;
    const checkGitState = () => {
      checks += 1;
      if (checks === 2) throw new Error("工作区不干净，请先 commit + push 后重新执行 mp:release");
    };

    await expect(runMiniRelease("upload", [], { projectRoot: root, checkGitState, ...runner })).rejects.toThrow("commit + push");
    expect(scriptNames(runner.steps)).toEqual(["check-import-boundaries.mjs", "sync-manifest-version.mjs"]);
  });

  test("build keeps the offline skip flag, skips the git gate and never uploads", async () => {
    const root = fixture();
    const runner = recordingRunner();
    let gitCalled = 0;
    await runMiniRelease("build", [], {
      projectRoot: root,
      env: { ...process.env, MINI_REVIEW_SKIP: "1" },
      checkGitState: () => {
        gitCalled += 1;
      },
      ...runner,
    });
    expect(scriptNames(runner.steps)).toEqual([
      "check-import-boundaries.mjs",
      "sync-manifest-version.mjs",
      "uni.js",
      "verify-mp-component-registrations.mjs",
    ]);
    expect(runner.steps[1].env.MINI_REVIEW_SKIP).toBe("1");
    expect(gitCalled).toBe(0);
  });

  test("never reaches upload when the compiled API is local", async () => {
    const root = fixture(localApi);
    const runner = recordingRunner();
    await expect(
      runMiniRelease("upload", [], { projectRoot: root, checkGitState: () => undefined, ...runner }),
    ).rejects.toThrow();
    expect(scriptNames(runner.steps)).not.toContain("mini-ci.mjs");
  });

  for (const failed of [
    "check-import-boundaries.mjs",
    "sync-manifest-version.mjs",
    "uni.js",
    "verify-mp-component-registrations.mjs",
  ]) {
    test(`stops on ${failed} failure instead of uploading stale output`, async () => {
      const root = fixture();
      const runner = recordingRunner(failed);
      await expect(
        runMiniRelease("upload", [], { projectRoot: root, checkGitState: () => undefined, ...runner }),
      ).rejects.toThrow();
      expect(scriptNames(runner.steps).at(-1)).toBe(failed);
      expect(scriptNames(runner.steps)).not.toContain("mini-ci.mjs");
    });
  }

  test("preview checks the existing bundle without allocating, rebuilding or git gates", async () => {
    const root = fixture();
    const runner = recordingRunner();
    let gitCalled = 0;
    await runMiniRelease("preview", ["--desc", "预览"], {
      projectRoot: root,
      checkGitState: () => {
        gitCalled += 1;
      },
      ...runner,
    });
    expect(scriptNames(runner.steps)).toEqual(["mini-ci.mjs"]);
    expect(runner.steps[0].cmd.slice(4)).toEqual(["preview", "--desc", "预览"]);
    expect(gitCalled).toBe(0);

    const badRoot = fixture(localApi);
    const blocked = recordingRunner();
    await expect(runMiniRelease("preview", [], { projectRoot: badRoot, ...blocked })).rejects.toThrow();
    expect(blocked.steps).toEqual([]);
  });

  test("verify is read-only: no build, no version registration, no upload", async () => {
    const root = fixture();
    const runner = recordingRunner();
    const before = readFileSync(path.join(root, ".env.production"), "utf8");
    await runMiniRelease("verify", [], { projectRoot: root, ...runner });
    expect(runner.steps).toEqual([]);
    expect(readFileSync(path.join(root, ".env.production"), "utf8")).toBe(before);

    const badRoot = fixture(localApi);
    await expect(runMiniRelease("verify", [], { projectRoot: badRoot, ...recordingRunner() })).rejects.toThrow();
  });

  test("rejects mode overrides and invalid robot values before any side effects", async () => {
    const root = fixture();
    for (const [command, args] of [
      ["upload", ["--mode", "development"]],
      ["upload", ["--robot", "0"]],
      ["upload", ["--robot", "3"]],
      ["upload", ["--robot"]],
      ["upload", ["--desc"]],
      ["upload", ["--reviewing", "--reviewing"]],
      ["build", ["--reviewing"]],
      ["preview", ["--reviewing"]],
      ["build", ["--robot", "1"]],
      ["verify", ["--desc", "x"]],
    ]) {
      const runner = recordingRunner();
      await expect(
        runMiniRelease(command, args, { projectRoot: root, checkGitState: () => undefined, ...runner }),
      ).rejects.toThrow();
      expect(runner.steps).toEqual([]);
    }
  });
});

describe("package.json release wiring", () => {
  const packageJson = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));

  test("routes mp build/upload/preview/verify through the unified entry", () => {
    expect(packageJson.scripts["build:mp-weixin"]).toBe("bun --no-env-file scripts/mini-release.mjs build");
    expect(packageJson.scripts["mp:release"]).toBe("bun --no-env-file scripts/mini-release.mjs upload");
    expect(packageJson.scripts["mp:preview"]).toBe("bun --no-env-file scripts/mini-release.mjs preview");
    expect(packageJson.scripts["verify:mp-release"]).toBe("bun --no-env-file scripts/mini-release.mjs verify");
    expect(packageJson.scripts["test:mp-release"]).toBe("bun test scripts/mini-release.test.mjs scripts/set-review-status.test.mjs");
    expect(packageJson.scripts["prebuild:mp-weixin"]).toBeUndefined();
    expect(packageJson.scripts["mp:upload"]).toBeUndefined();
  });
});
