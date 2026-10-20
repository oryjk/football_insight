import { expect, test } from "bun:test";
import { parseStudentConfig, studentConfigPayload } from "./studentConfigModel";

function draft() {
  return {
    contact_qr_url: "",
    services: [
      { code: "certificate_renewal" as const, title: "证书续期", description: "", price_cents: 9900, unit: "年", enabled: false, product_id: "" },
      { code: "tool_upgrade" as const, title: "工具升级", description: "", price_cents: 69900, unit: "套", enabled: false, product_id: "" },
    ],
  };
}

test("允许停用服务保存空草稿，完整保留两项固定配置", () => {
  expect(studentConfigPayload(parseStudentConfig(draft()))).toEqual(draft());
});
test("提交时修剪外围空白并保留说明内的换行", () => {
  const config = draft();
  config.contact_qr_url = " https://cdn.example.test/qr.png ";
  config.services[0] = { ...config.services[0], description: "  续期说明\n第二行  ", product_id: " published-99 ", enabled: true };
  const result = studentConfigPayload(parseStudentConfig(config));
  expect(result.contact_qr_url).toBe("https://cdn.example.test/qr.png");
  expect(result.services[0].description).toBe("续期说明\n第二行");
  expect(result.services[0].product_id).toBe("published-99");
});
test("开放购买必须有说明、商品ID与二维码", () => {
  const config = draft();
  config.services[0].enabled = true;
  expect(() => studentConfigPayload(parseStudentConfig(config))).toThrow("说明");
  config.services[0].description = "续期";
  expect(() => studentConfigPayload(parseStudentConfig(config))).toThrow("商品 ID");
  config.services[0].product_id = "published-99";
  expect(() => studentConfigPayload(parseStudentConfig(config))).toThrow("二维码");
});
test("说明和商品ID按Unicode字符计数，拒绝内部空白和越界", () => {
  const config = draft();
  config.services[0].description = "⚽".repeat(4000);
  config.services[0].product_id = "球".repeat(200);
  expect(() => studentConfigPayload(parseStudentConfig(config))).not.toThrow();
  config.services[0].description += "⚽";
  expect(() => studentConfigPayload(parseStudentConfig(config))).toThrow("4000");
  config.services[0].description = "";
  config.services[0].product_id += "球";
  expect(() => studentConfigPayload(parseStudentConfig(config))).toThrow("200");
  config.services[0].product_id = "published\t99";
  expect(() => studentConfigPayload(parseStudentConfig(config))).toThrow("空白");
});
test("二维码只允许无账号密码的HTTPS链接，最多2048字符", () => {
  for (const url of ["http://example.test/q.png", "javascript:alert(1)", "https://name:secret@example.test/q.png", "https://example.test/" + "x".repeat(2048)]) {
    const config = draft();
    config.contact_qr_url = url;
    expect(() => studentConfigPayload(parseStudentConfig(config))).toThrow("二维码");
  }
});
test("响应拒绝缺失、重复服务、类型错误及变化的固定名称金额单位", () => {
  const config = draft();
  for (const invalid of [null, {}, { ...config, services: [config.services[0]] }, { ...config, services: [config.services[0], config.services[0]] }]) {
    expect(() => parseStudentConfig(invalid)).toThrow("配置返回异常");
  }
  for (const patch of [{ title: "另一个服务" }, { price_cents: 1 }, { unit: "月" }, { enabled: "false" }, { description: null }, { product_id: null }]) {
    expect(() => parseStudentConfig({ ...config, services: [{ ...config.services[0], ...patch }, config.services[1]] })).toThrow("配置返回异常");
  }
});
test("响应顺序变化仍按固定代码展示，不丢失商品映射", () => {
  const config = draft();
  config.services[0].product_id = "renewal-99";
  config.services.reverse();
  expect(parseStudentConfig(config).services[0].product_id).toBe("renewal-99");
});
