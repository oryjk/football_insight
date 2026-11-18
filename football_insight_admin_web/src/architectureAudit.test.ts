import { describe, expect, test } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const SRC = fileURLToPath(new URL("./", import.meta.url));
const TAILWIND_LAYOUT_UTILITY_RE =
  /(?:^|\s)(flex|inline-flex|grid|gap-\d+|p[xytblr]?-\d+|m[xytblr]?-\d+|w-full|h-full|items-center|justify-between)(?=\s|$)/;

function listFeatureTsxFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) return listFeatureTsxFiles(full);
    return entry.name.endsWith(".tsx") && !entry.name.endsWith(".test.tsx")
      ? [full]
      : [];
  });
}

describe("admin web architecture audit", () => {
  test("feature pages and shell use semantic class names instead of raw Tailwind layout utilities", () => {
    const files = listFeatureTsxFiles(SRC).filter(
      (path) => !relative(SRC, path).replaceAll("\\", "/").startsWith("components/ui/"),
    );
    const violations = files.flatMap((path) => {
      const name = relative(SRC, path).replaceAll("\\", "/");
      const raw = readFileSync(path, "utf8");
      return [...raw.matchAll(/className="([^"]+)"/g)]
        .map((m) => m[1] ?? "")
        .filter((cls) => TAILWIND_LAYOUT_UTILITY_RE.test(cls))
        .map((cls) => `${name}: className="${cls}"`);
    });
    expect(violations).toEqual([]);
  });
});
