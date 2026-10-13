import { expect, test } from "bun:test";
import { newSetupForm, setupPayload } from "./setupModel";
test("初始管理员由本人设置密码，不提供默认密码", () =>
  expect(newSetupForm()).toEqual({ password: "", confirmPassword: "" }));
test("初始密码按Unicode字符8..128校验，原样保留空格", () => {
  for (const password of [
    "a".repeat(8),
    "a".repeat(128),
    "😀".repeat(8),
    "        ",
  ])
    expect(setupPayload({ password, confirmPassword: password })).toEqual({
      password,
    });
  for (const password of ["", "a".repeat(7), "a".repeat(129), "😀".repeat(7)])
    expect(() => setupPayload({ password, confirmPassword: password })).toThrow(
      "密码",
    );
});
test("初始化确认必须一致，提交不含账号、确认密码或默认凭据", () => {
  expect(() =>
    setupPayload({
      password: "test-new-value",
      confirmPassword: "test-other-value",
    }),
  ).toThrow("确认密码");
  expect(
    setupPayload({
      password: "test-new-value",
      confirmPassword: "test-new-value",
    }),
  ).toEqual({ password: "test-new-value" });
});
