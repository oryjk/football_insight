import { expect, test } from "bun:test";
import {
  purchaseDate,
  purchaseExpiry,
  purchaseAmount,
  purchaseSummaryValues,
  safeAvatarUrl,
  assertPurchaseIdentity,
} from "./purchasesModel";
test("购买金额按实际整数分精确展示，退款仍显示原支付金额", () => {
  for (const [cents, label] of [
    [1, "¥0.01"],
    [29, "¥0.29"],
    [3910, "¥39.10"],
    [0, "¥0.00"],
  ] as const)
    expect(purchaseAmount(cents)).toBe(label);
});
test("购买时间按北京时间展示，不直接使用UTC或裸ISO串", () => {
  expect(purchaseDate("2026-10-14T03:04:05Z")).toBe("2026-10-14 11:04:05");
  expect(purchaseDate("2026-10-14T11:04:05+08:00")).toBe("2026-10-14 11:04:05");
  expect(purchaseDate(null)).toBe("未记录");
  expect(purchaseDate("invalid")).toBe("日期异常");
});
test("到期来源按真实快照、历史计算和未记录区分", () => {
  expect(
    purchaseExpiry({ expires_at: null, expiry_source: "snapshot" }),
  ).toEqual({ text: "长期有效", hint: null });
  expect(
    purchaseExpiry({
      expires_at: "2027-10-14T03:04:05Z",
      expiry_source: "snapshot",
    }),
  ).toEqual({ text: "2027-10-14 11:04:05", hint: null });
  expect(
    purchaseExpiry({
      expires_at: "2027-10-14T03:04:05Z",
      expiry_source: "legacy_year",
    }),
  ).toEqual({ text: "2027-10-14 11:04:05", hint: "历史计算" });
  expect(
    purchaseExpiry({ expires_at: null, expiry_source: "unrecorded" }),
  ).toEqual({ text: "未记录", hint: null });
});
test("汇总直接来自API，与当前订单分页无关，金额不是退款净额", () => {
  expect(
    purchaseSummaryValues({
      purchased_user_count: 7,
      purchased_order_count: 23,
      refunded_order_count: 5,
      total_amount_cents: 3901,
    }),
  ).toEqual({ users: "7", orders: "23", refunded: "5", amount: "¥39.01" });
});
test("真实头像仅允许绝对http/https，不接受可执行或带凭据URL", () => {
  expect(safeAvatarUrl("https://avatars.example.test/photo.png")).toBe(
    "https://avatars.example.test/photo.png",
  );
  expect(safeAvatarUrl("http://avatars.example.test/photo.png")).toBe(
    "http://avatars.example.test/photo.png",
  );
  for (const value of [
    null,
    "",
    "javascript:alert(1)",
    "data:image/svg+xml,test",
    "/relative/avatar.png",
    "ftp://example.test/a",
    "https://user:private@example.test/a",
  ])
    expect(safeAvatarUrl(value)).toBeNull();
});

test("同业务键删除重建后拒绝替换商品的购买汇总", () => {
  const original = {
    product_key: "membership:V6@3900",
    catalog_product_id: "11111111-1111-4111-8111-111111111111",
  };
  const replacement = {
    product_key: "membership:V6@3900",
    catalog_product_id: "22222222-2222-4222-8222-222222222222",
  };
  expect(() =>
    assertPurchaseIdentity(
      original.catalog_product_id,
      original.catalog_product_id,
    ),
  ).not.toThrow();
  expect(() =>
    assertPurchaseIdentity(
      original.catalog_product_id,
      replacement.catalog_product_id,
    ),
  ).toThrow("商品已被删除或重建");
  expect(() =>
    assertPurchaseIdentity(undefined, replacement.catalog_product_id),
  ).toThrow();
  expect(() =>
    assertPurchaseIdentity(original.catalog_product_id, undefined),
  ).toThrow();
});
