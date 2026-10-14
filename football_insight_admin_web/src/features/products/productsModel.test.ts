import { describe, test, expect } from "bun:test";
import {
  amountToCents,
  centsToAmount,
  newProductForm,
  formToPayload,
  productPath,
  productToForm,
} from "./productsModel";
const uuid = "1f19bb22-53c7-4b8e-8969-e357f0a0c123";
const valid = () => ({
  ...newProductForm(),
  name: " 测试会员 ",
  product_id: " wx-id ",
  amount: "39.01",
});
describe("商品表单", () => {
  test("按字符串精确换分并往返显示", () => {
    for (const [input, cents] of [
      ["0.01", 1],
      ["0.29", 29],
      ["39.10", 3910],
      ["39", 3900],
      ["001.2", 120],
    ] as const) {
      expect(amountToCents(input)).toBe(cents);
      expect(amountToCents(centsToAmount(cents))).toBe(cents);
    }
  });
  test("拒绝零、负值、科学计数、三位小数及超范围金额", () => {
    for (const input of [
      "",
      "0",
      "-1",
      "1e3",
      "1.001",
      "abc",
      "90071992547409.92",
    ])
      expect(() => amountToCents(input)).toThrow();
  });
  test("新建默认停用、公开 V6，清理字段", () => {
    expect(newProductForm().enabled).toBe(false);
    expect(formToPayload(valid())).toEqual({
      membership_tier: "V6",
      name: "测试会员",
      product_id: "wx-id",
      price_cents: 3901,
      membership_duration_months: 12,
      enabled: false,
      remark: "",
      allowed_user_ids: [],
    });
  });
  test("V3 必须限购账号，错误不能变成公开商品", () => {
    expect(() =>
      formToPayload({
        ...valid(),
        membership_tier: "V3",
        purchaseScope: "specified",
      }),
    ).toThrow("V3");
    expect(() =>
      formToPayload({
        ...valid(),
        membership_tier: "V3",
        purchaseScope: "specified",
        allowedUsers: "bad-id",
      }),
    ).toThrow("UUID");
    expect(
      formToPayload({
        ...valid(),
        membership_tier: "V3",
        purchaseScope: "specified",
        allowedUsers: uuid + "\n" + uuid.toUpperCase(),
      }).allowed_user_ids,
    ).toEqual([uuid]);
  });
  test("拒绝空名称、空道具 ID 和不支持等级", () => {
    for (const patch of [
      { name: " " },
      { product_id: " " },
      { membership_tier: "V5" },
    ])
      expect(() => formToPayload({ ...valid(), ...patch })).toThrow();
  });
  test("业务键逐路径编码，不能作为路径或 query 注入", () => {
    expect(productPath("membership:V6@3900")).toBe(
      "/products/membership%3AV6%403900",
    );
    expect(productPath("a/b?c#d")).toBe("/products/a%2Fb%3Fc%23d");
  });
});

test("与后端 i32、长度、无空白道具 ID、非空 UUID 限制一致", () => {
  expect(amountToCents("21474836.47")).toBe(2147483647);
  expect(() => amountToCents("21474836.48")).toThrow();
  for (const patch of [
    { name: "字".repeat(201) },
    { product_id: "contains space" },
    { remark: "字".repeat(4001) },
    { allowedUsers: "00000000-0000-0000-0000-000000000000" },
    { allowedUsers: Array(101).fill(uuid).join("\n") },
  ])
    expect(() =>
      formToPayload({
        ...valid(),
        purchaseScope: "specified",
        allowedUsers: uuid,
        ...patch,
      }),
    ).toThrow();
});

test("会员有效期默认12月，整数范围1..1200，转换为数值payload", () => {
  expect(newProductForm().durationMonths).toBe("12");
  expect(formToPayload(valid()).membership_duration_months).toBe(12);
  for (const value of ["1", "1200", " 36 "])
    expect(
      formToPayload({ ...valid(), durationMonths: value })
        .membership_duration_months,
    ).toBe(Number(value));
  for (const value of ["", "0", "-1", "1.5", "1e2", "1201", "abc"])
    expect(() => formToPayload({ ...valid(), durationMonths: value })).toThrow(
      "会员有效期",
    );
});
test("编辑回填会员有效期，兼容旧记录默认12月", () => {
  const legacy = {
    catalog_product_id: "11111111-1111-4111-8111-111111111111",
    product_key: "membership:V6@3900",
    membership_tier: "V6",
    name: "会员",
    product_id: "id",
    price_cents: 3900,
    enabled: true,
    remark: "",
    allowed_user_ids: [],
    created_at: "",
    updated_at: "",
  };
  expect(productToForm(legacy).durationMonths).toBe("12");
  const current = { ...legacy, membership_duration_months: 36 };
  expect(formToPayload(productToForm(current)).membership_duration_months).toBe(
    36,
  );
});
