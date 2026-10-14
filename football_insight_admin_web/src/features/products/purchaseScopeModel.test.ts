import { expect, test } from "bun:test";
import { newProductForm, formToPayload, productToForm } from "./productsModel";
import {
  setPurchaseScope,
  setProductTier,
  selectedAccountIds,
  addAccountSelection,
  removeAccountSelection,
} from "./purchaseScopeModel";
const first = "11111111-1111-4111-8111-111111111111",
  second = "22222222-2222-4222-8222-222222222222";
const valid = () => ({
  ...newProductForm(),
  name: "商品",
  product_id: "property",
  amount: "12.34",
});
test("购买范围明确选择，所有支持等级指定账号不能为空", () => {
  expect(newProductForm().purchaseScope).toBe("public");
  for (const membership_tier of ["V3", "V6", "V7", "V8", "V9"]) {
    expect(() =>
      formToPayload({
        ...valid(),
        membership_tier,
        purchaseScope: "specified",
      }),
    ).toThrow();
    expect(
      formToPayload({
        ...valid(),
        membership_tier,
        purchaseScope: "specified",
        allowedUsers: first,
      }).allowed_user_ids,
    ).toEqual([first]);
  }
});
test("V3始终限购，切换等级保留已选账号", () => {
  const draft = { ...valid(), allowedUsers: first };
  expect(setProductTier(draft, "V3")).toMatchObject({
    membership_tier: "V3",
    purchaseScope: "specified",
    allowedUsers: first,
  });
  expect(() => setPurchaseScope(setProductTier(draft, "V3"), "public")).toThrow(
    "V3",
  );
  expect(() =>
    formToPayload({ ...draft, membership_tier: "V3", purchaseScope: "public" }),
  ).toThrow("V3");
});
test("显式公开提交空数组，暂存选择便于切回，保留已有数据", () => {
  const specified = {
    ...valid(),
    purchaseScope: "specified" as const,
    allowedUsers: first,
  };
  const open = setPurchaseScope(specified, "public");
  expect(open.allowedUsers).toBe(first);
  expect(formToPayload(open).allowed_user_ids).toEqual([]);
  expect(
    formToPayload(setPurchaseScope(open, "specified")).allowed_user_ids,
  ).toEqual([first]);
});
test("从服务端回填指定账号，不丢失未解析或已删除的UUID", () => {
  const product = {
    catalog_product_id: "11111111-1111-4111-8111-111111111111",
    ...formToPayload({
      ...valid(),
      purchaseScope: "specified",
      allowedUsers: first + "\n" + second,
    }),
    product_key: "membership:V6@1234",
    created_at: "",
    updated_at: "",
  };
  const form = productToForm(product);
  expect(form.purchaseScope).toBe("specified");
  expect(formToPayload(form).allowed_user_ids).toEqual([first, second]);
});
test("账号选择去重、移除指定项，手工无效输入不能悄悄消失", () => {
  let form: ReturnType<typeof newProductForm> = {
    ...valid(),
    purchaseScope: "specified",
    allowedUsers: first,
  };
  form = addAccountSelection(form, second);
  form = addAccountSelection(form, first.toUpperCase());
  expect(selectedAccountIds(form.allowedUsers)).toEqual([first, second]);
  expect(
    selectedAccountIds(removeAccountSelection(form, first).allowedUsers),
  ).toEqual([second]);
  const raw = addAccountSelection(
    { ...form, allowedUsers: "invalid\n" + first },
    second,
  );
  expect(raw.allowedUsers).toContain("invalid");
  expect(() => formToPayload(raw)).toThrow("UUID");
});

test("未知购买范围不能默认为公开", () => {
  expect(() =>
    formToPayload({ ...valid(), purchaseScope: "unknown" as "public" }),
  ).toThrow("购买范围");
});
