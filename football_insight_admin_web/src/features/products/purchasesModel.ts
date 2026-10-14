import type { ProductPurchase, ProductPurchaseSummary } from "../../api";
import { centsToAmount } from "./productsModel";
const beijingClock = new Intl.DateTimeFormat("zh-CN", {
  timeZone: "Asia/Shanghai",
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
  second: "2-digit",
  hourCycle: "h23",
});
export function safeAvatarUrl(value: string | null): string | null {
  if (!value) return null;
  const normalized = value.trim();
  const inlineImage =
    /^data:image\/(?:png|jpe?g|gif|webp|bmp);base64,([A-Za-z0-9+/]+={0,2})$/i.exec(
      normalized,
    );
  if (inlineImage && inlineImage[1].length % 4 === 0) return normalized;
  try {
    const url = new URL(normalized);
    return ["http:", "https:"].includes(url.protocol) &&
      !url.username &&
      !url.password
      ? url.toString()
      : null;
  } catch {
    return null;
  }
}
export function purchaseDate(value: string | null): string {
  if (value === null) return "未记录";
  const date = new Date(value);
  if (!value || Number.isNaN(date.getTime())) return "日期异常";
  const parts: Record<string, string> = {};
  for (const part of beijingClock.formatToParts(date))
    parts[part.type] = part.value;
  return `${parts.year}-${parts.month}-${parts.day} ${parts.hour}:${parts.minute}:${parts.second}`;
}
export function purchaseExpiry(
  purchase: Pick<ProductPurchase, "expires_at" | "expiry_source">,
): { text: string; hint: string | null } {
  if (purchase.expiry_source === "snapshot" && purchase.expires_at === null)
    return { text: "长期有效", hint: null };
  if (purchase.expiry_source === "unrecorded" || !purchase.expires_at)
    return { text: "未记录", hint: null };
  const text = purchaseDate(purchase.expires_at);
  return {
    text,
    hint:
      purchase.expiry_source === "legacy_year" &&
      text !== "未记录" &&
      text !== "日期异常"
        ? "历史计算"
        : null,
  };
}
export function purchaseAmount(cents: number): string {
  return `¥${centsToAmount(cents)}`;
}
export function purchaseSummaryValues(summary: ProductPurchaseSummary): {
  users: string;
  orders: string;
  refunded: string;
  amount: string;
} {
  return {
    users: String(summary.purchased_user_count),
    orders: String(summary.purchased_order_count),
    refunded: String(summary.refunded_order_count),
    amount: purchaseAmount(summary.total_amount_cents),
  };
}

export class PurchaseIdentityError extends Error {}
export function assertPurchaseIdentity(
  expected: string | undefined,
  actual: string | undefined,
): void {
  if (!expected)
    throw new PurchaseIdentityError("商品信息需要刷新，请关闭窗口后重新查询");
  if (!actual || actual !== expected)
    throw new PurchaseIdentityError("商品已被删除或重建，请关闭窗口后重新查询");
}
