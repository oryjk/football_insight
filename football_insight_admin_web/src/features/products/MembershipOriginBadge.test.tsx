import { expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { MembershipOriginBadge } from "./MembershipOriginBadge";

test("membership badges show explicit purchase, referral and unknown labels", () => {
  for (const [origin, label] of [
    ["purchase", "购买获得"],
    ["referral", "推荐升级"],
    ["unknown", "未知来源"],
    [undefined, "未知来源"],
    ["unrecognized", "未知来源"],
  ]) {
    const html = renderToStaticMarkup(
      <MembershipOriginBadge origin={origin} />,
    );
    expect(html).toContain(label!);
    expect(html).toContain('data-slot="badge"');
  }
});
