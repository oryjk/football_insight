import { expect, test } from "bun:test";
import { createApi } from "../../api";
import { ApiClient } from "../../apiClient";
import {
  DEFAULT_BRAND_SUBTITLE,
  MAX_SUBTITLE_CHARS,
  canSaveSubtitle,
  validateSubtitle,
} from "./brandModel";

test("subtitle counts Unicode characters and allows blank to restore the default", () => {
  expect(validateSubtitle("")).toBeNull();
  expect(validateSubtitle("  成都蓉城球迷的数据主场  ")).toBeNull();
  expect(validateSubtitle("蓉".repeat(MAX_SUBTITLE_CHARS))).toBeNull();
  expect(validateSubtitle("蓉".repeat(MAX_SUBTITLE_CHARS + 1))).not.toBeNull();
  expect(validateSubtitle("a\nb")).not.toBeNull();
  // An astral character is one character, matching the backend's chars().count().
  expect(validateSubtitle("😀".repeat(MAX_SUBTITLE_CHARS))).toBeNull();
});

test("save is only enabled for a valid, changed value", () => {
  expect(canSaveSubtitle("新文案", "旧文案")).toBe(true);
  expect(canSaveSubtitle(" 旧文案 ", "旧文案")).toBe(false);
  expect(canSaveSubtitle("蓉".repeat(MAX_SUBTITLE_CHARS + 1), "")).toBe(false);
  expect(canSaveSubtitle("", "旧文案")).toBe(true);
  expect(DEFAULT_BRAND_SUBTITLE).toBe("最懂球迷的工具");
});

test("brand API reads and writes the subtitle with the original key", async () => {
  const seen: { path: string; method: string; body: string; auth: string | null }[] = [];
  const server = Bun.serve({
    port: 0,
    async fetch(request) {
      seen.push({
        path: new URL(request.url).pathname,
        method: request.method,
        body: await request.text(),
        auth: request.headers.get("authorization"),
      });
      return Response.json({ subtitle: "新文案" });
    },
  });
  try {
    const api = createApi(new ApiClient(`http://127.0.0.1:${server.port}/api/v1/admin`));
    await api.brandSubtitle("tok");
    const saved = await api.setBrandSubtitle("tok", "新文案");
    expect(seen[0]).toMatchObject({ path: "/api/v1/admin/brand/subtitle", method: "GET", auth: "Bearer tok" });
    expect(seen[1].method).toBe("PUT");
    expect(JSON.parse(seen[1].body)).toEqual({ subtitle: "新文案" });
    expect(saved.subtitle).toBe("新文案");
  } finally {
    server.stop(true);
  }
});
