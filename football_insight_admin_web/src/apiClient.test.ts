import { afterAll, expect, test } from "bun:test";
import { ApiClient, ApiError } from "./apiClient";
const server = Bun.serve({
  port: 0,
  fetch(req) {
    const path = new URL(req.url).pathname;
    if (path === "/empty") return new Response(null, { status: 204 });
    if (path === "/unauthorized")
      return new Response("expired", { status: 401 });
    if (path === "/conflict-English")
      return new Response("product key already exists", { status: 409 });
    if (path === "/conflict")
      return new Response("商品业务键已存在", { status: 409 });
    if (path === "/internal")
      return new Response("SQL password=private", { status: 500 });
    if (path === "/html")
      return new Response("<html>proxy error</html>", { status: 502 });
    return Response.json({
      authorization: req.headers.get("Authorization"),
    });
  },
});
// Network boundary tests use a real local HTTP server, not a mocked fetch.
afterAll(() => server.stop(true));
const base = `http://127.0.0.1:${server.port}`;
test("真实请求携带 Bearer，并正确解析 JSON", async () => {
  const value = await new ApiClient(base).request<{ authorization: string }>(
    "/ok",
    "test-token",
  );
  expect(value.authorization).toBe("Bearer test-token");
});
test("204 无正文无需 JSON 解析", async () =>
  expect(
    await new ApiClient(base).request("/empty", "token", { method: "DELETE" }),
  ).toBeUndefined());
test("401 失效回调绑定请求 token", async () => {
  let invalidated = "";
  const api = new ApiClient(base, (token) => (invalidated = token));
  try {
    await api.request("/unauthorized", "old-token");
  } catch (error) {
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(401);
  }
  expect(invalidated).toBe("old-token");
});
test("409 中文操作提示，内部和代理错误不暴露", async () => {
  await expect(
    new ApiClient(base).request("/conflict", "token"),
  ).rejects.toThrow("该等级与金额的商品已存在");
  for (const path of ["/internal", "/html"]) {
    try {
      await new ApiClient(base).request(path, "token");
      throw new Error("should fail");
    } catch (error) {
      expect(error).toBeInstanceOf(ApiError);
      expect((error as Error).message).not.toContain("password");
      expect((error as Error).message).not.toContain("<html>");
    }
  }
});
test("取消请求保留 AbortError 语义", async () => {
  const controller = new AbortController();
  controller.abort();
  try {
    await new ApiClient(base).request("/ok", "token", {
      signal: controller.signal,
    });
    throw new Error("should fail");
  } catch (error) {
    expect((error as Error).name).toBe("AbortError");
  }
});

test("真实后端英文冲突正文转换为中文操作提示", async () => {
  await expect(
    new ApiClient(base).request("/conflict-English", "token"),
  ).rejects.toThrow("该等级与金额的商品已存在");
});
