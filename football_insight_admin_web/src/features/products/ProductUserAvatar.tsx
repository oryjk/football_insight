import { UserRound } from "lucide-react";
import {
  Avatar,
  AvatarImage,
  AvatarFallback,
} from "../../components/ui/avatar";
import { safeAvatarUrl } from "./purchasesModel";
export function PurchaseAvatar({ url }: { url: string | null }) {
  const safeUrl = safeAvatarUrl(url);
  return (
    <Avatar className="purchase-avatar" aria-hidden="true">
      <AvatarImage
        src={safeUrl ?? undefined}
        alt=""
        loading="lazy"
        referrerPolicy="no-referrer"
      />
      <AvatarFallback>
        <UserRound />
      </AvatarFallback>
    </Avatar>
  );
}
