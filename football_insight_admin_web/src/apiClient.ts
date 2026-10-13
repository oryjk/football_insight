export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}
const messages: Record<number, string> = {
  400: "请求内容无效，请检查输入",
  401: "登录已过期，请重新登录",
  403: "当前管理员没有商品管理权限",
  404: "商品不存在，可能已被其他管理员删除，请刷新列表",
  409: "该等级与金额的商品已存在，请修改后重试",
};
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "请求失败，请重试";
}
export function isAborted(error: unknown): boolean {
  return error instanceof Error && error.name === "AbortError";
}
export class ApiClient {
  constructor(
    private base: string,
    private onUnauthorized?: (token: string) => void,
  ) {}
  async request<T>(
    path: string,
    token: string | null,
    init: RequestInit = {},
  ): Promise<T> {
    const headers = new Headers(init.headers);
    headers.set("Accept", "application/json");
    if (init.body) headers.set("Content-Type", "application/json");
    if (token) headers.set("Authorization", `Bearer ${token}`);
    let response: Response;
    try {
      response = await fetch(this.base + path, {
        ...init,
        headers,
        cache: "no-store",
      });
    } catch (error) {
      if (isAborted(error)) throw error;
      throw new ApiError(0, "网络连接失败，请检查连接后重试");
    }
    if (!response.ok) {
      if (response.status === 401 && token) this.onUnauthorized?.(token);
      const credentialsMessages: Record<number, string> = {
        400: "账号或密码格式无效，请检查输入",
        403: "当前密码不正确",
        409: "管理员账号已存在",
      };
      const setupMessages: Record<number, string> = {
        400: "初始化密码格式无效，请检查输入",
        409: "管理员已初始化，请登录",
      };
      const accountMessages: Record<number, string> = {
        403: "当前管理员没有账号查询权限",
        404: "账号不存在，已有 UUID 已保留",
      };
      const message =
        (path.startsWith("/users")
          ? accountMessages[response.status]
          : undefined) ||
        (path === "/auth/setup" ? setupMessages[response.status] : undefined) ||
        (path === "/auth/credentials"
          ? credentialsMessages[response.status]
          : undefined) ||
        messages[response.status] ||
        "服务暂不可用，请稍后重试";
      // Status-specific Chinese recovery messages avoid proxy HTML and backend internals.
      throw new ApiError(response.status, message);
    }
    if (response.status === 204) return undefined as T;
    try {
      return (await response.json()) as T;
    } catch {
      throw new ApiError(response.status, "服务返回格式异常，请刷新后重试");
    }
  }
}
