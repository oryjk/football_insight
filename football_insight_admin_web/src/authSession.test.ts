import { expect, test } from "bun:test";
import { AuthSession, browserSessionStorage } from "./authSession";
function storage() {
  const data = new Map<string, string>();
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => {
      data.set(key, value);
    },
    removeItem: (key: string) => {
      data.delete(key);
    },
  };
}
test("旧会话的迟到 401 不能清除新登录", () => {
  const session = new AuthSession(storage());
  session.setToken("old");
  session.setToken("new");
  expect(session.invalidate("old")).toBe(false);
  expect(session.getToken()).toBe("new");
  expect(session.invalidate("new")).toBe(true);
  expect(session.getToken()).toBeNull();
});
test("登录存储按项目隔离，可恢复当前 token", () => {
  const store = storage();
  new AuthSession(store).setToken("saved");
  expect(new AuthSession(store).getToken()).toBe("saved");
});

test("浏览器拒绝读取 localStorage 时仍可建立内存会话", () => {
  const previous = Object.getOwnPropertyDescriptor(globalThis, "window");
  Object.defineProperty(globalThis, "window", {
    value: {
      get localStorage() {
        throw new Error("blocked");
      },
    },
    configurable: true,
  });
  try {
    const session = new AuthSession(browserSessionStorage());
    session.setToken("memory");
    expect(session.getToken()).toBe("memory");
  } finally {
    if (previous) Object.defineProperty(globalThis, "window", previous);
    else Reflect.deleteProperty(globalThis, "window");
  }
});
