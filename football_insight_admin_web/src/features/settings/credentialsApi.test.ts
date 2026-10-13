import { afterAll, expect, test } from "bun:test";
import { createApi } from "../../api";
import { ApiClient } from "../../apiClient";
let requestData: {
  path: string;
  method: string;
  authorization: string | null;
  body: unknown;
} | null = null;
const server = Bun.serve({
  port: 0,
  async fetch(req) {
    const body = (await req.json()) as { username: string };
    requestData = {
      path: new URL(req.url).pathname,
      method: req.method,
      authorization: req.headers.get("Authorization"),
      body,
    };
    if (body.username === "wrong-password")
      return new Response("wrong current password", { status: 403 });
    if (body.username === "already-taken")
      return new Response("username already exists", { status: 409 });
    return new Response(null, { status: 204 });
  },
});
afterAll(() => server.stop(true));
const api = createApi(
  new ApiClient(`http://127.0.0.1:${server.port}/scoped/api/v1/admin`),
);
test("凭据修改走部署前缀下真实PUT与204，不发送确认密码或管理员ID", async () => {
  const body = {
    username: "changed.admin",
    current_password: "test-current-value",
    new_password: "test-new-value",
  };
  expect(await api.updateCredentials("session-token", body)).toBeUndefined();
  expect(requestData).toEqual({
    path: "/scoped/api/v1/admin/auth/credentials",
    method: "PUT",
    authorization: "Bearer session-token",
    body,
  });
});
test("凭据错误使用账号设置语义，与商品错误区别", async () => {
  await expect(
    api.updateCredentials("session-token", {
      username: "wrong-password",
      current_password: "test-current-value",
    }),
  ).rejects.toThrow("当前密码不正确");
  await expect(
    api.updateCredentials("session-token", {
      username: "already-taken",
      current_password: "test-current-value",
    }),
  ).rejects.toThrow("管理员账号已存在");
});
