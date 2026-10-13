import { afterAll, expect, test } from "bun:test";
import { createApi } from "../../api";
import { ApiClient } from "../../apiClient";
let statusReply: unknown = { needs_setup: true, username: "superadmin" };
let observed: {
  method: string;
  path: string;
  authorization: string | null;
  body: unknown;
} | null = null;
const server = Bun.serve({
  port: 0,
  async fetch(req) {
    const body =
      req.method === "POST"
        ? ((await req.json()) as { password: string })
        : null;
    observed = {
      method: req.method,
      path: new URL(req.url).pathname,
      authorization: req.headers.get("Authorization"),
      body,
    };
    if (req.method === "GET") return Response.json(statusReply);
    if (body?.password === "already-initialized")
      return new Response("already initialized", { status: 409 });
    return new Response(null, { status: 204 });
  },
});
afterAll(() => server.stop(true));
const api = createApi(
  new ApiClient(`http://127.0.0.1:${server.port}/scoped/api/v1/admin`),
);
test("首先查询初始化状态，使用部署前缀且不携带失效旧token", async () => {
  expect(await api.setupStatus()).toEqual({
    needs_setup: true,
    username: "superadmin",
  });
  expect(observed).toEqual({
    method: "GET",
    path: "/scoped/api/v1/admin/auth/setup",
    authorization: null,
    body: null,
  });
});
test("初始化真实POST只发送选择的密码并解析204", async () => {
  const body = { password: "test-new-value" };
  expect(await api.initialize(body)).toBeUndefined();
  expect(observed).toEqual({
    method: "POST",
    path: "/scoped/api/v1/admin/auth/setup",
    authorization: null,
    body,
  });
});
test("并发已初始化409使用初始化错误文案，不允许覆盖", async () => {
  await expect(
    api.initialize({ password: "already-initialized" }),
  ).rejects.toThrow("管理员已初始化");
});

test("初始化状态缺失或格式异常时不假定管理员已经存在", async () => {
  try {
    for (const value of [
      {},
      { needs_setup: "false", username: "superadmin" },
      { needs_setup: false, username: "other" },
    ]) {
      statusReply = value;
      await expect(api.setupStatus()).rejects.toThrow("初始化状态");
    }
  } finally {
    statusReply = { needs_setup: true, username: "superadmin" };
  }
});
