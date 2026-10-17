import { describe, expect, test } from "bun:test";
import type { ReviewExemptUser } from "../../api";
import {
  exemptName,
  isAlreadyExempt,
  withAdded,
  withRemoved,
} from "./reviewExemptModel";

const user = (id: string, name: string | null = null): ReviewExemptUser => ({
  user_id: id,
  display_name: name,
  account_identifier: `acc-${id}`,
  avatar_url: null,
  created_at: "2026-10-17T00:00:00Z",
});

describe("review exempt model", () => {
  test("falls back when the nickname is blank", () => {
    expect(exemptName({ display_name: "  ", account_identifier: "a" })).toBe(
      "未设置昵称",
    );
    expect(exemptName({ display_name: "测试号", account_identifier: "a" })).toBe(
      "测试号",
    );
  });
  test("matches existing members case-insensitively", () => {
    expect(isAlreadyExempt({ id: "ABC" }, [user("abc")])).toBe(true);
    expect(isAlreadyExempt({ id: "x" }, [user("abc")])).toBe(false);
  });
  test("adds new members first and keeps repeats in place", () => {
    const list = [user("a"), user("b")];
    expect(withAdded(list, user("c")).map((item) => item.user_id)).toEqual([
      "c",
      "a",
      "b",
    ]);
    expect(withAdded(list, user("b", "新")).map((item) => item.user_id)).toEqual(
      ["a", "b"],
    );
  });
  test("removes by id", () => {
    expect(withRemoved([user("a"), user("b")], "A").map((i) => i.user_id)).toEqual(
      ["b"],
    );
  });
});
