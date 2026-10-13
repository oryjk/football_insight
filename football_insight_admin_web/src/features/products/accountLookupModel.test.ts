import { expect, test } from "bun:test";
import { lookupAccountMetadata } from "./accountLookupModel";
test("已有购买账号详情加载最多四个并发且复用重复UUID", async () => {
  const ids = Array.from({ length: 20 }, (_, index) => String(index));
  let active = 0,
    peak = 0,
    calls = 0;
  const result = await lookupAccountMetadata(
    [...ids, ids[0]],
    async (id) => {
      calls++;
      active++;
      peak = Math.max(peak, active);
      await new Promise((resolve) => setTimeout(resolve, 2));
      active--;
      return { id };
    },
    new AbortController().signal,
  );
  expect(peak).toBeLessThanOrEqual(4);
  expect(peak).toBeGreaterThan(0);
  expect(calls).toBe(20);
  expect(result.map((item) => item.value?.id)).toEqual(ids);
});
test("详情失败仍保留对应UUID结果，不拖掉其他已选账号", async () => {
  const result = await lookupAccountMetadata(
    ["kept", "missing"],
    async (id) => {
      if (id === "missing") throw new Error("missing");
      return id;
    },
    new AbortController().signal,
  );
  expect(result.map((item) => item.id)).toEqual(["kept", "missing"]);
  expect(result[1].error).toBeInstanceOf(Error);
});
test("取消后不派发后续详情或提交迟到结果", async () => {
  const controller = new AbortController();
  let calls = 0;
  const result = await lookupAccountMetadata(
    Array.from({ length: 20 }, (_, index) => String(index)),
    async () => {
      calls++;
      controller.abort();
      return "late";
    },
    controller.signal,
  );
  expect(calls).toBeLessThanOrEqual(4);
  expect(result).toEqual([]);
});
