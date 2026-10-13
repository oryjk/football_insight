import { useEffect, useRef, useState, type FormEvent } from "react";
import { Save } from "lucide-react";
import type { Admin, AdminApi } from "../../api";
import { ApiError, errorMessage, isAborted } from "../../apiClient";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import {
  credentialsPayload,
  CredentialsFormError,
  newCredentialsForm,
  type CredentialsForm,
} from "./credentialsModel";
export function SettingsPage({
  admin,
  api,
  token,
  onBusyChange,
  onUpdated,
}: {
  admin: Admin;
  api: AdminApi;
  token: string;
  onBusyChange: (busy: boolean) => void;
  onUpdated: (requestToken: string, username: string) => void;
}) {
  const [form, setForm] = useState(() => newCredentialsForm(admin.username));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [fieldError, setFieldError] = useState<{
    field: keyof CredentialsForm;
    message: string;
  } | null>(null);
  const lock = useRef(false);
  const request = useRef<AbortController | null>(null);
  useEffect(
    () => () => {
      request.current?.abort();
      onBusyChange(false);
    },
    [onBusyChange],
  );
  function change(field: keyof CredentialsForm, value: string) {
    setForm((current) => ({
      ...current,
      [field]: value,
      ...(field === "newPassword" && !value ? { confirmPassword: "" } : {}),
    }));
    setFieldError(null);
    setError("");
  }
  async function save(event: FormEvent) {
    event.preventDefault();
    if (lock.current) return;
    setError("");
    setFieldError(null);
    let payload;
    try {
      payload = credentialsPayload(form, admin.username);
    } catch (error) {
      if (error instanceof CredentialsFormError)
        setFieldError({ field: error.field, message: error.message });
      else setError(errorMessage(error));
      return;
    }
    lock.current = true;
    setBusy(true);
    onBusyChange(true);
    const controller = new AbortController();
    request.current = controller;
    try {
      await api.updateCredentials(token, payload, controller.signal);
      if (!controller.signal.aborted) {
        setForm(newCredentialsForm(payload.username));
        onUpdated(token, payload.username);
      }
    } catch (error) {
      if (!controller.signal.aborted && !isAborted(error)) {
        if (error instanceof ApiError && error.status === 403)
          setFieldError({
            field: "currentPassword",
            message: "当前密码不正确",
          });
        else if (error instanceof ApiError && error.status === 409)
          setFieldError({ field: "username", message: "管理员账号已存在" });
        else setError(errorMessage(error));
      }
    } finally {
      lock.current = false;
      if (!controller.signal.aborted) {
        setBusy(false);
        onBusyChange(false);
      }
    }
  }
  const invalid = (field: keyof CredentialsForm) => ({
    "aria-invalid": fieldError?.field === field,
    "aria-describedby":
      fieldError?.field === field ? `settings-${field}-error` : undefined,
  });
  const validation = (field: keyof CredentialsForm) =>
    fieldError?.field === field ? (
      <p id={`settings-${field}-error`} className="field-error" role="alert">
        {fieldError.message}
      </p>
    ) : null;
  return (
    <section className="settings-workspace" aria-label="管理员账号设置">
      <div className="page-section-header">
        <div>
          <h2>管理员账号</h2>
          <p className="page-description">修改当前管理员的登录账号和密码。</p>
        </div>
      </div>
      <dl className="settings-account-summary">
        <dt>当前账号</dt>
        <dd>{admin.username}</dd>
      </dl>
      <form
        onSubmit={save}
        className="settings-credentials-form"
        aria-busy={busy}
      >
        <div className="form-field">
          <Label htmlFor="settings-username">管理员账号</Label>
          <Input
            id="settings-username"
            name="username"
            autoComplete="username"
            value={form.username}
            disabled={busy}
            onChange={(event) => change("username", event.target.value)}
            {...invalid("username")}
          />
          <p className="field-hint">
            3–64 位英文字母、数字、点、下划线或连字符；保存时统一为小写。
          </p>
          {validation("username")}
        </div>
        <div className="form-field">
          <Label htmlFor="settings-current-password">当前密码</Label>
          <Input
            id="settings-current-password"
            name="current_password"
            type="password"
            autoComplete="current-password"
            value={form.currentPassword}
            disabled={busy}
            onChange={(event) => change("currentPassword", event.target.value)}
            {...invalid("currentPassword")}
          />
          {validation("currentPassword")}
        </div>
        <div className="form-field">
          <Label htmlFor="settings-new-password">新密码（选填）</Label>
          <Input
            id="settings-new-password"
            name="new_password"
            type="password"
            autoComplete="new-password"
            value={form.newPassword}
            disabled={busy}
            onChange={(event) => change("newPassword", event.target.value)}
            {...invalid("newPassword")}
          />
          <p className="field-hint">
            留空保留当前密码。填写时须为 8–128 个字符，空格也属于密码。
          </p>
          {validation("newPassword")}
        </div>
        {form.newPassword && (
          <div className="form-field">
            <Label htmlFor="settings-confirm-password">确认新密码</Label>
            <Input
              id="settings-confirm-password"
              name="confirm_password"
              type="password"
              autoComplete="new-password"
              value={form.confirmPassword}
              disabled={busy}
              onChange={(event) =>
                change("confirmPassword", event.target.value)
              }
              {...invalid("confirmPassword")}
            />
            {validation("confirmPassword")}
          </div>
        )}
        <p className="settings-session-notice">
          保存成功后，当前管理员在 Web 和 Android
          上的全部登录会话都会退出，需要使用更新后的账号和密码重新登录。
        </p>
        {error && (
          <p className="operation-error" role="alert">
            {error}
          </p>
        )}
        <div className="settings-form-actions">
          <Button type="submit" disabled={busy}>
            <Save />
            {busy ? "保存中…" : "保存账号设置"}
          </Button>
          {busy && (
            <span className="field-hint" role="status">
              正在更新账号，请等待完成。
            </span>
          )}
        </div>
      </form>
    </section>
  );
}
