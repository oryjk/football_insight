import { afterAll, expect, test } from "bun:test";
import { createApi } from "../../api";
import { ApiClient } from "../../apiClient";
let observed: {
  path: string;
  query: Record<string, string>;
  authorization: string | null;
  method: string;
} | null = null;
const reply = {
  catalog_product_id: "11111111-1111-4111-8111-111111111111",
  summary: {
    purchased_user_count: 7,
    purchased_order_count: 23,
    refunded_order_count: 5,
    total_amount_cents: 3901,
  },
  items: [
    {
      order_no: "order-example",
      user_id: "11111111-1111-4111-8111-111111111111",
      user_nickname: "示例购买者",
      user_avatar_url: null,
      paid_at: "2026-10-14T03:04:05Z",
      expires_at: null,
      expiry_source: "unrecorded",
      amount_cents: 29,
      status: "refunded",
    },
  ],
  total: 23,
  page: 2,
  page_size: 20,
};
const server = Bun.serve({
  port: 0,
  fetch(req) {
    const url = new URL(req.url);
    observed = {
      path: url.pathname,
      query: Object.fromEntries(url.searchParams),
      authorization: req.headers.get("Authorization"),
      method: req.method,
    };
    return Response.json(reply);
  },
});
afterAll(() => server.stop(true));
const api = createApi(
  new ApiClient(`http://127.0.0.1:${server.port}/scoped/api/v1/admin`),
);
test("商品购买记录编码业务键并携带Bearer与分页/部署前缀", async () => {
  const result = await api.productPurchases(
    "session-token",
    "membership:V6@3900",
    2,
    20,
  );
  expect(observed).toEqual({
    path: "/scoped/api/v1/admin/products/membership%3AV6%403900/purchases",
    query: { page: "2", page_size: "20" },
    authorization: "Bearer session-token",
    method: "GET",
  });
  expect(result.summary).toEqual(reply.summary);
  expect(result.catalog_product_id).toBe(reply.catalog_product_id);
  expect(result.items[0].amount_cents).toBe(29);
  expect(result.total).toBe(23);
});
test("购买记录路径不能由键注入query或其他路径", async () => {
  await api.productPurchases("session-token", "a/b?c#d");
  expect(observed?.path).toBe(
    "/scoped/api/v1/admin/products/a%2Fb%3Fc%23d/purchases",
  );
});
test("关闭或切换购买分页可通过AbortSignal取消", async () => {
  const controller = new AbortController();
  controller.abort();
  await expect(
    api.productPurchases(
      "session-token",
      "membership:V6@3900",
      1,
      20,
      controller.signal,
    ),
  ).rejects.toMatchObject({ name: "AbortError" });
});
