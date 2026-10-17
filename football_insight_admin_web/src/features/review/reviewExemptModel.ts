import type { CustomerAccount, ReviewExemptUser } from "../../api";

type Named = { display_name: string | null; account_identifier: string };

export function exemptName(user: Named): string {
  return user.display_name?.trim() || "未设置昵称";
}

export function exemptIds(items: ReviewExemptUser[]): Set<string> {
  return new Set(items.map((item) => item.user_id.toLowerCase()));
}

export function isAlreadyExempt(
  account: Pick<CustomerAccount, "id">,
  items: ReviewExemptUser[],
): boolean {
  return exemptIds(items).has(account.id.toLowerCase());
}

// A repeated add keeps the existing row in place; a new one goes first.
export function withAdded(
  items: ReviewExemptUser[],
  added: ReviewExemptUser,
): ReviewExemptUser[] {
  const id = added.user_id.toLowerCase();
  return items.some((item) => item.user_id.toLowerCase() === id)
    ? items.map((item) => (item.user_id.toLowerCase() === id ? added : item))
    : [added, ...items];
}

export function withRemoved(
  items: ReviewExemptUser[],
  userId: string,
): ReviewExemptUser[] {
  const id = userId.toLowerCase();
  return items.filter((item) => item.user_id.toLowerCase() !== id);
}
