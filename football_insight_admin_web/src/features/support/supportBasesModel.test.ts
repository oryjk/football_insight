import { expect, test } from "bun:test";
import type { SupportBaseMatch } from "../../api";
import { createApi } from "../../api";
import { ApiClient } from "../../apiClient";
import { canSave, displayedCount, matchTitle, parseBase } from "./supportBasesModel";

const match: SupportBaseMatch = {
  match_id: 1,
  round_number: 28,
  match_date: "2026-10-18",
  match_time: "20:00",
  status: "scheduled",
  home_team: { team_id: 11, team_name: "主", real_count: 0, base: 2, displayed_count: 2 },
  away_team: { team_id: 22, team_name: "客", real_count: 3, base: 20, displayed_count: 23 },
};

test("base input accepts whole numbers in range only", () => {
  expect(parseBase("0")).toBe(0);
  expect(parseBase(" 20 ")).toBe(20);
  expect(parseBase("100000")).toBe(100000);
  for (const bad of ["", "-1", "1.5", "abc", "100001", "1e3", "1234567"])
    expect(parseBase(bad)).toBeNull();
});

test("displayed count is real plus base, null for invalid input", () => {
  expect(displayedCount(3, "20")).toBe(23);
  expect(displayedCount(3, "x")).toBeNull();
});

test("save needs valid and changed values", () => {
  expect(canSave(match, "2", "20")).toBe(false);
  expect(canSave(match, "2", "21")).toBe(true);
  expect(canSave(match, "", "20")).toBe(false);
  expect(canSave(match, "2", "-3")).toBe(false);
});

test("match title shows round and kickoff", () => {
  expect(matchTitle(match)).toBe("第 28 轮 · 2026-10-18 20:00");
});

test("support base requests use the admin paths and PUT body", async () => {
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
      return Response.json({ items: [] });
    },
  });
  try {
    const api = createApi(new ApiClient(`http://127.0.0.1:${server.port}/api/v1/admin`));
    await api.supportBaseMatches("tok");
    await api.setSupportBases("tok", 288795, 2, 20);
    expect(seen[0]).toMatchObject({ path: "/api/v1/admin/support/matches", method: "GET", auth: "Bearer tok" });
    expect(seen[1].path).toBe("/api/v1/admin/support/matches/288795/bases");
    expect(seen[1].method).toBe("PUT");
    expect(JSON.parse(seen[1].body)).toEqual({ home_base: 2, away_base: 20 });
  } finally {
    server.stop(true);
  }
});
