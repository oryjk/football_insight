import type { MembershipOrigin } from "../../api";
import { useId } from "react";
import { Checkbox } from "../../components/ui/checkbox";
import { Label } from "../../components/ui/label";
const options: [MembershipOrigin, string][] = [
  ["purchase", "支付购买"],
  ["referral", "推荐升级"],
  ["unknown", "未知来源"],
];
export function MembershipOriginFilters({
  origins,
  onChange,
}: {
  origins: MembershipOrigin[];
  onChange: (origins: MembershipOrigin[]) => void;
}) {
  const id = useId();
  return (
    <fieldset className="membership-origin-filters">
      <legend>会员来源（可多选）</legend>
      <div>
        {options.map(([origin, label]) => (
          <Label key={origin} htmlFor={`${id}-${origin}`}>
            <Checkbox
              id={`${id}-${origin}`}
              checked={origins.includes(origin)}
              onCheckedChange={(checked) =>
                onChange(
                  checked === true
                    ? [...origins.filter((value) => value !== origin), origin]
                    : origins.filter((value) => value !== origin),
                )
              }
            />
            {label}
          </Label>
        ))}
      </div>
    </fieldset>
  );
}
