import * as React from "react";
import {
  BadgeCheck,
  Crown,
  Edit3,
  Gift,
  LogOut,
  Plus,
  RefreshCw,
  Search,
  Shield,
  Trash2,
  Users,
} from "lucide-react";

import {
  AdminApiError,
  AdminUser,
  CreateUserPayload,
  createUser,
  deleteUser,
  listUsers,
  updateUser,
} from "@/lib/api";
import { formatDateTime } from "@/lib/utils";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";

const TOKEN_STORAGE_KEY = "football-insight-admin-token";
const MEMBERSHIP_TIERS = Array.from({ length: 9 }, (_, index) => `V${index + 1}`);
const MEMBERSHIP_TIER_OPTIONS = MEMBERSHIP_TIERS.map((tier) => ({
  value: tier,
  label: tier,
}));

type UserFormState = {
  account_identifier: string;
  display_name: string;
  avatar_url: string;
  password: string;
  membership_tier: string;
};

const emptyForm: UserFormState = {
  account_identifier: "",
  display_name: "",
  avatar_url: "",
  password: "",
  membership_tier: "V1",
};

export default function App() {
  const [token, setToken] = React.useState(
    () => localStorage.getItem(TOKEN_STORAGE_KEY) || "",
  );
  const [tokenDraft, setTokenDraft] = React.useState(token);
  const [users, setUsers] = React.useState<AdminUser[]>([]);
  const [total, setTotal] = React.useState(0);
  const [page, setPage] = React.useState(1);
  const [pageSize] = React.useState(20);
  const [search, setSearch] = React.useState("");
  const [searchDraft, setSearchDraft] = React.useState("");
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [formOpen, setFormOpen] = React.useState(false);
  const [editingUser, setEditingUser] = React.useState<AdminUser | null>(null);
  const [form, setForm] = React.useState<UserFormState>(emptyForm);
  const [submitting, setSubmitting] = React.useState(false);

  const isAuthed = token.trim().length > 0;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));

  const loadUsers = React.useCallback(async () => {
    if (!token.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const result = await listUsers({
        token,
        displayName: search,
        page,
        pageSize,
      });
      setUsers(result.items);
      setTotal(result.total);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setLoading(false);
    }
  }, [page, pageSize, search, token]);

  React.useEffect(() => {
    void loadUsers();
  }, [loadUsers]);

  function saveToken(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const nextToken = tokenDraft.trim();
    localStorage.setItem(TOKEN_STORAGE_KEY, nextToken);
    setToken(nextToken);
    setPage(1);
  }

  function logout() {
    localStorage.removeItem(TOKEN_STORAGE_KEY);
    setToken("");
    setTokenDraft("");
    setUsers([]);
    setTotal(0);
  }

  function openCreateDialog() {
    setEditingUser(null);
    setForm(emptyForm);
    setFormOpen(true);
  }

  function openEditDialog(user: AdminUser) {
    setEditingUser(user);
    setForm({
      account_identifier: user.account_identifier,
      display_name: user.display_name,
      avatar_url: user.avatar_url || "",
      password: "",
      membership_tier: user.membership_tier,
    });
    setFormOpen(true);
  }

  async function submitForm(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setSubmitting(true);
    setError(null);

    try {
      if (editingUser) {
        await updateUser(token, editingUser.id, {
          account_identifier: form.account_identifier.trim(),
          display_name: form.display_name.trim(),
          avatar_url: form.avatar_url.trim() || null,
          membership_tier: form.membership_tier,
        });
      } else {
        const payload: CreateUserPayload = {
          account_identifier: form.account_identifier.trim(),
          display_name: form.display_name.trim(),
          avatar_url: form.avatar_url.trim() || null,
          password: form.password,
          membership_tier: form.membership_tier,
        };
        await createUser(token, payload);
      }
      setFormOpen(false);
      await loadUsers();
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setSubmitting(false);
    }
  }

  async function handleDelete(user: AdminUser) {
    const confirmed = window.confirm(`确定删除用户「${user.display_name}」吗？`);
    if (!confirmed) return;

    setError(null);
    try {
      await deleteUser(token, user.id);
      await loadUsers();
    } catch (err) {
      setError(errorMessage(err));
    }
  }

  return (
    <main className="min-h-screen bg-background text-foreground">
      {!isAuthed ? (
        <TokenLogin
          tokenDraft={tokenDraft}
          onTokenDraftChange={setTokenDraft}
          onSubmit={saveToken}
        />
      ) : (
        <div className="flex min-h-screen">
          <aside className="hidden w-64 border-r border-border bg-card px-4 py-5 lg:block">
            <div className="mb-8 flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-md bg-primary text-primary-foreground">
                <Shield className="h-5 w-5" />
              </div>
              <div>
                <div className="text-sm font-semibold">Football Insight</div>
                <div className="text-xs text-muted-foreground">Admin Console</div>
              </div>
            </div>
            <nav className="space-y-1">
              <button className="flex w-full items-center gap-2 rounded-md bg-secondary px-3 py-2 text-left text-sm font-medium">
                <Users className="h-4 w-4" />
                用户管理
              </button>
            </nav>
          </aside>

          <section className="flex min-w-0 flex-1 flex-col">
            <header className="border-b border-border bg-card/80 px-4 py-4 backdrop-blur md:px-7">
              <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <div>
                  <p className="text-xs font-medium text-muted-foreground">后台管理系统</p>
                  <h1 className="text-2xl font-semibold tracking-normal">用户列表</h1>
                </div>
                <div className="flex items-center gap-2">
                  <Button variant="outline" onClick={() => void loadUsers()}>
                    <RefreshCw className="mr-2 h-4 w-4" />
                    刷新
                  </Button>
                  <Button variant="outline" onClick={logout}>
                    <LogOut className="mr-2 h-4 w-4" />
                    退出
                  </Button>
                </div>
              </div>
            </header>

            <div className="flex-1 px-4 py-5 md:px-7">
              <div className="mb-4 flex flex-col gap-3 rounded-lg border border-border bg-card p-4 md:flex-row md:items-center md:justify-between">
                <form
                  className="flex w-full max-w-xl items-center gap-2"
                  onSubmit={(event) => {
                    event.preventDefault();
                    setPage(1);
                    setSearch(searchDraft);
                  }}
                >
                  <div className="relative flex-1">
                    <Search className="pointer-events-none absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                    <Input
                      className="pl-9"
                      placeholder="按昵称模糊搜索"
                      value={searchDraft}
                      onChange={(event) => setSearchDraft(event.target.value)}
                    />
                  </div>
                  <Button type="submit">搜索</Button>
                </form>
                <Button onClick={openCreateDialog}>
                  <Plus className="mr-2 h-4 w-4" />
                  新增用户
                </Button>
              </div>

              {error ? (
                <div className="mb-4 rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                  {error}
                </div>
              ) : null}

              <div className="overflow-hidden rounded-lg border border-border bg-card">
                <div className="overflow-x-auto">
                  <table className="w-full min-w-[1080px] border-collapse text-sm">
                    <thead className="bg-secondary text-xs uppercase tracking-wide text-muted-foreground">
                      <tr>
                        <th className="px-4 py-3 text-left">用户</th>
                        <th className="px-4 py-3 text-left">账号</th>
                        <th className="px-4 py-3 text-left">邀请码</th>
                        <th className="px-4 py-3 text-left">邀请来源</th>
                        <th className="px-4 py-3 text-left">会员等级</th>
                        <th className="px-4 py-3 text-left">微信绑定</th>
                        <th className="px-4 py-3 text-left">注册时间</th>
                        <th className="px-4 py-3 text-right">操作</th>
                      </tr>
                    </thead>
                    <tbody>
                      {users.map((user) => (
                        <tr key={user.id} className="border-t border-border">
                          <td className="px-4 py-3">
                            <div className="flex items-center gap-3">
                              <Avatar user={user} />
                              <div className="min-w-0">
                                <div className="truncate font-medium">{user.display_name}</div>
                                <div className="truncate text-xs text-muted-foreground">
                                  {user.id}
                                </div>
                              </div>
                            </div>
                          </td>
                          <td className="px-4 py-3">{user.account_identifier}</td>
                          <td className="px-4 py-3">
                            <InviteCode value={user.invite_code} />
                          </td>
                          <td className="px-4 py-3">
                            <InviterSummary user={user} />
                          </td>
                          <td className="px-4 py-3">
                            <Badge className="gap-1">
                              <Crown className="h-3 w-3" />
                              {user.membership_tier}
                            </Badge>
                            {user.status === "disabled" ? (
                              <Badge className="ml-2 border-destructive/30 bg-destructive/10 text-destructive">
                                不可用
                              </Badge>
                            ) : null}
                          </td>
                          <td className="px-4 py-3">
                            {user.has_wechat_binding ? (
                              <span className="inline-flex items-center gap-1 text-emerald-700">
                                <BadgeCheck className="h-4 w-4" />
                                已绑定
                              </span>
                            ) : (
                              <span className="text-muted-foreground">未绑定</span>
                            )}
                          </td>
                          <td className="px-4 py-3">{formatDateTime(user.created_at)}</td>
                          <td className="px-4 py-3 text-right">
                            <div className="flex justify-end gap-2">
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => openEditDialog(user)}
                              >
                                <Edit3 className="mr-1 h-3.5 w-3.5" />
                                编辑
                              </Button>
                              <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => void handleDelete(user)}
                              >
                                <Trash2 className="mr-1 h-3.5 w-3.5" />
                                删除
                              </Button>
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>

                {loading ? (
                  <div className="border-t border-border px-4 py-8 text-center text-sm text-muted-foreground">
                    正在加载用户数据...
                  </div>
                ) : users.length === 0 ? (
                  <div className="border-t border-border px-4 py-8 text-center text-sm text-muted-foreground">
                    暂无用户数据
                  </div>
                ) : null}

                <div className="flex items-center justify-between border-t border-border px-4 py-3 text-sm">
                  <span className="text-muted-foreground">共 {total} 位用户</span>
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      disabled={page <= 1}
                      onClick={() => setPage((value) => Math.max(1, value - 1))}
                    >
                      上一页
                    </Button>
                    <span className="min-w-16 text-center">
                      {page} / {totalPages}
                    </span>
                    <Button
                      variant="outline"
                      size="sm"
                      disabled={page >= totalPages}
                      onClick={() => setPage((value) => value + 1)}
                    >
                      下一页
                    </Button>
                  </div>
                </div>
              </div>
            </div>
          </section>
        </div>
      )}

      <UserFormDialog
        open={formOpen}
        editingUser={editingUser}
        form={form}
        submitting={submitting}
        onOpenChange={setFormOpen}
        onFormChange={setForm}
        onSubmit={submitForm}
      />
    </main>
  );
}

function TokenLogin({
  tokenDraft,
  onTokenDraftChange,
  onSubmit,
}: {
  tokenDraft: string;
  onTokenDraftChange: (value: string) => void;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
}) {
  return (
    <section className="flex min-h-screen items-center justify-center px-4">
      <form
        className="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-xl"
        onSubmit={onSubmit}
      >
        <div className="mb-6 flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-md bg-primary text-primary-foreground">
            <Shield className="h-5 w-5" />
          </div>
          <div>
            <h1 className="text-lg font-semibold">Football Insight Admin</h1>
            <p className="text-sm text-muted-foreground">输入管理密钥继续</p>
          </div>
        </div>
        <div className="space-y-2">
          <Label htmlFor="admin-token">Admin Token</Label>
          <Input
            id="admin-token"
            type="password"
            autoFocus
            value={tokenDraft}
            onChange={(event) => onTokenDraftChange(event.target.value)}
          />
        </div>
        <Button className="mt-5 w-full" type="submit">
          进入后台
        </Button>
      </form>
    </section>
  );
}

function UserFormDialog({
  open,
  editingUser,
  form,
  submitting,
  onOpenChange,
  onFormChange,
  onSubmit,
}: {
  open: boolean;
  editingUser: AdminUser | null;
  form: UserFormState;
  submitting: boolean;
  onOpenChange: (open: boolean) => void;
  onFormChange: React.Dispatch<React.SetStateAction<UserFormState>>;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
}) {
  return (
    <Dialog
      open={open}
      title={editingUser ? "编辑用户" : "新增用户"}
      onOpenChange={onOpenChange}
    >
      <form className="grid gap-4" onSubmit={onSubmit}>
        <Field label="账号" htmlFor="account_identifier">
          <Input
            id="account_identifier"
            value={form.account_identifier}
            onChange={(event) =>
              onFormChange((current) => ({
                ...current,
                account_identifier: event.target.value,
              }))
            }
            required
          />
        </Field>
        <Field label="昵称" htmlFor="display_name">
          <Input
            id="display_name"
            value={form.display_name}
            onChange={(event) =>
              onFormChange((current) => ({
                ...current,
                display_name: event.target.value,
              }))
            }
            required
          />
        </Field>
        <Field label="头像 URL" htmlFor="avatar_url">
          <Input
            id="avatar_url"
            value={form.avatar_url}
            onChange={(event) =>
              onFormChange((current) => ({
                ...current,
                avatar_url: event.target.value,
              }))
            }
          />
        </Field>
        {!editingUser ? (
          <Field label="初始密码" htmlFor="password">
            <Input
              id="password"
              type="password"
              value={form.password}
              minLength={6}
              onChange={(event) =>
                onFormChange((current) => ({
                  ...current,
                  password: event.target.value,
                }))
              }
              required
            />
          </Field>
        ) : null}
        <Field label="会员等级" htmlFor="membership_tier">
          <Select
            id="membership_tier"
            value={form.membership_tier}
            options={MEMBERSHIP_TIER_OPTIONS}
            onValueChange={(value) =>
              onFormChange((current) => ({
                ...current,
                membership_tier: value,
              }))
            }
          />
        </Field>
        {editingUser ? <InviteReadOnlyPanel user={editingUser} /> : null}
        <div className="mt-2 flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button type="submit" disabled={submitting}>
            {submitting ? "保存中..." : "保存"}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}

function InviteCode({ value }: { value: string | null }) {
  if (!value) {
    return <span className="text-muted-foreground">未生成</span>;
  }

  return (
    <span className="inline-flex max-w-32 items-center gap-1 rounded-md border border-border bg-background px-2 py-1 font-mono text-xs">
      <Gift className="h-3.5 w-3.5 text-muted-foreground" />
      <span className="truncate">{value}</span>
    </span>
  );
}

function InviterSummary({ user }: { user: AdminUser }) {
  if (!user.invited_by) {
    return <span className="text-muted-foreground">自然注册</span>;
  }

  return (
    <div className="max-w-48">
      <div className="truncate font-medium">{user.invited_by.display_name}</div>
      <div className="truncate text-xs text-muted-foreground">
        {user.invited_by.account_identifier}
      </div>
    </div>
  );
}

function InviteReadOnlyPanel({ user }: { user: AdminUser }) {
  return (
    <div className="grid gap-3 rounded-md border border-border bg-secondary/40 p-3">
      <div className="grid gap-1">
        <div className="text-xs font-medium text-muted-foreground">用户自己的邀请码</div>
        <div className="truncate font-mono text-sm">{user.invite_code || "未生成"}</div>
      </div>
      <div className="grid gap-1">
        <div className="text-xs font-medium text-muted-foreground">被谁邀请</div>
        {user.invited_by ? (
          <div className="min-w-0 text-sm">
            <div className="truncate font-medium">{user.invited_by.display_name}</div>
            <div className="truncate text-xs text-muted-foreground">
              {user.invited_by.account_identifier} · 使用邀请码{" "}
              {user.invited_by.referral_invite_code}
            </div>
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">自然注册</div>
        )}
      </div>
    </div>
  );
}

function Field({
  label,
  htmlFor,
  children,
}: {
  label: string;
  htmlFor: string;
  children: React.ReactNode;
}) {
  return (
    <div className="grid gap-2">
      <Label htmlFor={htmlFor}>{label}</Label>
      {children}
    </div>
  );
}

function Avatar({ user }: { user: AdminUser }) {
  if (user.avatar_url) {
    return (
      <img
        src={user.avatar_url}
        alt=""
        className="h-10 w-10 rounded-md object-cover"
      />
    );
  }

  return (
    <div className="flex h-10 w-10 items-center justify-center rounded-md bg-secondary text-sm font-semibold">
      {user.display_name.slice(0, 1).toUpperCase()}
    </div>
  );
}

function errorMessage(err: unknown) {
  if (err instanceof AdminApiError) {
    if (err.status === 401) return "管理密钥不正确";
    if (err.status === 503) return "后端未配置 ADMIN_API_TOKEN";
    return err.message;
  }
  if (err instanceof Error) return err.message;
  return "操作失败";
}
