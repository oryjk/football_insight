import type { Product, ProductPayload } from "../../api";
export const membershipTiers = ["V3", "V6", "V7", "V8", "V9"] as const;
export interface ProductForm {
  membership_tier: string;
  name: string;
  product_id: string;
  amount: string;
  durationMonths: string;
  enabled: boolean;
  remark: string;
  allowedUsers: string;
  purchaseScope: "public" | "specified";
}
export class FormError extends Error {
  constructor(
    public field: keyof ProductForm,
    message: string,
  ) {
    super(message);
  }
}
export function amountToCents(value: string): number {
  const text = value.trim();
  if (!/^\d+(?:\.\d{1,2})?$/.test(text))
    throw new FormError("amount", "道具金额须为正数，最多两位小数");
  const [whole, fraction = ""] = text.split(".");
  const cents = BigInt(whole) * 100n + BigInt(fraction.padEnd(2, "0"));
  if (cents <= 0n || cents > 2147483647n)
    throw new FormError("amount", "道具金额超出有效范围");
  return Number(cents);
}
export function centsToAmount(value: number): string {
  if (!Number.isSafeInteger(value) || value < 0)
    throw new Error("无效的金额（分）");
  const cents = BigInt(value);
  return `${cents / 100n}.${String(cents % 100n).padStart(2, "0")}`;
}
export function newProductForm(): ProductForm {
  return {
    membership_tier: "V6",
    name: "",
    product_id: "",
    amount: "",
    durationMonths: "12",
    enabled: false,
    remark: "",
    allowedUsers: "",
    purchaseScope: "public",
  };
}
export function productToForm(
  p: Omit<Product, "membership_duration_months"> &
    Partial<Pick<Product, "membership_duration_months">>,
): ProductForm {
  return {
    membership_tier: p.membership_tier,
    name: p.name,
    product_id: p.product_id,
    amount: centsToAmount(p.price_cents),
    durationMonths: String(p.membership_duration_months ?? 12),
    enabled: p.enabled,
    remark: p.remark,
    allowedUsers: p.allowed_user_ids.join("\n"),
    purchaseScope:
      p.membership_tier === "V3" || p.allowed_user_ids.length > 0
        ? "specified"
        : "public",
  };
}
export function productPath(key: string): string {
  return `/products/${encodeURIComponent(key)}`;
}
export function formToPayload(form: ProductForm): ProductPayload {
  if (form.purchaseScope !== "public" && form.purchaseScope !== "specified")
    throw new FormError("purchaseScope", "请选择有效的购买范围");
  if (
    !membershipTiers.includes(
      form.membership_tier as (typeof membershipTiers)[number],
    )
  )
    throw new FormError("membership_tier", "请选择支持的会员等级");
  if ([...form.name.trim()].length > 200)
    throw new FormError("name", "商品名称最多 200 字");
  if (
    [...form.product_id.trim()].length > 200 ||
    /\s/.test(form.product_id.trim())
  )
    throw new FormError("product_id", "微信道具 ID 最多 200 字，不能含空白");
  if ([...form.remark.trim()].length > 4000)
    throw new FormError("remark", "备注最多 4000 字");
  if (!form.name.trim()) throw new FormError("name", "请输入商品名称");
  if (!form.product_id.trim())
    throw new FormError("product_id", "请输入微信道具 ID");
  if (form.membership_tier === "V3" && form.purchaseScope !== "specified")
    throw new FormError("purchaseScope", "V3 测试商品必须指定购买账号");
  const allowed =
    form.purchaseScope === "specified" && form.allowedUsers.trim()
      ? form.allowedUsers
          .trim()
          .split(/[\s,，;；]+/)
          .map((v) => v.toLowerCase())
      : [];
  if (allowed.length > 100)
    throw new FormError("allowedUsers", "允许购买账号最多 100 个");
  if (
    allowed.some(
      (v) =>
        v === "00000000-0000-0000-0000-000000000000" ||
        !/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(
          v,
        ),
    )
  )
    throw new FormError(
      "allowedUsers",
      "允许购买账号须填写有效 UUID，每行一个",
    );
  if (form.membership_tier === "V3" && !allowed.length)
    throw new FormError("allowedUsers", "V3 测试商品必须指定允许购买账号 UUID");
  if (form.purchaseScope === "specified" && !allowed.length)
    throw new FormError("allowedUsers", "指定账号商品必须至少选择一个购买账号");
  const durationText = form.durationMonths.trim();
  const duration = Number(durationText);
  if (
    !/^\d+$/.test(durationText) ||
    !Number.isSafeInteger(duration) ||
    duration < 1 ||
    duration > 1200
  )
    throw new FormError("durationMonths", "会员有效期须为 1–1200 月的整数");
  return {
    membership_tier: form.membership_tier,
    name: form.name.trim(),
    product_id: form.product_id.trim(),
    price_cents: amountToCents(form.amount),
    membership_duration_months: duration,
    enabled: form.enabled,
    remark: form.remark.trim(),
    allowed_user_ids: [...new Set(allowed)],
  };
}
export function formatTime(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : date.toLocaleString("zh-CN", { hour12: false });
}
