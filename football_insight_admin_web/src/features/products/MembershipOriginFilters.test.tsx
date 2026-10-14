import { expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";
import { MembershipOriginFilters } from "./MembershipOriginFilters";

test("membership origin filters provide three independently checked shadcn checkboxes", () => {
  const html = renderToStaticMarkup(
    <MembershipOriginFilters
      origins={["purchase", "unknown"]}
      onChange={() => {}}
    />,
  );
  expect(html.match(/role="checkbox"/g)?.length).toBe(3);
  expect(html.match(/aria-checked="true"/g)?.length).toBe(2);
  expect(html).toContain("支付购买");
  expect(html).toContain("推荐升级");
  expect(html).toContain("未知来源");
});
