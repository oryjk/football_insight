import { afterAll, expect, test } from "bun:test";
import { createApi } from "../../api";
import { ApiClient } from "../../apiClient";
let observed: {
  path: string;
  query: Record<string, string>;
  authorization: string | null;
} | null = null;
const account = {
  id: "11111111-1111-4111-8111-111111111111",
  account_identifier: "customer.example",
  display_name: "测试账号",
  membership_tier: "V6",
  status: "active",
};
const server = Bun.serve({
  port: 0,
  fetch(req) {
    const url = new URL(req.url);
    observed = {
      path: url.pathname,
      query: Object.fromEntries(url.searchParams),
      authorization: req.headers.get("Authorization"),
    };
    if (url.searchParams.get("query") === "denied")
      return new Response("forbidden", { status: 403 });
    if (url.pathname.endsWith("/users/missing"))
      return new Response("missing", { status: 404 });
    return Response.json(
      url.pathname.endsWith("/users")
        ? { items: [account], total: 1, page: 2, page_size: 20 }
        : account,
    );
  },
});
afterAll(() => server.stop(true));
const api = createApi(
  new ApiClient(`http://127.0.0.1:${server.port}/scoped/api/v1/admin`),
);
test("真实账号搜索使用query分页与部署前缀/Bearer", async () => {
  const result = await api.customerAccounts("session-token", "姓名 & 手机", 2);
  expect(result.items[0].account_identifier).toBe("customer.example");
  expect(observed).toEqual({
    path: "/scoped/api/v1/admin/users",
    query: { query: "姓名 & 手机", page: "2", page_size: "20" },
    authorization: "Bearer session-token",
  });
});
test("已有UUID详情解析使用编码路径，缺失保留账号而非商品语义", async () => {
  expect(
    (await api.customerAccount("session-token", account.id)).display_name,
  ).toBe("测试账号");
  await expect(api.customerAccount("session-token", "missing")).rejects.toThrow(
    "账号不存在",
  );
  await expect(api.customerAccounts("session-token", "denied")).rejects.toThrow(
    "账号查询权限",
  );
});
test("账号搜索遵循AbortSignal，迟到搜索可取消", async () => {
  const controller = new AbortController();
  controller.abort();
  await expect(
    api.customerAccounts("session-token", "", 1, controller.signal),
  ).rejects.toMatchObject({ name: "AbortError" });
});
