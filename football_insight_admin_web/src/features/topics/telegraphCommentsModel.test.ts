import { describe, expect, test } from "bun:test";
import {
  commentCreatedLabel,
  commentEntryLabel,
  commentStatusLabel,
} from "./telegraphCommentsModel";

describe("telegraphCommentsModel", () => {
  test("formats status labels and entry fallback title", () => {
    expect(commentStatusLabel("visible")).toBe("展示中");
    expect(commentStatusLabel("hidden")).toBe("已隐藏");
    expect(commentStatusLabel("deleted")).toBe("已删除");
    expect(commentEntryLabel(" 成都蓉城签下新援 ", "abcdef123456")).toBe(
      "成都蓉城签下新援",
    );
    expect(commentEntryLabel("  ", "abcdef123456")).toBe("条目 abcdef12");
  });

  test("formats Beijing minute timestamp", () => {
    expect(commentCreatedLabel("2026-11-17T06:30:00Z")).toBe(
      "2026-11-17 14:30",
    );
  });
});
