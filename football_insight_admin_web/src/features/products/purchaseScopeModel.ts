import { FormError, type ProductForm } from "./productsModel";
export function setPurchaseScope(
  form: ProductForm,
  scope: "public" | "specified",
): ProductForm {
  if (form.membership_tier === "V3" && scope === "public")
    throw new FormError("purchaseScope", "V3 测试商品必须指定购买账号");
  return { ...form, purchaseScope: scope };
}
export function setProductTier(form: ProductForm, tier: string): ProductForm {
  return {
    ...form,
    membership_tier: tier,
    ...(tier === "V3" ? { purchaseScope: "specified" as const } : {}),
  };
}
function entries(raw: string): string[] {
  return raw.trim()
    ? raw
        .trim()
        .split(/[\s,，;；]+/)
        .map((value) => value.toLowerCase())
    : [];
}
export function selectedAccountIds(raw: string): string[] {
  return [
    ...new Set(
      entries(raw).filter(
        (value) =>
          /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(
            value,
          ) && value !== "00000000-0000-0000-0000-000000000000",
      ),
    ),
  ];
}
export function addAccountSelection(
  form: ProductForm,
  id: string,
): ProductForm {
  const current = entries(form.allowedUsers);
  const next = id.toLowerCase();
  if (!selectedAccountIds(next).length)
    throw new FormError("allowedUsers", "账号 UUID 无效");
  if (current.includes(next)) return form;
  if (current.length >= 100)
    throw new FormError("allowedUsers", "允许购买账号最多 100 个");
  return {
    ...form,
    purchaseScope: "specified",
    allowedUsers: [...current, next].join("\n"),
  };
}
export function removeAccountSelection(
  form: ProductForm,
  id: string,
): ProductForm {
  return {
    ...form,
    allowedUsers: entries(form.allowedUsers)
      .filter((value) => value !== id.toLowerCase())
      .join("\n"),
  };
}
