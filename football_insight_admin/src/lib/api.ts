export interface AdminUser {
  id: string;
  account_identifier: string;
  display_name: string;
  avatar_url: string | null;
  has_wechat_binding: boolean;
  status: "active" | "disabled";
  invite_code: string | null;
  invited_by: AdminInviter | null;
  membership_tier: string;
  membership_expires_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface AdminInviter {
  id: string;
  display_name: string;
  account_identifier: string;
  referral_invite_code: string;
}

export interface AdminUserListResponse {
  total: number;
  page: number;
  page_size: number;
  items: AdminUser[];
}

export interface CreateUserPayload {
  account_identifier: string;
  display_name: string;
  avatar_url?: string | null;
  password: string;
  membership_tier: string;
  membership_expires_at?: string | null;
}

export interface UpdateUserPayload {
  account_identifier?: string;
  display_name?: string;
  avatar_url?: string | null;
  membership_tier?: string;
  membership_expires_at?: string | null;
}

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL?.replace(/\/$/, "") ||
  "";

export class AdminApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
  ) {
    super(message);
  }
}

export async function listUsers(params: {
  token: string;
  displayName?: string;
  page: number;
  pageSize: number;
}): Promise<AdminUserListResponse> {
  const displayName = params.displayName?.trim() || "";
  const query = new URLSearchParams({
    page: String(params.page),
    page_size: String(params.pageSize),
  });
  if (displayName) {
    query.set("display_name", displayName);
  }

  return request<AdminUserListResponse>(
    `/api/v1/admin/users?${query.toString()}`,
    {
      token: params.token,
    },
  );
}

export async function createUser(token: string, payload: CreateUserPayload) {
  return request<AdminUser>("/api/v1/admin/users", {
    method: "POST",
    token,
    body: payload,
  });
}

export async function updateUser(
  token: string,
  userId: string,
  payload: UpdateUserPayload,
) {
  return request<AdminUser>(`/api/v1/admin/users/${userId}`, {
    method: "PATCH",
    token,
    body: payload,
  });
}

export async function deleteUser(token: string, userId: string) {
  await request<void>(`/api/v1/admin/users/${userId}`, {
    method: "DELETE",
    token,
  });
}

async function request<T>(
  path: string,
  options: {
    method?: string;
    token: string;
    body?: unknown;
  },
): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    method: options.method || "GET",
    headers: {
      "Content-Type": "application/json",
      "X-Admin-Token": options.token,
    },
    body: options.body === undefined ? undefined : JSON.stringify(options.body),
  });

  if (!response.ok) {
    const message = await response.text();
    throw new AdminApiError(message || "请求失败", response.status);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  const contentType = response.headers.get("content-type") || "";
  if (!contentType.includes("application/json")) {
    const text = await response.text();
    const preview = text.trim().slice(0, 80);
    throw new AdminApiError(
      preview.startsWith("<!doctype") || preview.startsWith("<html")
        ? "接口没有转发到后端：请重启前端 dev server，并确认 Rust 后端正在运行"
        : preview || "接口返回的不是 JSON",
      response.status,
    );
  }

  return response.json() as Promise<T>;
}
