import { afterAll, expect, test } from "bun:test";
import { createApi } from "../../api";
import { ApiClient } from "../../apiClient";

const config = {
  contact_qr_url: "https://cdn.example.test/qr.png",
  services: [
    { code: "certificate_renewal" as const, title: "证书续期", description: "续期说明", price_cents: 9900, unit: "年", enabled: true, product_id: "published-99" },
    { code: "tool_upgrade" as const, title: "工具升级", description: "", price_cents: 69900, unit: "套", enabled: false, product_id: "" },
  ],
};
let requestData: unknown;
const invalidated: string[] = [];
const server = Bun.serve({ port: 0, async fetch(req) {
  const token = req.headers.get("Authorization");
  if (token === "Bearer expired") return new Response(null, { status: 401 });
  if (token === "Bearer denied") return new Response(null, { status: 403 });
  if (token === "Bearer malformed") return Response.json({});
  if (token === "Bearer conflict") return new Response(null, { status: 409 });
  if (token === "Bearer invalid") return new Response(null, { status: 400 });
  requestData = { path: new URL(req.url).pathname, method: req.method, authorization: token, body: req.method === "PUT" ? await req.json() : null };
  return Response.json(config);
} });
afterAll(() => server.stop(true));
const api = createApi(new ApiClient(`http://127.0.0.1:${server.port}/scoped/api/v1/admin`, (token) => invalidated.push(token)));

test("学员配置读写使用共享部署前缀、JWT和完整PUT契约", async () => {
  const result = await api.studentConfig("current");
  expect(result).toEqual(config);
  expect(requestData).toEqual({ path: "/scoped/api/v1/admin/student/config", method: "GET", authorization: "Bearer current", body: null });
  expect(await api.setStudentConfig("current", result)).toEqual(config);
  expect(requestData).toEqual({ path: "/scoped/api/v1/admin/student/config", method: "PUT", authorization: "Bearer current", body: config });
});
test("配置响应异常不能变成可写默认值，权限错误交由共享会话处理", async () => {
  await expect(api.studentConfig("malformed")).rejects.toThrow("配置返回异常");
  await expect(api.studentConfig("expired")).rejects.toThrow("登录已过期");
  expect(invalidated).toEqual(["expired"]);
  await expect(api.studentConfig("denied")).rejects.toThrow("商品管理权限");
});
test("已取消的配置读取和写入不返回成功", async () => {
  const controller = new AbortController();
  controller.abort();
  await expect(api.studentConfig("current", controller.signal)).rejects.toHaveProperty("name", "AbortError");
  await expect(api.setStudentConfig("current", await api.studentConfig("current"), controller.signal)).rejects.toHaveProperty("name", "AbortError");
});
test("配置写入失败显示学员服务语义，不展示会员等级冲突提示", async () => {
  const payload = await api.studentConfig("current");
  await expect(api.setStudentConfig("conflict", payload)).rejects.toThrow("学员功能配置冲突");
  await expect(api.setStudentConfig("invalid", payload)).rejects.toThrow("学员功能配置无效");
});
