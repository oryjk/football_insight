import { describe, expect, test } from "bun:test";
import { adminApiBaseForMount } from "./deploymentModel";

describe("同源部署路径", () => {
  test("根路径开发仍使用原 Admin API", () => {
    expect(adminApiBaseForMount("/")).toBe("/api/v1/admin");
  });
  test("子路径部署隔离足球 API，避开共享 Mesh 路由", () => {
    expect(adminApiBaseForMount("/football-admin/")).toBe("/football-admin/api/v1/admin");
    expect(adminApiBaseForMount("/football-admin")).toBe("/football-admin/api/v1/admin");
  });
});
