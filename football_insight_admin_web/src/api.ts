import { ApiClient, ApiError } from "./apiClient";
import { productPath } from "./features/products/productsModel";
export interface Admin {
  id: string;
  username: string;
  display_name: string;
  role: string;
}
export interface LoginResult {
  access_token: string;
  expires_at: string;
  admin: Admin;
}
export interface CustomerAccount {
  id: string;
  account_identifier: string;
  display_name: string;
  avatar_url: string | null;
  has_wechat_binding: boolean;
  status: string;
  invite_code: string | null;
  invited_by: {
    id: string;
    display_name: string;
    account_identifier: string;
    referral_invite_code: string;
  } | null;
  membership_tier: string;
  membership_expires_at: string | null;
  created_at: string;
  updated_at: string;
}
export interface CustomerAccountPage {
  items: CustomerAccount[];
  total: number;
  page: number;
  page_size: number;
}
export interface Product {
  product_key: string;
  membership_tier: string;
  name: string;
  product_id: string;
  price_cents: number;
  membership_duration_months: number;
  enabled: boolean;
  remark: string;
  allowed_user_ids: string[];
  created_at: string;
  updated_at: string;
}
export type ProductPayload = Pick<
  Product,
  | "membership_tier"
  | "name"
  | "product_id"
  | "price_cents"
  | "membership_duration_months"
  | "enabled"
  | "remark"
  | "allowed_user_ids"
>;
export interface ProductPage {
  items: Product[];
  total: number;
  page: number;
  page_size: number;
}
export interface ProductFilters {
  query: string;
  membership_tier: string;
  enabled: string;
  page: number;
  page_size: number;
}
export interface AdminSetupStatus {
  needs_setup: boolean;
  username: "superadmin";
}
export interface CredentialsPayload {
  username: string;
  current_password: string;
  new_password?: string;
}
export function createApi(client: ApiClient) {
  return {
    setupStatus: async (signal?: AbortSignal): Promise<AdminSetupStatus> => {
      const status = await client.request<AdminSetupStatus>(
        "/auth/setup",
        null,
        { signal },
      );
      if (
        !status ||
        typeof status.needs_setup !== "boolean" ||
        status.username !== "superadmin"
      )
        throw new ApiError(200, "管理员初始化状态返回异常，请重新检查");
      return status;
    },
    initialize: (payload: { password: string }, signal?: AbortSignal) =>
      client.request<void>("/auth/setup", null, {
        method: "POST",
        body: JSON.stringify(payload),
        signal,
      }),
    login: (username: string, password: string, signal?: AbortSignal) =>
      client.request<LoginResult>("/auth/login", null, {
        method: "POST",
        body: JSON.stringify({ username, password }),
        signal,
      }),
    me: (token: string, signal?: AbortSignal) =>
      client.request<Admin>("/auth/me", token, { signal }),
    logout: (token: string) =>
      client.request<void>("/auth/logout", token, { method: "POST" }),
    updateCredentials: (
      token: string,
      payload: CredentialsPayload,
      signal?: AbortSignal,
    ) =>
      client.request<void>("/auth/credentials", token, {
        method: "PUT",
        body: JSON.stringify(payload),
        signal,
      }),
    customerAccounts: (
      token: string,
      query: string,
      page = 1,
      signal?: AbortSignal,
    ) => {
      const params = new URLSearchParams({
        query,
        page: String(page),
        page_size: "20",
      });
      return client.request<CustomerAccountPage>(`/users?${params}`, token, {
        signal,
      });
    },
    customerAccount: (token: string, id: string, signal?: AbortSignal) =>
      client.request<CustomerAccount>(
        `/users/${encodeURIComponent(id)}`,
        token,
        { signal },
      ),
    products: (
      token: string,
      filters: ProductFilters,
      signal?: AbortSignal,
    ) => {
      const query = new URLSearchParams();
      Object.entries(filters).forEach(([k, v]) => {
        if (v !== "") query.set(k, String(v));
      });
      return client.request<ProductPage>(`/products?${query}`, token, {
        signal,
      });
    },
    product: (token: string, key: string, signal?: AbortSignal) =>
      client.request<Product>(productPath(key), token, { signal }),
    save: (
      token: string,
      payload: ProductPayload,
      oldKey?: string,
      signal?: AbortSignal,
    ) =>
      client.request<Product>(
        oldKey ? productPath(oldKey) : "/products",
        token,
        {
          method: oldKey ? "PUT" : "POST",
          body: JSON.stringify(payload),
          signal,
        },
      ),
    delete: (token: string, key: string, signal?: AbortSignal) =>
      client.request<void>(productPath(key), token, {
        method: "DELETE",
        signal,
      }),
  };
}
export type AdminApi = ReturnType<typeof createApi>;
