import { AuthSession } from "../../authSession";
import { expect, test } from "bun:test";
import {
  credentialsPayload,
  newCredentialsForm,
  completeCredentialsChange,
} from "./credentialsModel";
const changed = () => ({
  ...newCredentialsForm("original.admin"),
  username: "new.admin",
  currentPassword: "test-current-value",
});
test("当前用户名回填，新建设置表单没有密码数据", () => {
  expect(newCredentialsForm("original.admin")).toEqual({
    username: "original.admin",
    currentPassword: "",
    newPassword: "",
    confirmPassword: "",
  });
});
test("用户名规范化且只传当前管理员修改字段", () => {
  expect(
    credentialsPayload(
      { ...changed(), username: "  NEW.Admin  " },
      "original.admin",
    ),
  ).toEqual({ username: "new.admin", current_password: "test-current-value" });
});
test("用户名3..64 ASCII范围校验", () => {
  for (const username of ["abc", "a".repeat(64), "admin.name_2-3"])
    expect(
      credentialsPayload({ ...changed(), username }, "original.admin").username,
    ).toBe(username);
  for (const username of ["ab", "a".repeat(65), "含中文", "abc def", "abc@def"])
    expect(() =>
      credentialsPayload({ ...changed(), username }, "original.admin"),
    ).toThrow("账号");
});
test("当前密码必填，密码输入不修剪", () => {
  expect(() =>
    credentialsPayload({ ...changed(), currentPassword: "" }, "original.admin"),
  ).toThrow("当前密码");
  const form = {
    ...changed(),
    currentPassword: " current spaces ",
    newPassword: " password spaces ",
    confirmPassword: " password spaces ",
  };
  expect(credentialsPayload(form, "original.admin")).toEqual({
    username: "new.admin",
    current_password: " current spaces ",
    new_password: " password spaces ",
  });
});
test("新密码可空，填写时8..128字符且确认一致", () => {
  expect(
    credentialsPayload(changed(), "original.admin").new_password,
  ).toBeUndefined();
  for (const value of ["a".repeat(8), "a".repeat(128), "        "])
    expect(
      credentialsPayload(
        { ...changed(), newPassword: value, confirmPassword: value },
        "original.admin",
      ).new_password,
    ).toBe(value);
  for (const value of ["a".repeat(7), "a".repeat(129)])
    expect(() =>
      credentialsPayload(
        { ...changed(), newPassword: value, confirmPassword: value },
        "original.admin",
      ),
    ).toThrow("新密码");
  expect(() =>
    credentialsPayload(
      {
        ...changed(),
        newPassword: "test-new-value",
        confirmPassword: "test-other-value",
      },
      "original.admin",
    ),
  ).toThrow("确认密码");
});
test("没有实际修改时拒绝提交", () => {
  expect(() =>
    credentialsPayload(
      { ...changed(), username: " Original.Admin " },
      "original.admin",
    ),
  ).toThrow("修改");
  expect(() =>
    credentialsPayload(
      {
        ...changed(),
        username: "original.admin",
        newPassword: "test-current-value",
        confirmPassword: "test-current-value",
      },
      "original.admin",
    ),
  ).toThrow("修改");
  expect(
    credentialsPayload(
      {
        ...changed(),
        username: "original.admin",
        newPassword: "test-new-value",
        confirmPassword: "test-new-value",
      },
      "original.admin",
    ).new_password,
  ).toBe("test-new-value");
});

test("凭据成功后清除当前持久化session，只返回登录账号与成功提示", () => {
  const data = new Map<string, string>();
  const session = new AuthSession({
    getItem: (key) => data.get(key) ?? null,
    setItem: (key, value) => {
      data.set(key, value);
    },
    removeItem: (key) => {
      data.delete(key);
    },
  });
  session.setToken("current-session-token");
  expect(
    completeCredentialsChange(
      session,
      "current-session-token",
      "updated.admin",
    ),
  ).toEqual({
    username: "updated.admin",
    message: "管理员账号已更新，请使用新账号和密码登录",
  });
  expect(session.getToken()).toBeNull();
  expect([...data.entries()]).toEqual([]);
});
test("旧凭据响应不能清除新会话或切换新账号登录页", () => {
  const session = new AuthSession({
    getItem: () => null,
    setItem: () => {},
    removeItem: () => {},
  });
  session.setToken("new-session-token");
  expect(
    completeCredentialsChange(session, "old-session-token", "updated.admin"),
  ).toBeNull();
  expect(session.getToken()).toBe("new-session-token");
});
