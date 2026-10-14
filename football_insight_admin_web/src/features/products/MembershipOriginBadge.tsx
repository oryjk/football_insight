import { Badge } from "../../components/ui/badge";
import { CreditCard, Users, CircleHelp } from "lucide-react";
export function MembershipOriginBadge({ origin }: { origin?: string }) {
  const value =
    origin === "purchase" || origin === "referral" ? origin : "unknown";
  const Icon =
    value === "purchase"
      ? CreditCard
      : value === "referral"
        ? Users
        : CircleHelp;
  const label =
    value === "purchase"
      ? "购买获得"
      : value === "referral"
        ? "推荐升级"
        : "未知来源";
  return (
    <Badge className="membership-origin-badge" data-origin={value}>
      <Icon aria-hidden="true" />
      {label}
    </Badge>
  );
}
