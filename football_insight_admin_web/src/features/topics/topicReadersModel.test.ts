import { expect, test } from "bun:test";
import { createApi } from "../../api";
import { ApiClient } from "../../apiClient";
import { firstReadTime, readerName, topicOptionLabel } from "./topicReadersModel";

test("reader without nickname gets a readable fallback", () => {
  expect(readerName({ display_name: null })).toBe("未设置昵称");
  expect(readerName({ display_name: "  " })).toBe("未设置昵称");
  expect(readerName({ display_name: "球迷小王" })).toBe("球迷小王");
});

test("first read time is shown in Beijing time to the minute", () => {
  expect(firstReadTime("2026-10-17T16:05:59Z")).toBe("2026-10-18 00:05");
  expect(firstReadTime("bad")).toBe("时间异常");
});

test("topic option shows date, title and reader count", () => {
  expect(
    topicOptionLabel({ title: "标题", edition_date: "2026-10-17", reader_count: 3 }),
  ).toBe("2026-10-17 · 标题（3 人）");
});

test("reader requests use the admin paths with the bearer token", async () => {
  const seen: { url: string; auth: string | null }[] = [];
  const server = Bun.serve({
    port: 0,
    fetch(request) {
      seen.push({
        url: new URL(request.url).pathname,
        auth: request.headers.get("authorization"),
      });
      return Response.json({ items: [] });
    },
  });
  try {
    const api = createApi(new ApiClient(`http://127.0.0.1:${server.port}/api/v1/admin`));
    await api.topicReaderTopics("tok");
    await api.topicReaders("tok", 77680, "a/b");
    expect(seen).toEqual([
      { url: "/api/v1/admin/daily-topics/topics", auth: "Bearer tok" },
      { url: "/api/v1/admin/daily-topics/77680/a%2Fb/readers", auth: "Bearer tok" },
    ]);
  } finally {
    server.stop(true);
  }
});
